## 1. Bugfix — Floor bei 0,00 € in calculate_payout

- [ ] 1.1 In `crates/domain/src/services/dto.rs`, Methode `calculate_payout()`: Die Zeile `let net_payout = self.round_to_step(theoretical_net);` ersetzen durch `let net_payout = self.round_to_step(theoretical_net).max(Decimal::ZERO);`

## 2. Tests ergänzen

- [ ] 2.1 In `crates/domain/src/services/dto.rs` im `mod tests`-Block: neuen Test `test_net_floor_at_zero` hinzufügen
- [ ] 2.2 Test prüft V8-Grenzfall: Brutto 0,50 €, TG 1,00 €, UA 15 %, RS 0,50 € → `net_payout = 0,00 €`, `fees_due = 0,50 €`
- [ ] 2.3 Test prüft V7-Grenzfall: Brutto 1,00 €, TG 1,00 €, UA 15 %, RS 0,50 € → `net_payout = 0,00 €`, `fees_due = 1,00 €`
- [ ] 2.4 Test prüft positives theoretical_net ist unverändert (Regression): Brutto 92,00 €, TG 1,00 €, UA 15 %, RS 0,50 € → `net_payout = 77,00 €`

## 3. Verifikation

- [ ] 3.1 `cargo test -p domain` — alle Tests grün
- [ ] 3.2 `cargo build --target wasm32-unknown-unknown` — Kompilierung ohne Fehler
- [ ] 3.3 App starten, `geraet_b_vor_merge.json` importieren → V8 zeigt `Auszahlung 0,00 €`, `Gebühren 0,50 €`
- [ ] 3.4 `geraet_a_vor_merge.json` importieren → V7 zeigt `Auszahlung 0,00 €`, `Gebühren 1,00 €`
