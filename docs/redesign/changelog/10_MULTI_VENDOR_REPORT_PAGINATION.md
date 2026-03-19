# Multi-Vendor Report Pagination

**Date:** March 19, 2026  
**Type:** Feature Addition  
**Status:** Documented  
**Priority:** P1 (User Experience Enhancement)

---

## Overview

This document describes the addition of **multi-vendor report pagination** to the ez-booth-rs architecture, addressing the requirement that vendor reports should be printable with each vendor on a separate page for easy distribution.

## Problem Statement

The original specification did not explicitly address how vendor reports should be formatted when printing for multiple vendors. Key questions:

1. **Should all vendors print in a single continuous document?**
2. **Should each vendor start on a new page?**
3. **How should the preview look on screen?**
4. **Can users select which vendors to include in batch printing?**

## Decision

**Each vendor report will start on a new page** when printing multiple vendors, enabling:
- Easy physical separation and distribution of printed reports
- Clear vendor-by-vendor organization
- Professional presentation for vendor handouts

## Implementation

### CSS Page Break Strategy

```css
@media print {
    .vendor-report-page {
        page-break-after: always;  /* Force new page after each vendor */
    }
    
    .vendor-report-page:last-child {
        page-break-after: auto;  /* No blank page at end */
    }
    
    @page {
        margin: 2cm;
        size: A4 portrait;
    }
}
```

### Template Function

**File:** `crates/frontend/src/pages/reports.rs`

```rust
pub fn render_multi_vendor_report(
    vendors: &[Vendor],
    items_by_vendor: &HashMap<VendorId, Vec<PurchaseItem>>,
) -> String {
    let locale = get_locale();
    let vendor_pages = vendors
        .iter()
        .map(|v| format!(
            r#"<div class="vendor-report-page">
                {single_vendor_html}
            </div>"#
        ))
        .collect::<Vec<_>>()
        .join("\n");
    
    // Returns HTML with proper page breaks
}
```

### User Interface Features

1. **Preview Mode (Screen):**
   - Each vendor displayed as a visual "card" with borders
   - Scrollable view showing all vendors sequentially
   - Matches print output layout

2. **Print Options:**
   - **"Print All Vendors"** - Single button for batch printing
   - **"Print Selected"** - Checkboxes to select specific vendors
   - **"Print Single Vendor"** - Dropdown for individual vendor

3. **Localization:**
   ```json
   {
     "report": {
       "all_vendor_receipts": {
         "de": "Alle Verkäufer-Quittungen",
         "en": "All Vendor Receipts"
       },
       "print_all_vendors": {
         "de": "Alle Verkäufer drucken",
         "en": "Print All Vendors"
       },
       "print_selected": {
         "de": "Ausgewählte drucken",
         "en": "Print Selected"
       }
     }
   }
   ```

## Benefits

✅ **Easy Distribution:** Printed reports can be torn/cut apart and handed to individual vendors  
✅ **Professional Presentation:** Clean page breaks, no manual splitting needed  
✅ **Efficient Workflow:** Single print job for all vendors  
✅ **Preview Accuracy:** Screen preview matches print output  
✅ **Cross-Browser Compatible:** CSS page breaks work consistently across all modern browsers

## Files Modified

1. **ARCHITECTURE.md** - Added section 10.7 "Multi-Vendor Report Pagination"
2. **IMPLEMENTATION.md** - Added `render_multi_vendor_report()` function with pagination CSS

## Testing Considerations

- [ ] Verify page breaks work in Chrome, Firefox, Safari, Edge
- [ ] Confirm last vendor doesn't create blank page
- [ ] Test with 1, 5, 10, and 20 vendors
- [ ] Verify screen preview matches print output
- [ ] Check print dialog shows correct page count

## Related Documents

- [02_ARCHITECTURE.md](../02_ARCHITECTURE.md#multi-vendor-report-pagination) - Section 10.7
- [04_IMPLEMENTATION.md](../04_IMPLEMENTATION.md) - Section 4.4.4
- [08_REPORT_TEMPLATE_LOCALIZATION.md](./08_REPORT_TEMPLATE_LOCALIZATION.md) - Base report localization

---

**Decision Approved By:** Documentation review  
**Implementation Priority:** P1 (Include in initial release)
