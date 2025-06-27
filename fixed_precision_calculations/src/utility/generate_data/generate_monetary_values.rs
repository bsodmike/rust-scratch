use rand::{Rng, rng};

/// Generate a vector of monetary values as strings like "12.02"
/// Values are in the range 1.00 to 1000.00
pub fn generate_fake_monetary_values(count: usize) -> Vec<String> {
    let mut rng = rng();
    let mut values = Vec::with_capacity(count);

    for _ in 0..count {
        let amount_cents = rng.random_range(100..=100_000); // 100 cents to 100000 cents
        let amount = amount_cents as f64 / 100.0;
        values.push(format!("{:.2}", amount));
    }

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_generate_fake_monetary_values() {
        let values = generate_fake_monetary_values(10);

        for value in &values {
            assert!(
                value.contains('.'),
                "Value '{}' should have decimal point",
                value
            );

            let parsed: f64 = value.parse().unwrap();
            assert!(
                parsed >= 1.00 && parsed <= 1000.00,
                "Value '{}' should be between 1.00 and 1000.00",
                value
            );

            assert_eq!(
                value.split('.').nth(1).unwrap().len(),
                2,
                "Value '{}' should have 2 decimal places",
                value
            );
        }

        assert!(!values.is_empty());
        // dbg!(&values);
    }
}
