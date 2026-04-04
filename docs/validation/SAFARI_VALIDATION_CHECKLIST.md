# Safari Validation Checklist

Focused manual validation for checkout reliability, data consistency, and report correctness in Safari.

This checklist also covers browser-local backup, import recovery, and storage-warning comprehension.

## Test Session

- Safari version: `26.4 (21624.1.16.11.4)`
- App URL: `http://127.0.0.1:8080`
- Branch / build: __________________
- Tester: __________________
- Date: __________________

## Preparation

1. Start the app with `trunk serve` from `crates/ez-booth-app`.
2. Open Safari Developer Tools and clear previous console noise.
3. If needed, clear IndexedDB and localStorage before starting a fresh session.
4. Keep a calculator ready for manual fee verification.
5. Prepare a writable folder outside the browser for downloaded JSON backups.

## Scenario 1: Base Checkout Flow

### Steps
- [ ] Create a booth with participation fee `10.00`, sales fee `15`, rounding `0.50`
- [ ] Create vendors `1`, `2`, `3`
- [ ] Add one checkout each for `100.00`, `518.11`, `75.25`
- [ ] Verify all checkouts save successfully

### Expected Values

| Vendor | Gross sales | Participation fee | Sales fee | Total fees | Payout |
|--------|-------------|-------------------|-----------|------------|--------|
| 1 | 100.00 | 10.00 | 15.00 | 25.00 | 75.00 |
| 2 | 518.11 | 10.00 | 77.50 | 87.50 | 430.61 |
| 3 | 75.25 | 10.00 | 11.50 | 21.50 | 53.75 |

### Result Comparison

| Vendor | Expected sales fee | Actual sales fee | Match | Notes |
|--------|--------------------|------------------|-------|-------|
| 1 | 15.00 | __________ | [ ] | |
| 2 | 77.50 | __________ | [ ] | |
| 3 | 11.50 | __________ | [ ] | |

Performance notes: ________________________________________________

## Scenario 2: Booth Summary Consistency And Correction Workflow

- [ ] Open booth summary / report view
- [ ] Verify the booth report totals match the sum of vendor fees
- [ ] Print the booth summary
- [ ] Verify printed values match the on-screen values
- [ ] Delete one checkout and verify the success message states that totals and reports were recalculated
- [ ] Verify the updated totals match the remaining recoverable purchases
- [ ] Open vendor report and verify it only includes remaining purchases
- [ ] Open booth summary and verify totals are correct for remaining purchases
- [ ] Create a new vendor without purchases (for example vendor `99`)
- [ ] Open vendor deletion view and verify only vendors without purchases appear in the list
- [ ] Verify the safe-delete hint is visible
- [ ] Delete the vendor without purchases
- [ ] Verify aftercare toast explains the deletion policy clearly
- [ ] Verify vendor reports remain unchanged after vendor deletion

### Expected Totals

| Metric | Expected | Actual | Match |
|--------|----------|--------|-------|
| Total sales | 693.36 | __________ | [ ] |
| Total participation fees | 30.00 | __________ | [ ] |
| Total sales fees | 104.00 | __________ | [ ] |
| Total booth revenue | 134.00 | __________ | [ ] |

Calculation note: `vendor fees sum = 25.00 + 87.50 + 21.50 = 134.00`

### Correction Workflow Results

| Check | Expected | Actual | Match |
|-------|----------|--------|-------|
| Deletion success toast shown | Yes | __________ | [ ] |
| Deletion message mentions recalculation | Yes | __________ | [ ] |
| Running totals update after deletion | Yes | __________ | [ ] |
| Vendor report excludes deleted purchase | Yes | __________ | [ ] |
| Booth summary correct after deletion | Yes | __________ | [ ] |
| Only vendors without purchases in delete list | Yes | __________ | [ ] |
| Safe-delete hint visible | Yes | __________ | [ ] |
| Aftercare toast after vendor deletion | Yes | __________ | [ ] |
| Vendor reports unchanged after vendor deletion | Yes | __________ | [ ] |

Performance notes: ________________________________________________

## Scenario 3: Validation Rejection

- [ ] Enter invalid amount `10.123` and confirm checkout is rejected
- [ ] Enter negative amount and confirm checkout is rejected
- [ ] Enter zero amount and confirm checkout is rejected
- [ ] Confirm no invalid checkout is stored
- [ ] Confirm no uncaught console errors appear

### Validation Result Table

| Input | Expected result | Actual result | Match |
|------|------------------|---------------|-------|
| 10.123 | Rejected | __________ | [ ] |
| -5.00 | Rejected | __________ | [ ] |
| 0.00 | Rejected | __________ | [ ] |

## Scenario 4: Draft Persistence And Recovery

- [ ] Enter a draft checkout without submitting
- [ ] Refresh Safari
- [ ] Verify the draft is restored
- [ ] Submit the restored draft successfully
- [ ] Verify the draft is cleared afterwards

### Result Comparison

| Check | Expected | Actual | Match |
|-------|----------|--------|-------|
| Draft restored after refresh | Yes | __________ | [ ] |
| Restored data matches input | Yes | __________ | [ ] |
| Successful submit after recovery | Yes | __________ | [ ] |
| Draft cleared after successful save | Yes | __________ | [ ] |

Performance notes: ________________________________________________

## Scenario 5: Corruption Warning And Operator Guidance

- [ ] Prepare one valid purchase for the booth
- [ ] Corrupt one purchase record manually in IndexedDB
- [ ] Refresh checkout view in Safari
- [ ] Verify a visible warning appears about skipped purchases
- [ ] Verify healthy purchases still load
- [ ] Verify visible totals only reflect recoverable purchases
- [ ] Verify the warning remains visible while reviewing totals and recent transactions
- [ ] Review totals before continuing and note whether operator guidance is clear enough
- [ ] Verify the operator guidance explains what to do next clearly enough without console inspection

### Result Comparison

| Check | Expected | Actual | Match |
|-------|----------|--------|-------|
| Warning toast shown | Yes | __________ | [ ] |
| Healthy purchases still visible | Yes | __________ | [ ] |
| Corrupted purchase excluded | Yes | __________ | [ ] |
| Totals match recoverable data only | Yes | __________ | [ ] |

Operator notes: _________________________________________________

## Scenario 6: Display And Printing Accuracy

- [ ] Verify currency values use two decimal places consistently
- [ ] Verify report print layout keeps values readable
- [ ] Verify no columns are truncated in print preview
- [ ] Verify on-screen and printed payouts match exactly

### Print Result Table

| Artifact | On-screen value matches printed value | Notes |
|----------|---------------------------------------|-------|
| Vendor report | [ ] | |
| Booth summary | [ ] | |

## Scenario 7: Performance Observations

| Action | Observed time | Acceptable | Notes |
|--------|---------------|------------|-------|
| Booth creation | __________ | [ ] | |
| Single checkout submit | __________ | [ ] | |
| Three-checkout report generation | __________ | [ ] | |
| Print preview load | __________ | [ ] | |

## Scenario 8: Backup Export And Warning Visibility

- [ ] Open the booth list and verify the permanent storage warning is visible
- [ ] Verify the global warning banner appears until dismissed
- [ ] Export a full backup and confirm a `.json` file downloads successfully
- [ ] Export a booth backup and confirm a `.json` file downloads successfully
- [ ] Verify both downloaded files can be opened as readable JSON text
- [ ] Verify the operator warning copy is understandable without developer explanation

### Result Comparison

| Check | Expected | Actual | Match |
|-------|----------|--------|-------|
| Booth list warning visible | Yes | __________ | [ ] |
| Global banner visible before dismissal | Yes | __________ | [ ] |
| Full backup downloaded | Yes | __________ | [ ] |
| Booth backup downloaded | Yes | __________ | [ ] |
| Warning copy understandable | Yes | __________ | [ ] |

Operator notes: _________________________________________________

## Scenario 9: Backup Import And Recovery

- [ ] Start with at least one booth that contains vendors and purchases
- [ ] Export a full backup and one booth backup
- [ ] Delete one booth from the app
- [ ] Import the booth backup with `Merge` or `Replace`
- [ ] Verify the deleted booth is recreated
- [ ] Verify the booth list refreshes immediately after import
- [ ] Import the full backup
- [ ] Verify booths, vendors, and purchases are present afterwards
- [ ] Repeat one import with `Skip` and confirm conflicting records are skipped safely

### Result Comparison

| Check | Expected | Actual | Match |
|-------|----------|--------|-------|
| Deleted booth restored from booth backup | Yes | __________ | [ ] |
| Booth list refreshed after import | Yes | __________ | [ ] |
| Full backup import succeeds | Yes | __________ | [ ] |
| Conflict handling message is clear | Yes | __________ | [ ] |
| Skip strategy keeps existing records | Yes | __________ | [ ] |

Recovery notes: _________________________________________________

## Scenario 10: Multi-Device Booth Merge

- [ ] Create one booth with at least one vendor and one purchase on Device A
- [ ] Export that booth backup from Device A
- [ ] Import the booth backup on Device B
- [ ] Add at least one new purchase on Device B
- [ ] Export the booth backup from Device B
- [ ] On Device A, import the Device B booth backup with `Merge`
- [ ] Verify the original purchase from Device A is still present
- [ ] Verify the new purchase from Device B is now present
- [ ] Verify vendor names remain correct after merge
- [ ] If the same booth was edited on both devices, verify the strictly newer booth update wins

### Result Comparison

| Check | Expected | Actual | Match |
|-------|----------|--------|-------|
| Original purchase still present | Yes | __________ | [ ] |
| New purchase from second device present | Yes | __________ | [ ] |
| Duplicate shared-history purchases created | No | __________ | [ ] |
| Vendor names remain correct | Yes | __________ | [ ] |
| Booth merge behavior understood | Yes | __________ | [ ] |

Merge notes: _________________________________________________

## Console / Error Review

- [ ] No uncaught console errors during checkout/report flow
- [ ] No unexpected storage or serialization errors
- [ ] Any warnings are understood and documented below

Notes: _____________________________________________________________

## Final Result

| Area | Status | Notes |
|------|--------|-------|
| Checkout correctness | [ ] Pass / [ ] Fail | |
| Data consistency | [ ] Pass / [ ] Fail | |
| Report accuracy | [ ] Pass / [ ] Fail | |
| Recovery guidance | [ ] Pass / [ ] Fail | |
| Backup and restore workflow | [ ] Pass / [ ] Fail | |
| Correction workflows | [ ] Pass / [ ] Fail | |
| Performance | [ ] Pass / [ ] Fail | |

Overall outcome: [ ] PASS / [ ] FAIL

Sign-off: ______________________________
