# Safari Validation Checklist

Focused manual validation for checkout reliability, data consistency, and report correctness in Safari.

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

## Scenario 2: Booth Summary Consistency

- [ ] Open booth summary / report view
- [ ] Verify the booth report totals match the sum of vendor fees
- [ ] Print the booth summary
- [ ] Verify printed values match the on-screen values
- [ ] Delete one checkout and verify the success message states that totals and reports were recalculated
- [ ] Verify the updated totals match the remaining recoverable purchases

### Expected Totals

| Metric | Expected | Actual | Match |
|--------|----------|--------|-------|
| Total sales | 693.36 | __________ | [ ] |
| Total participation fees | 30.00 | __________ | [ ] |
| Total sales fees | 104.00 | __________ | [ ] |
| Total booth revenue | 134.00 | __________ | [ ] |

Calculation note: `vendor fees sum = 25.00 + 87.50 + 21.50 = 134.00`

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
| Performance | [ ] Pass / [ ] Fail | |

Overall outcome: [ ] PASS / [ ] FAIL

Sign-off: ______________________________
