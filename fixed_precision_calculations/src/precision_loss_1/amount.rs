use fastnum::D128;
use std::fmt::Formatter;

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

/// Semantic type to indicate the underlying value is in Euros and not [`Cents`].
#[allow(dead_code)]
pub type Euros = Amount<0>;

/// A monetary amount in cents (2 decimal places).
#[allow(dead_code)]
pub type Cents = Amount<2>;
