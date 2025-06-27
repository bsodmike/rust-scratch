use anyhow::Context;
use fastnum::D128;
use std::fmt::Formatter;

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

#[derive(Debug, thiserror::Error)]
pub enum AmountConverterError {
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}
pub struct AmountConverter<const DECIMALS: usize> {
    amount: Amount<DECIMALS>,
}

impl<const DECIMALS: usize> AmountConverter<DECIMALS> {
    pub fn new(amount: Amount<DECIMALS>) -> Self {
        Self { amount }
    }

    pub fn amount(&self) -> Amount<DECIMALS> {
        self.amount
    }

    pub fn amount_to_i32(&self) -> Result<i32, AmountConverterError> {
        self.amount
            .0
            .to_i32()
            .map_err(|err| anyhow::anyhow!("error converting amount to i32: {:?}", err).into())
    }

    pub fn amount_to_f64(&self) -> f64 {
        self.amount.0.to_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[test]
    fn test_amount_converter_init() {
        // Using decimals = 2
        let value = Amount::new_scaled_i32(1234);
        let converter = AmountConverter::<2>::new(value);

        assert_eq!(converter.amount_to_i32().unwrap(), 12);
        assert_eq!(converter.amount_to_f64(), 12.34);

        // Using decimals = 0
        let value = Amount::new_scaled_i32(1234);
        let converter = AmountConverter::<0>::new(value);

        assert_eq!(converter.amount_to_i32().unwrap(), 1234);
        assert_eq!(converter.amount_to_f64(), 1234.00);
    }

    #[test]
    fn check_fractional_digits() {
        let average: f64 = 56098.9;
        let r = D128::from(average) / D128::from(100);
        assert_eq!(r.fractional_digits_count(), 35);

        let rounded = r.round(2);
        assert_eq!(rounded.fractional_digits_count(), 2);
    }
}
