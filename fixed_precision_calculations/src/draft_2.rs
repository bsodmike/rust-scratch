use fastnum::D128;
use std::fmt::Formatter;

pub struct CurrencyFormatter {}
impl CurrencyFormatter {
    pub fn new() -> Self {
        Self {}
    }

    /// Convert cents from [`f64`] to [`Euros`]
    pub fn format_cents(&self, cents_raw: f64) -> Euros {
        let r = D128::from(cents_raw) / D128::from(100);
        let r = r.round(2);

        r.into()
    }
}

/// Semantic type to indicate the underlying value is in Euros and not [`Cents`].
type Euros = Amount<0>;

/// A monetary amount in cents (2 decimal places).
#[allow(dead_code)]
type Cents = Amount<2>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Amount<const DECIMALS: usize>(D128);

impl<const DECIMALS: usize> Amount<DECIMALS> {
    /// Treats the input as a scaled integer (e.g. 1234 → 12.34)
    pub const fn new_scaled_i32(inner: i32) -> Self {
        Self(D128::from_i32(inner).div(D128::from_i32(10_i32).pow(D128::from_usize(DECIMALS))))
    }

    pub const fn new_f64(inner: f64) -> Self {
        Self(D128::from_f64(inner))
    }
}

impl<const DECIMALS: usize> std::fmt::Display for Amount<DECIMALS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const DECIMALS: usize> From<D128> for Amount<DECIMALS> {
    fn from(value: D128) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

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
        let euros = op(input);
        assert_eq!(euros.to_string(), expected);
    }
}
