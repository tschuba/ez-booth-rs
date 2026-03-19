# Print Implementation Review

## Current Implementation (Java/Thymeleaf)

### Approach
The existing ez-booth application uses **browser-native printing** with CSS print media queries.

### Technical Details

**1. CSS Print Styles** (`reports/styles/default.css`):
```css
@media print {
    .page-break-after {
        page-break-after: always;
        break-after: always;  /* Modern CSS spec */
    }
    
    .page-break-before {
        page-break-before: always;
        break-before: always;
    }
}
```

**2. Template Logic** (`VendorReport.template.html`):
```html
<main class="p-m"
      th:classappend="${iteration.size > 1 && !iteration.last} ? 'page-break-after' : ''"
      th:each="report, iteration: ${reportData}">
    <!-- Vendor report content -->
</main>
```

**Logic:**
- When iterating through multiple vendors (`iteration.size > 1`)
- Add `page-break-after` class to all but the last vendor (`!iteration.last`)
- Result: Each vendor prints on a separate page, no blank page at end

**3. User Interaction:**
- User clicks "Print" button in UI
- Invokes browser's native `window.print()` dialog
- Browser applies CSS `@media print` rules
- Multi-page PDF can be saved or sent to printer

## Rust Implementation Strategy

### Same Approach, Different Template Engine

**Keep:**
- ✅ CSS `@media print` media queries
- ✅ Conditional `page-break-after` classes
- ✅ Browser `window.print()` API
- ✅ HTML template generation

**Change:**
- Replace Thymeleaf with Rust template engine (Tera or Askama)
- Generate HTML in Rust/WASM instead of server-side Java

### Proposed Rust Implementation

**1. Template Engine: Tera**
```rust
use tera::{Tera, Context};

pub fn generate_vendor_reports(vendors: Vec<VendorReport>) -> String {
    let mut tera = Tera::new("templates/**/*.html").unwrap();
    let mut context = Context::new();
    context.insert("vendors", &vendors);
    context.insert("vendor_count", &vendors.len());
    
    tera.render("vendor_report.html", &context).unwrap()
}
```

**2. Tera Template** (`templates/vendor_report.html`):
```html
{% for vendor in vendors %}
<main class="p-m {% if vendors | length > 1 and not loop.last %}page-break-after{% endif %}">
    <h2>{{ t.vendor_receipt }}</h2>
    
    <div class="vendor-header">
        <span>{{ t.vendor_number }}: #{{ vendor.id }}</span>
        <span>{{ t.total }}: {{ vendor.total | format_currency }}</span>
    </div>
    
    <!-- Sales items -->
    <div class="items-grid">
        {% for item in vendor.items %}
        <div class="item">
            <span>{{ item.price | format_currency }}</span>
            <span>{{ item.time | format_time }}</span>
        </div>
        {% endfor %}
    </div>
</main>
{% endfor %}
```

**3. CSS (same as current):**
```css
@media print {
    .page-break-after {
        page-break-after: always;
        break-after: always;
    }
}

@media not print {
    main {
        border-bottom: 1px dashed;
        margin: 2rem 0;
    }
}
```

**4. Svelte Component** (triggers print):
```svelte
<script lang="ts">
    import { invoke } from '@tauri-apps/api/tauri';
    
    async function printVendorReports(vendorIds: number[]) {
        // Generate HTML via Rust/WASM
        const html = await invoke('generate_vendor_reports', { vendorIds });
        
        // Open in new window and print
        const printWindow = window.open('', '_blank');
        printWindow.document.write(html);
        printWindow.document.close();
        printWindow.print();
    }
</script>

<button on:click={() => printVendorReports(selectedVendors)}>
    {$t('reports.print_all')}
</button>
```

## Advantages of This Approach

| Benefit | Description |
|---------|-------------|
| **No dependencies** | Uses native browser APIs, no PDF libraries needed |
| **Small footprint** | Minimal CSS (~2KB), no PDF generation overhead |
| **Cross-platform** | Works on all browsers (Chrome, Firefox, Safari, Edge) |
| **Print preview** | Users see preview before printing |
| **PDF export** | Browsers can "Print to PDF" natively |
| **Familiar UX** | Standard print dialog users already know |
| **Easy styling** | CSS media queries for print customization |
| **Localization-friendly** | Templates support i18n via Tera filters |

## Localization Integration

**Tera Custom Filters:**
```rust
use tera::Tera;

pub fn register_filters(tera: &mut Tera, locale: &str) {
    // Currency formatting
    tera.register_filter("format_currency", |value, _| {
        // Format based on locale (EUR for de, USD for en, etc.)
    });
    
    // Time formatting
    tera.register_filter("format_time", |value, _| {
        // Format based on locale (24h for de, 12h for en)
    });
}
```

**Translation Loading:**
```rust
pub fn load_translations(locale: &str) -> HashMap<String, String> {
    // Load from embedded JSON files
    match locale {
        "de" => serde_json::from_str(include_str!("i18n/de.json")),
        "en" => serde_json::from_str(include_str!("i18n/en.json")),
        _ => serde_json::from_str(include_str!("i18n/en.json")), // Fallback
    }
}
```

## Testing Strategy

**Manual Testing:**
1. Generate report for single vendor → verify no page break
2. Generate report for 2 vendors → verify page break between them
3. Generate report for 10 vendors → verify 10 pages, no blank page at end
4. Test in Chrome, Firefox, Safari
5. Test "Print to PDF" functionality
6. Test different paper sizes (A4, Letter)

**Automated Testing:**
```rust
#[test]
fn test_single_vendor_no_page_break() {
    let html = generate_vendor_reports(vec![mock_vendor(1)]);
    assert!(!html.contains("page-break-after"));
}

#[test]
fn test_multiple_vendors_with_page_breaks() {
    let html = generate_vendor_reports(vec![
        mock_vendor(1),
        mock_vendor(2),
        mock_vendor(3),
    ]);
    
    // First two should have page breaks
    let page_breaks = html.matches("page-break-after").count();
    assert_eq!(page_breaks, 2);
}
```

## Migration from Current System

**No data migration needed** - This is purely a presentation layer change.

**Steps:**
1. ✅ CSS styles already defined (copy from current system)
2. ✅ Template structure documented (Thymeleaf → Tera mapping)
3. Implement Rust template rendering function
4. Create Svelte print button component
5. Test cross-browser compatibility
6. Deploy alongside existing system

## Open Questions

- [x] **How are page breaks handled?** → CSS `page-break-after: always`
- [x] **Can each vendor print separately?** → Yes, via conditional class application
- [x] **Is PDF generation needed?** → No, browser "Print to PDF" sufficient
- [ ] **What about QR code generation?** → TBD (separate feature)
- [ ] **Preview before printing?** → Native print preview dialog

## Timeline Estimate

| Task | Effort | Dependencies |
|------|--------|--------------|
| Tera template setup | 2h | None |
| Report generation logic | 4h | Database queries |
| CSS print styles | 1h | None |
| Svelte print component | 2h | Tera integration |
| Cross-browser testing | 3h | Template complete |
| **Total** | **12h** | **Phase 2** |

## References

- Current CSS: `/server/src/main/resources/reports/styles/default.css`
- Current template: `/server/src/main/resources/reports/templates/VendorReport.template.html`
- Architecture doc: Section 5.4.3 "Report Templates & Localization"
- Implementation doc: Step 2.3 "Report Generation Module"
