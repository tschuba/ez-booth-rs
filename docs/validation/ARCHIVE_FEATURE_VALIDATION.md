# Archive Feature Validation

## Archive Workflow

- [ ] Archive action appears only for active booths in the booth list
- [ ] Archive wizard review step shows the expected vendor and purchase counts
- [ ] Backup export must complete before archive confirmation is enabled
- [ ] Archive confirmation requires the correct token
- [ ] Archived booth moves into the archived section after completion
- [ ] Archived booth is removed from active selection lists

## Archived Summary

- [ ] Archived booth modal title uses the localized archived summary label
- [ ] Archived booth modal shows non-zero summary totals when source data exists
- [ ] Archived booth print preview matches the on-screen archived summary totals
- [ ] Archived booth report shows vendor totals without transaction details

## Active Flow Protection

- [ ] Checkout clears an archived booth selection and shows an error toast
- [ ] Vendor list clears an archived booth selection and shows an error toast
- [ ] Archiving a booth in another tab clears the active selection in this tab
- [ ] Archived booths do not expose delete actions in the booth list

## Diagnostics And UI

- [ ] Settings diagnostics shows archive history entries
- [ ] Archive history rows show action, event, device, and timestamp
- [ ] Archived booth cards show a localized archived timestamp
- [ ] Booth selector explains when all available booths are archived
- [ ] Archived section expand/collapse preference persists after reload

## Restore Via Import

- [ ] Importing an archived booth backup restores the booth to active use
- [ ] Restored booth returns with vendors and purchases from the imported backup
