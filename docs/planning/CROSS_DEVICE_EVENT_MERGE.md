# Cross-Device Event Merge — Implementation Plan

> Extracted from [TESTING_SESSION_FIXES_2026-05-21.md](TESTING_SESSION_FIXES_2026-05-21.md) (Issues 4–6)

**Context:** When the same event is created independently on two devices (e.g., device A and device B both create "Spring Market 2026-05-10"), they receive different booth UUIDs. Importing device B's data into device A currently creates a duplicate entry instead of merging. Issues 1–3 in this plan fix the import logic, surface the merge decision to the user before import, and provide a cleanup tool for duplicates that already exist.

---

## Issue 1 — Import of same-event file from another device creates a duplicate instead of merging

**Root cause:** `import_booth_record` (`crates/storage/src/export/import_service.rs:135`) matches only by `booth.id`. When device B creates the same event independently, it receives a different UUID. The `find_by_description_and_date` index exists but is never consulted during import.

**Fix:** In `import_booth_record`, after no ID match, fall back to `find_by_description_and_date`. When found, treat it as a conflict (apply `ConflictStrategy`) using the **existing local booth ID as canonical**. Return the canonical `BoothId` so callers can remap subordinate records.

### Changes in `crates/storage/src/export/import_service.rs`

1. **`import_booth_record`** — change return type to `Result<BoothId, ImportError>`:
   - No match by ID → also try `find_by_description_and_date(&incoming.description, &incoming.date)`
   - Name+date match found: apply strategy, always return `existing.id` as canonical
     - Skip → push `SkippedRecord`, return `existing.id`
     - Replace → save `Booth { id: existing.id, ..incoming }`, return `existing.id`
     - Merge → pick newer `updated_at`, save with `existing.id`, return `existing.id`
   - No match at all → save incoming as-is, return `incoming.id`
   - ID match found (existing path) → unchanged logic, return `incoming.id`

2. **`import_booth_backup`** — use returned canonical ID to remap vendors/purchases:
   - After calling `import_booth_record`, if canonical ID differs from `data.booth.id`, rewrite all `vendor.booth_id` and `purchase.booth_id` fields before importing them.

3. **`import_all`** — build a `HashMap<incoming_BoothId, canonical_BoothId>` from the booth pass, then remap all vendor/purchase `booth_id` fields before importing.

4. **`import_booth_backup_restoring_archived`** — the archived-booth restore check at line 116 also needs to try `find_by_description_and_date` so restoring works for cross-device events.

### New tests in `crates/storage/tests/import_service_tests.rs`

- Import a `BoothBackupData` where booth has same description+date but different ID → no new booth created, vendors/purchases stored under existing booth ID
- Same test with `ConflictStrategy::Skip` → booth metadata not updated, vendors/purchases still imported under existing ID
- Same test with `ConflictStrategy::Replace` → existing booth metadata replaced, canonical ID preserved

### Verification

Run `wasm-pack test --headless --chrome`; confirm no duplicate booth is created when importing a same-name/date event from device B.

---

## Issue 2 — Import modal: pre-import analysis and smart merge suggestions

**Depends on Issue 1** (the name+date fallback merge must be implemented first so the analysis reflects actual import behaviour).

**Problem:** The import modal currently shows only counts (booths/vendors/purchases) with no indication of whether each event will be merged into an existing one or created fresh. When a name+date match triggers Issue 1's new merge path, the user has no way to see that or verify it before confirming.

### New backend: `analyze_import` on `ImportService`

Add a read-only async method that produces a dry-run analysis without writing anything:

```rust
pub async fn analyze_import(&self, data: &ParsedImportData) -> Result<ImportAnalysis, ImportError>
```

New types (in `crates/storage/src/export/`):

```rust
pub enum BoothMatchKind {
    ById,           // incoming.id matches an existing booth
    ByNameAndDate,  // different ID, but same description + date
}

pub struct BoothImportAnalysis {
    pub incoming_id: BoothId,
    pub incoming_description: String,
    pub incoming_date: NaiveDate,
    pub match_kind: Option<BoothMatchKind>,  // None = will be created new
    pub existing_id: Option<BoothId>,
    pub vendor_count: usize,
    pub purchase_count: usize,
}

pub struct ImportAnalysis {
    pub booths: Vec<BoothImportAnalysis>,
}
```

For each booth in the import file, `analyze_import` applies the same lookup sequence as Issue 1 (`find_by_id` → `find_by_description_and_date`) and records the outcome without persisting.

### UI changes in `crates/ez-booth-ui/src/components/import_button.rs`

After parsing a file, call `analyze_import` and store the result alongside the candidate. Extend `ImportCandidate` with `analysis: Option<ImportAnalysis>`.

In the candidate card rendering (currently shows simple counts), add a per-booth outcome row for each booth in the analysis:

| Outcome | Indicator |
| ------- | --------- |
| No existing match | "New event" — neutral/blue |
| ID match | "Will merge (same device)" — green |
| Name+date match, different ID | "Suggested merge — same event from another device" — amber, prominent |

For the **name+date suggestion** case: since Issue 1 will automatically merge these, the amber indicator serves as a confirmation prompt ("this import will merge into your existing event X"). The selected conflict strategy still applies (merge/skip/replace).

### Verification

Import a booth backup from device B (same event name/date, different ID); confirm the modal shows the amber "Suggested merge" indicator before applying.

---

## Issue 3 — Merge locally duplicated events (same name + date, different ID)

**Context:** Before Issue 1's fix, importing from device B created a second booth entry on device A even when the event already existed. Devices may also have independently created the same event. This leaves existing installs with duplicate entries that the import fix won't clean up retroactively.

**Trigger:** Any two booths on the same device sharing `(description.trim(), date)` but different IDs are duplicates that should be mergeable.

### Detection

Add a method to `BoothRepository` that returns groups of booths with the same `(description.trim(), date)`:

```rust
async fn find_duplicate_groups(&self) -> DomainResult<Vec<Vec<Booth>>>;
```

Implemented via the existing `[description, date]` IndexedDB index — iterate all booths and group by key.

### Merge logic

Reuse the ID-remapping infrastructure from Issue 1. Given two booths to merge:

1. Pick the **canonical booth**: the one with more purchases; tie-break by earlier `created_at`
2. Re-assign all vendors and purchases from the other booth to the canonical booth ID (same remap logic as `import_booth_backup`)
3. Merge booth metadata: keep the canonical booth's ID, prefer the newer `updated_at` for other fields (same as Issue 1's Merge strategy)
4. Delete the now-empty non-canonical booth

Expose as a method on `ImportService` or a dedicated `MergeService`:

```rust
async fn merge_booths(&self, canonical_id: &BoothId, other_id: &BoothId) -> DomainResult<()>;
```

### UI

Surface in the event list page:

- On load, run `find_duplicate_groups`; if any groups found, show an amber warning banner: *"X events share the same name and date. Merge duplicates?"*
- Clicking opens a modal listing each duplicate group with booth name, date, purchase counts, and a "Merge" button per group
- After merge, re-fetch the event list and dismiss the banner

### Notes

- Shares the booth-ID remapping logic with Issue 1 — extract into a shared helper to avoid duplication
- Issue 2's analysis step will no longer flag these pairs after they are merged

### Verification

With existing duplicate events (same name+date) present, open the event list — confirm the amber warning banner appears and the merge modal correctly consolidates vendors and purchases under the canonical booth.

---

## Files to modify (summary)

| File | Issues |
| ---- | ------ |
| `crates/storage/src/export/import_service.rs` | 1, 2 |
| `crates/storage/tests/import_service_tests.rs` | 1 |
| `crates/ez-booth-ui/src/components/import_button.rs` | 2 |
| `crates/domain/src/repositories.rs` | 3 |
| `crates/storage/src/repositories/` (IndexedDB impl) | 3 |
