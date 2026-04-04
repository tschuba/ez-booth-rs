# Multi-Device Booth Merge Guide

This guide explains the safest current workflow for working on one booth across multiple devices and then merging the booth data back together.

## Recommended Use Case

Use this workflow when:

- one event booth is used on multiple laptops or tablets
- each device may record purchases while offline
- the team needs to merge those purchases back into one booth safely

This guide is about booth backups, not cloud sync.

## Safe Workflow

1. Create or confirm one known-good booth backup.
2. Import that booth backup onto every device that will be used for the same booth.
3. Record new purchases on each device.
4. Export a booth backup from each device.
5. Choose one target device as the merge device.
6. Import each device backup on the target device with `Merge`.
7. Verify vendor list, purchase count, and booth totals.
8. Create one new booth backup from the merged result.

## What `Merge` Safely Does

- imports new booths, vendors, and purchases that do not exist locally yet
- keeps both purchases when the devices created different purchase IDs
- chooses the strictly newer booth record when the same booth was updated on multiple devices
- chooses the strictly newer purchase record when the same purchase ID was updated on multiple devices
- keeps the local record when booth or purchase timestamps are exactly equal
- keeps a non-empty vendor name and prefers the richer vendor name when names differ

## What `Merge` Does Not Try To Do

- it does not guess that two different purchase IDs are the same real-world sale
- it does not combine two conflicting booth edits field by field
- it does not provide a manual conflict-resolution UI
- it does not make multi-file import atomic

## Practical Team Rules

- prefer booth backups over full backups for single-booth device transfer
- import the latest known booth backup before entering more data on another device
- keep all exported files until the merged result is verified
- after merging, create a new backup and treat that file as the current recovery point
- if totals look wrong, stop and compare the imported purchase list before recording more sales

## Verification Checklist After Merge

- the expected booth is present
- vendor count looks correct
- recently added vendors still have the expected names
- purchase count matches the combined expected count from all devices
- latest purchase notes or corrections are present where expected
- booth totals match the combined purchases
- no unrelated booths were changed

## When To Use `Skip` Or `Replace`

- use `Skip` when the local device should stay authoritative and imported conflicts should be ignored
- use `Replace` when one backup should fully overwrite conflicting local records
- use `Merge` for the normal multi-device booth workflow

## Current Limits To Communicate To Operators

- if two devices independently record the same real-world sale as two different purchases, EZ Booth will keep both because the purchase IDs are different
- if two devices edit the same booth or purchase at the exact same timestamp, the target device keeps its existing local record
- if you import several files and a later file fails, earlier successful imports are already applied

## Validation Status

The storage-layer merge behavior for this workflow has automated browser-backed coverage for:

- repeated import of shared booth history without duplicate records
- parallel multi-device booth purchase merges
- round-trip imports
- same-purchase conflict resolution by newer timestamp
- richer vendor-name convergence
- mixed booth-backup and full-backup merge sequences

For cross-browser operator validation, also use:

- `docs/SAFARI_VALIDATION_CHECKLIST.md`
- `docs/DATA_BACKUP_GUIDE.md`
- `docs/VALIDATION_WORKFLOW.md`
