use fastnum::D128;

#[derive(Debug, Clone, Copy, Eq, Ord, PartialOrd, PartialEq, Hash, Default)]
pub struct Amount<const DECIMALS: usize>(D128);

impl<const DECIMALS: usize> Amount<DECIMALS> {
    pub const ZERO: Self = Self::new_scaled_i32(0);

    /// Treats the input as a scaled integer (e.g. 1234 → 12.34)
    pub const fn new_scaled_i32(inner: i32) -> Self {
        Self(D128::from_i32(inner).div(D128::from_i32(10_i32).pow(D128::from_usize(DECIMALS))))
    }

    /// Treats the input as a scaled integer (e.g. 1234 → 12.34)
    pub const fn new_scaled_i64(inner: i64) -> Self {
        Self(D128::from_i64(inner).div(D128::from_i64(10_i64).pow(D128::from_usize(DECIMALS))))
    }

    pub const fn new_f64(inner: f64) -> Self {
        Self(D128::from_f64(inner))
    }

    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub const fn raw(&self) -> D128 {
        self.0
    }
}

impl<const DECIMALS: usize> From<i32> for Amount<DECIMALS> {
    fn from(value: i32) -> Self {
        Self::new_scaled_i32(value)
    }
}

impl<const DECIMALS: usize> From<i64> for Amount<DECIMALS> {
    fn from(value: i64) -> Self {
        Self::new_scaled_i64(value)
    }
}

impl<const DECIMALS: usize> From<Amount<DECIMALS>> for i32 {
    /// # Panics
    ///
    /// May panic if the underlying number is outside i32 bounds. This should be avoided
    /// but is there to ensure backwards-compatibility.
    fn from(value: Amount<DECIMALS>) -> Self {
        (value.0 * 10_i32.pow(DECIMALS as u32)).to_i32().unwrap()
    }
}

/// Semantic type to indicate the underlying value is in Euros and not [`Cents`].
type Euros = Amount<0>;

/// A monetary amount in cents (2 decimal places).
#[allow(dead_code)]
type Cents = Amount<2>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_example() {
        // monetary value 12.34, stored as cents
        // - stored as a scaled integer, where it is scaled by N of Amount<N>
        let amount: Cents = Amount::new_scaled_i32(1234);
        assert!(format!("{:?}", &amount).contains("D128(digits=[1234], exp=[-2]"));

        let value: i32 = amount.into();
        assert_eq!(value, 1234_i32);
    }

    #[test]
    fn convert_from_f64_using_new() {
        // Assume we have a whole currency unit, parsed from a CSV file, into a f64.  Since this is
        // a whole currency unit, we need to convert it to cents before we can use our semantic type.
        let provided = 1.23;
        // Notice in `new_scaled_i32` above this is scaled before storage, and we can see it is internally
        // stored with a scaling factor of N
        let converted: Cents = Amount::new_scaled_i32((provided * 100.00) as i32);
        assert!(format!("{:?}", &converted).contains("D128(digits=[123], exp=[-2]"));

        // converting from internal storage, this is scaled by N of Amount<N>
        let d: i32 = converted.into();
        assert_eq!(d, 123_i32);

        // Let's do the same but change our original value; assume we have 1 Euro:
        let provided = 1.00;
        let converted: Cents = Amount::new_scaled_i32((provided * 100.00) as i32);
        // Interestingly, this is stored as 1e0 (which is the same as 100e-2).
        assert!(format!("{:?}", &converted).contains("D128(digits=[1], exp=[0]"));

        // converting from internal storage, this is scaled by N of Amount<N>
        let d: i32 = converted.into();
        assert_eq!(d, 100_i32);
    }

    #[test]
    fn convert_from_i32() {
        // whole currency unit
        let provided = 100;
        let _: Cents = provided.into();
    }

    #[test]
    fn convert_from_i64() {
        // whole currency unit
        let provided = 100_i64;
        let _: Cents = provided.into();
    }
}
