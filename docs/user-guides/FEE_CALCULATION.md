---
title: Fee Calculation
nav_order: 1
parent: User Guides
---

# Gebührenberechnung / Fee Calculation

Diese Dokumentation erklärt die Gebührenlogik je Gebührenmodell inklusive Rundung und Beispielrechnungen.  
*This document explains fee logic per strategy, including rounding and worked examples.*

---

## Überblick / Overview

**Deutsch:**  
Die App berechnet pro Verkäufer eine **Netto-Abrechnung** (*Net Settlement*).  
Die Rundung wird auf die Netto-Abrechnung angewendet.

- **Netto-Abrechnung $\ge 0$**: Auszahlung an Verkäufer
- **Netto-Abrechnung $< 0$**: Vom Verkäufer geschuldeter Betrag

**English:**  
The app calculates a **Net Settlement** per vendor.  
Rounding is applied to net settlement.

- **Net Settlement $\ge 0$**: amount paid to vendor
- **Net Settlement $< 0$**: amount owed by vendor

---

## Gebührenbestandteile / Fee Components

1. **Teilnahmegebühr / Participation Fee (TG)**: fester Betrag  
2. **Umsatzbeteiligung / Sales Fee (UB)**: Prozentsatz auf Bruttoumsatz

Konfiguration kommt aus der Veranstaltung:
- Teilnahmegebühr
- Umsatzbeteiligung in %
- Rundungsschritt

---

## Gebührenmodelle / Fee Strategies

### 1) `sales_fee_first` (Default)

**Deutsch:** UB wird immer berücksichtigt. TG nur, wenn nach UB genug verbleibt:  
$VU - UB_{threshold} > TG$

**English:** Sales fee is always considered. Participation fee is charged only if enough remains after UB:  
$VU - UB_{threshold} > TG$

### 2) `both_fees_if_profitable`

**Deutsch:** Beide Gebühren nur, wenn zusammen kleiner als Umsatz:  
$TG + UB_{threshold} < VU$  
Sonst: keine Gebühren.

**English:** Both fees only if their sum is below gross sales:  
$TG + UB_{threshold} < VU$  
Otherwise: no fees.

### 3) `both_fees`

**Deutsch:** Beide Gebühren immer. Netto kann negativ werden.  
**English:** Both fees always apply. Net can become negative.

---

## Rundung / Rounding

### Begriffe / Terms

- $UB_{raw} = VU \times \frac{UB\%}{100}$
- $UB_{threshold} = round\_to\_step(UB_{raw})$ (für Schwellwertprüfung)
- $Netto = round\_to\_step(VU - TG^\* - UB_{raw})$

$TG^\*$ ist je Modell entweder TG oder 0.

### Regel / Rule

- Bei Rundungsschritt `0.00`: kaufmännisch auf 2 Dezimalstellen
- Sonst: kaufmännisch auf Vielfache des Rundungsschritts

---

## Strategie-Beispiele (Szenario 1) / Strategy Examples (Scenario 1)

**Parameter:**  
- TG = **1.10 €**
- UB = **10 %**
- Rundungsschritt = **0.10 €**

### `sales_fee_first`

- **A) VU = 1.20 €**  
  $UB_{threshold}=0.10$; $1.20-0.10 = 1.10 \nleqslant 1.10$ (nicht größer) → TG entfällt  
  Netto = **1.10 €**

- **B) VU = 2.00 €**  
  $UB_{threshold}=0.20$; $2.00-0.20 = 1.80 > 1.10$ → TG aktiv  
  Netto = **0.70 €**

### `both_fees_if_profitable`

- **A) VU = 1.20 €**  
  $TG + UB_{threshold} = 1.10 + 0.10 = 1.20$ (nicht kleiner) → keine Gebühren  
  Netto = **1.20 €**

- **B) VU = 2.00 €**  
  $1.10 + 0.20 = 1.30 < 2.00$ → beide Gebühren aktiv  
  Netto = **0.70 €**

### `both_fees`

- **A) VU = 1.20 €**  
  Netto = **0.00 €**

- **B) VU = 1.00 €**  
  Netto = **-0.20 €**

### Vergleich (VU = 1.20 €)

| Strategie | Netto |
|---|---:|
| `sales_fee_first` | 1.10 € |
| `both_fees_if_profitable` | 1.20 € |
| `both_fees` | 0.00 € |

---

## Strategie-Beispiele (Szenario 2) / Strategy Examples (Scenario 2)

**Parameter:**  
- TG = **1.10 €**
- UB = **10 %**
- Rundungsschritt = **0.50 €**

### `sales_fee_first`

- **A) VU = 1.00 €**  
  $UB_{threshold}=0.00$; $1.00-0.00 \le 1.10$ → TG entfällt  
  Netto = **1.00 €** (UB wirkt durch Rundung effektiv 0)

- **B) VU = 5.00 €**  
  $UB_{threshold}=0.50$; $5.00-0.50 > 1.10$ → TG aktiv  
  Netto = **3.50 €**

### `both_fees_if_profitable`

- **A) VU = 1.00 €**  
  $1.10 + 0.00 \not< 1.00$ → keine Gebühren  
  Netto = **1.00 €**

- **B) VU = 5.00 €**  
  $1.10 + 0.50 < 5.00$ → beide Gebühren aktiv  
  Netto = **3.50 €**

### `both_fees`

- **A) VU = 1.00 €**  
  Netto = **0.00 €**

- **B) VU = 5.00 €**  
  Netto = **3.50 €**

### Vergleich (VU = 1.00 €)

| Strategie | Netto |
|---|---:|
| `sales_fee_first` | 1.00 € |
| `both_fees_if_profitable` | 1.00 € |
| `both_fees` | 0.00 € |

---

## Technische Referenz / Technical Reference

Berechnung in: `crates/domain/src/services/dto.rs`  
Methode: `ChargingConfig::calculate_payout()`

---

**Letzte Aktualisierung / Last Updated:** April 2026