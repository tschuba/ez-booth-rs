---
title: Validation Workflow
nav_order: 1
parent: Validation
---

# Validation Workflow

Use the smallest validation set that still proves the change is safe. Automated checks are the baseline; manual validation is required when operator behavior, browser behavior, or reporting trust changes.

## Validation Levels

| Level | When to use it | Command / artifact |
|-------|----------------|--------------------|
| Fast local regression | Any code or documentation change that touches compiled crates or testable behavior | `./run-tests.sh` |
| Browser-backed automated validation | UI, storage, or browser-sensitive changes | `./run-tests.sh --chrome` and, when relevant, `./run-tests.sh --safari` |
| Manual Safari validation | Safari-specific behavior, storage recovery, print layout, correction flows | `docs/validation/SAFARI_VALIDATION_CHECKLIST.md` |
| Guided bilingual UAT | Operator walkthroughs, stakeholder review, reusable acceptance sessions | `docs/validation/UAT_Ausfuehrungsplan_DE_EN.html` |
| Milestone sign-off record | When a phase or milestone needs durable evidence of manual validation | milestone-specific results file such as `docs/validation/PHASE2_M2_VALIDATION_RESULTS.md` |

## When Manual Validation Is Required

Run a manual session when a change affects any of these areas:

- checkout correction or deletion flows
- recovery guidance, draft restore, or corrupted-storage messaging
- report totals, payout trust, or print layout
- Safari-specific behavior
- onboarding or operator instructions that people will follow during an event

## Artifact Guide

### `docs/validation/SAFARI_VALIDATION_CHECKLIST.md`

Use this for focused Safari sessions. It is the quickest reusable checklist for checkout, correction, recovery, reporting, and print validation.

Best for:
- confirming a bug fix in Safari
- checking totals after destructive actions
- capturing pass/fail evidence during development

### `docs/validation/UAT_Ausfuehrungsplan_DE_EN.html`

Use this when the tester should follow an on-screen or printable guided script. The document is bilingual and works well for operator walkthroughs or stakeholder review.

Best for:
- structured acceptance testing
- mixed German/English sessions
- repeatable operator exercises such as Module H correction and deletion workflows

### `docs/planning/PHASE1_M2_PREPARATION.md`

Use this as background context when the current validation work builds on the safety and recovery track from Phase 1. It explains why the recovery-focused artifacts exist and what risks they were designed to cover.

Best for:
- understanding validation scope before extending a checklist
- tracing the intent behind recovery and corruption scenarios

### Milestone result files

Use a milestone result file when a change needs a durable sign-off record beyond a checklist. `docs/validation/PHASE2_M2_VALIDATION_RESULTS.md` is the current example.

Best for:
- recording who tested what and when
- summarizing checklist and UAT outcomes in one place
- documenting whether acceptance criteria were met before merge

## Recommended Session Flow

1. Run the fastest automated checks that fit the change.
2. Start the app from `crates/ez-booth-app` with `trunk serve`.
3. Pick the manual artifact that matches the risk:
   - Safari-focused issue: use `docs/validation/SAFARI_VALIDATION_CHECKLIST.md`
   - operator walkthrough or sign-off: use `docs/validation/UAT_Ausfuehrungsplan_DE_EN.html`
4. If the work belongs to a milestone, copy results into the milestone result file.
5. Reference the validation in the pull request description.

## Example Result Entry

Use short factual notes. One good entry is enough; avoid writing a transcript.

```md
| Scenario | Status | Notes |
|----------|--------|-------|
| 2. Booth Summary Consistency And Correction Workflow | PASS | Deleted one checkout, totals recalculated immediately, vendor report excluded deleted purchase. |
```

For a milestone result file, the same session can be summarized like this:

```md
- [x] Deletion success toast shown
- [x] Running totals updated after deletion
- [x] Vendor reports unchanged after vendor deletion
Notes: Aftercare toast was clear enough for an operator without console access.
```

## Sign-Off Criteria

Validation is strong enough to merge when:

- the relevant automated checks pass
- manual scenarios that match the change risk have been executed
- any checklist or milestone result file records the real outcome
- deferred validation is called out explicitly in the pull request when it cannot be completed yet

If a change updates operator-facing workflows, the corresponding validation artifacts in `docs/validation/` should usually be updated in the same branch.
