## Why

Die Netto-Auszahlung kann negative Werte annehmen, wenn der Bruttoumsatz eines Verkäufers kleiner oder gleich der Teilnahmegebühr ist. Laut Abrechnungsformel muss das Netto bei `0,00 €` gefloor't werden — dies wird in `calculate_payout()` nicht umgesetzt. Das Problem wurde im Validierungstest mit Verkäufer V8 (Brutto 0,50 €, TG 1,00 €) nachgewiesen: App zeigt `−0,50 €` statt `0,00 €`.

## What Changes

- `ChargingConfig::calculate_payout()` in `crates/domain/src/services/dto.rs` wendet nach dem Runden einen Floor bei `0,00 €` an (`net_payout.max(Decimal::ZERO)`)
- `ChargingConfig::calculate_fees()` (deprecated) wird konsistenzhalber geprüft, ob derselbe Floor dort ebenfalls fehlt
- Bestehende Unit-Tests für den Grenzfall werden ergänzt bzw. korrigiert

## Capabilities

### New Capabilities

*(keine neuen Capabilities — reine Bugfix-Korrektur)*

### Modified Capabilities

- `booth-fees-calculation`: Das Berechnungsverhalten ändert sich für den Grenzfall `theoretical_net < 0`: Netto wird auf `0,00 €` geklemmt statt negative Werte zu liefern

## Impact

**Geänderte Dateien:**
- `crates/domain/src/services/dto.rs` — eine Zeile: `.max(Decimal::ZERO)` nach `round_to_step()`

**Betroffene Szenarien:**
- Verkäufer deren Bruttoumsatz ≤ Teilnahmegebühr (z. B. Brutto 0,50 € bei TG 1,00 €)
- Verkäufer mit Bruttoumsatz = 0,00 € (bereits korrekt: theoretical_net = −1,00 → net = −1,00 statt 0,00)

**Keine Breaking Changes** — bestehende Berechnungen mit positivem theoretical_net sind unverändert.
