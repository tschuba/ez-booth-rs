## Context

Die Abrechnungsformel lautet:
```
theoretical_net = brutto − participation_fee − brutto × sales_fee_percent
net = commercial_round(theoretical_net, rounding_step)   // floor bei 0,00 €
fees = brutto − net
```

In `ChargingConfig::calculate_payout()` (`crates/domain/src/services/dto.rs`, Zeile ~108) wird `round_to_step(theoretical_net)` aufgerufen, aber der anschließende Floor bei `0,00 €` fehlt. `round_to_step()` wendet `MidpointAwayFromZero` an — für negative Werte rundet es korrekt auf die nächste negative Stufe, anstatt auf 0,00 zu klemmen.

`TransactionService::calculate_payout()` delegiert direkt an `ChargingConfig::calculate_payout()` — ein einziger Fix am Ursprung genügt.

## Goals / Non-Goals

**Goals:**
- `net_payout` ist immer ≥ `0,00 €`
- `fees_due = brutto − net_payout` ist dadurch immer ≤ `brutto` und immer ≥ `0,00 €`
- Alle Pfade (Kassiervorgang-Bericht, Veranstaltungs-Zusammenfassung, Auszahlungsberechnung) sind konsistent

**Non-Goals:**
- Keine Änderung der Rundungslogik für positive `theoretical_net`-Werte
- Keine Behandlung von negativem Bruttoumsatz (nicht möglich in der App)
- Die deprecated `calculate_fees()`-Methode wird nicht geändert (kein Produktionspfad)

## Decisions

### Fix direkt in `round_to_step` vs. in `calculate_payout`

**Entschieden: Fix in `calculate_payout()`** — der Floor bei 0,00 ist eine fachliche Regel der Auszahlungsberechnung, nicht eine Eigenschaft des allgemeinen Rundungsalgorithmus. `round_to_step()` könnte theoretisch auch für andere Zwecke genutzt werden, bei denen negative Ergebnisse korrekt sind.

Konkret: Eine Zeile nach der Rundung:
```rust
let net_payout = self.round_to_step(theoretical_net).max(Decimal::ZERO);
```

### Kein eigener Grenzfall-Zweig nötig

`Decimal::max()` ist ausreichend — kein `if theoretical_net < 0` erforderlich. Der Ausdruck ist minimal und lässt sich direkt in den bestehenden Einzeiler integrieren.

## Risks / Trade-offs

**Bestehende Tests könnten den negativen Wert als erwartet haben** → Alle Tests für `calculate_payout` und `calculate_fees` müssen geprüft und bei Bedarf korrigiert werden. Tests die `net_payout < 0` assertieren, sind inhaltlich falsch und werden korrigiert.

**`grenzfall_netto.json`-Testdatei** → Diese Testdatendatei in `docs/validation/testdata/` existiert bereits für genau diesen Grenzfall und kann nach dem Fix für den Importtest verwendet werden.

## Open Questions

*(keine)*
