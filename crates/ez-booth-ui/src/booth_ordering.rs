use std::cmp::Ordering;

use domain::models::booth::Booth;

/// Apply consistent ordering for booth lists:
/// 1. Open booths before closed booths
/// 2. Within each status group, date descending (newest first)
/// 3. Tie-breaker: description ascending (case-insensitive)
pub fn sort_booths(booths: &mut [Booth]) {
    booths.sort_by(compare_booths);
}

fn compare_booths(a: &Booth, b: &Booth) -> Ordering {
    match (a.is_open(), b.is_open()) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => {}
    }

    match b.date.cmp(&a.date) {
        Ordering::Equal => {}
        other => return other,
    }

    let a_desc = a.description.to_lowercase();
    let b_desc = b.description.to_lowercase();
    a_desc.cmp(&b_desc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Utc};
    use domain::models::booth::{Booth, BoothStatus, FeeConfig};
    use rust_decimal::Decimal;

    fn test_booth(desc: &str, date: &str, status: BoothStatus) -> Booth {
        let fees = FeeConfig {
            participation_fee: Decimal::ONE,
            sales_fee_percent: Decimal::ONE,
            rounding_step: Decimal::ONE,
        };

        let mut booth = Booth::new(
            desc.to_string(),
            NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            fees,
        )
        .unwrap();

        booth.status = status;
        booth
    }

    #[test]
    fn sorts_by_status_date_and_description() {
        let mut booths = vec![
            test_booth(
                "Alpha",
                "2026-03-25",
                BoothStatus::Closed {
                    closed_at: Utc::now(),
                },
            ),
            test_booth("Beta", "2026-03-26", BoothStatus::Open),
            test_booth("Gamma", "2026-03-24", BoothStatus::Open),
            test_booth("beta", "2026-03-26", BoothStatus::Open),
            test_booth(
                "Omega",
                "2026-03-28",
                BoothStatus::Closed {
                    closed_at: Utc::now(),
                },
            ),
        ];

        sort_booths(&mut booths);

        let ordered: Vec<_> = booths.into_iter().map(|b| b.description).collect();
        assert_eq!(ordered, vec!["Beta", "beta", "Gamma", "Omega", "Alpha"]);
    }
}
