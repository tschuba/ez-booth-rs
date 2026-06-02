## ADDED Requirements

### Requirement: Netto-Auszahlung ist immer mindestens 0,00 €
Das System SHALL sicherstellen, dass die berechnete Netto-Auszahlung (`net_payout`) eines Verkäufers niemals negativ ist. Wenn `theoretical_net` nach Abzug von Teilnahmegebühr und Umsatzbeteiligung negativ ist, SHALL `net_payout = 0,00 €` gelten. Die Gebühren (`fees_due`) entsprechen in diesem Fall dem vollen Bruttoumsatz.

#### Scenario: Brutto kleiner als Teilnahmegebühr
- **WHEN** der Bruttoumsatz eines Verkäufers kleiner als die Teilnahmegebühr ist (z. B. Brutto 0,50 € bei TG 1,00 €)
- **THEN** ist `net_payout = 0,00 €`
- **THEN** ist `fees_due = brutto` (der gesamte Umsatz wird als Gebühr verbucht)
- **THEN** wird kein negativer Auszahlungsbetrag angezeigt

#### Scenario: Brutto gleich Teilnahmegebühr
- **WHEN** der Bruttoumsatz exakt der Teilnahmegebühr entspricht (z. B. Brutto 1,00 € bei TG 1,00 €)
- **THEN** ist `theoretical_net = 1,00 − 1,00 − 1,00 × 0,15 = −0,15`
- **THEN** ist `net_payout = 0,00 €` (Floor angewendet)
- **THEN** ist `fees_due = 1,00 €`

#### Scenario: Normaler Bruttoumsatz über Teilnahmegebühr
- **WHEN** der Bruttoumsatz deutlich über der Teilnahmegebühr liegt (z. B. Brutto 92,00 € bei TG 1,00 €, UA 15 %, RS 0,50 €)
- **THEN** ist `theoretical_net = 92,00 − 1,00 − 13,80 = 77,20`
- **THEN** ist `net_payout = 77,00 €` (kaufmännische Rundung auf 0,50 €)
- **THEN** ist `fees_due = 15,00 €`
- **THEN** ist der Floor irrelevant (theoretical_net ist positiv)

#### Scenario: Auszahlungsberechnung in der Veranstaltungs-Zusammenfassung
- **WHEN** die Veranstaltungs-Zusammenfassung für eine Veranstaltung mit einem Verkäufer mit Brutto ≤ TG angezeigt wird
- **THEN** zeigt die App `0,00 €` als Auszahlung für diesen Verkäufer
- **THEN** zeigt die App keinen negativen Betrag in der Spalte „Auszahlung"
