use crate::domain::value_type::amount::Euros;
use fastnum::D128;

pub struct CurrencyFormatter {}
impl CurrencyFormatter {
    pub fn new() -> Self {
        Self {}
    }

    /// Convert cents from [`f64`] to [`Euros`]
    pub fn format_cents(&self, cents_raw: f64) -> Euros {
        let euros = D128::from(cents_raw) / D128::from(100);
        let rounded = euros.round(2);

        rounded.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_type::amount::{Amount, Cents};
    use rstest::rstest;

    #[test]
    fn create_cents_from_f64() {
        // This is 1 unit of currency
        let _value: Cents = Amount::<2>::new_f64(100.0);
    }

    #[should_panic(expected = "assertion `left == right` failed")]
    #[test]
    fn simulate_rounding_failure() {
        let average: f64 = 56098.9;
        let r = D128::from(average) / D128::from(100);
        let euros: Euros = r.into();

        assert_eq!(format!("{:.2}", euros), "560.99");

        // thread 'ex_1::tests::simulate_rounding_failure' panicked at src/ex_1.rs:37:9:
        //     assertion `left == right` failed
        // left: "560.98900000000001455191522836685180664"
        // right: "560.99"
    }

    #[rstest]
    #[case(|cents_raw| {
        let formatter = CurrencyFormatter::new();
        formatter.format_cents(cents_raw)
    }, 56097.26,"560.97")]
    #[case(|cents_raw| {
        let formatter = CurrencyFormatter::new();
        formatter.format_cents(cents_raw)
    }, 56099.9,"561.00")] // 560.999 → 561.00
    #[case(|cents_raw| {
        let formatter = CurrencyFormatter::new();
        formatter.format_cents(cents_raw)
    }, 56098.9,"560.99")] // 560.989 → 560.99
    fn test_formatted_from_cents(
        #[case] op: impl Fn(f64) -> Euros,
        #[case] input: f64,
        #[case] expected: String,
    ) {
        assert_eq!(format!("{:.2}", op(input)), expected);
    }
}
