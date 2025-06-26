use fastnum::D128;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, Eq, Ord, PartialOrd, PartialEq, Hash, Default)]
pub struct Amount<const DECIMALS: usize>(D128);

impl<const DECIMALS: usize> Amount<DECIMALS> {
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

impl<const DECIMALS: usize> From<D128> for Amount<DECIMALS> {
    fn from(value: D128) -> Self {
        Self(value)
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

impl<const DECIMALS: usize> From<Amount<DECIMALS>> for i64 {
    /// # Panics
    ///
    /// May panic if the underlying number is outside i64 bounds. This should be avoided
    /// but is there to ensure backwards-compatibility.
    fn from(value: Amount<DECIMALS>) -> Self {
        (value.0 * 10_i64.pow(DECIMALS as u32)).to_i64().unwrap()
    }
}

impl<const DECIMALS: usize> std::ops::Neg for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl<const DECIMALS: usize> std::ops::Add for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<const DECIMALS: usize> std::ops::Sub for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<const DECIMALS: usize> std::ops::Div for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    /// Divide two same-decimals amounts while keeping the same number of decimals
    fn div(self, rhs: Self) -> Self::Output {
        assert!(!rhs.0.is_zero(), "Attempt to divide Amount by zero");
        Self(self.0 / rhs.0)
    }
}

impl<const DECIMALS: usize> std::ops::Mul for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    /// Multiply two same-decimals amounts while keeping the same number of decimals
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl<const DECIMALS: usize> From<Amount<DECIMALS>> for f64 {
    fn from(value: Amount<DECIMALS>) -> Self {
        value.0.to_f64()
    }
}

impl<const DECIMALS: usize> From<f64> for Amount<DECIMALS> {
    fn from(value: f64) -> Self {
        Self::new_f64(value)
    }
}

impl<const DECIMALS: usize> std::ops::AddAssign for Amount<DECIMALS> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<const DECIMALS: usize> std::ops::SubAssign for Amount<DECIMALS> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<const DECIMALS: usize> std::ops::Mul<i32> for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl<const DECIMALS: usize> std::ops::Div<i32> for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    fn div(self, rhs: i32) -> Self::Output {
        assert_ne!(rhs, 0, "Attempt to divide Amount by zero");
        Self(self.0 / rhs)
    }
}

impl<const DECIMALS: usize> std::ops::Mul<f64> for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl<const DECIMALS: usize> std::ops::Div<f64> for Amount<DECIMALS> {
    type Output = Amount<DECIMALS>;

    fn div(self, rhs: f64) -> Self::Output {
        assert_ne!(rhs, 0_f64, "Attempt to divide Amount by zero");
        Self(self.0 / rhs)
    }
}

impl<const DECIMALS: usize> std::fmt::Display for Amount<DECIMALS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Semantic type to indicate the underlying value is in Euros and not [`Cents`].
pub type Euros = Amount<0>;

/// A monetary amount in cents (2 decimal places).
///
/// Semantic type to indicate the underlying value has 2 decimal places, where we need to convey the underlying value are in fact cents, where 1 unit = 1 cent (0.01), of a particular currency.
///
/// This does not relate to precision, which confuses AI helpers.  Looking at you @coderabbitai O_O
pub type Cents = Amount<2>;

#[cfg(test)]
mod tests {
    use super::Amount;
    use fastnum::decimal::{Context, Sign};
    use fastnum::{D128, u128};
    use rstest::rstest;

    const F64_THRESHOLD: f64 = 0.001_f64;

    #[rstest]
    #[case(1234, Amount(D128::from_parts(u128!(1234), -2, Sign::Plus, Context::default())))]
    #[case(-4729, Amount(D128::from_parts(u128!(4729), -2, Sign::Minus, Context::default())))]
    #[case(0, Amount(D128::from_parts(u128!(0), -2, Sign::Plus, Context::default())))]
    fn convert_i32_to_amount2(#[case] input: i32, #[case] expected: Amount<2>) {
        let amount: Amount<2> = input.into();
        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case(1234, Amount(D128::from_parts(u128!(1234), -2, Sign::Plus, Context::default())))]
    #[case(-4729, Amount(D128::from_parts(u128!(4729), -2, Sign::Minus, Context::default())))]
    #[case(0, Amount(D128::from_parts(u128!(0), -2, Sign::Plus, Context::default())))]
    fn convert_i64_to_amount2(#[case] input: i64, #[case] expected: Amount<2>) {
        let amount: Amount<2> = input.into();
        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case(12.34_f64, Amount(D128::from_parts(u128!(1234), -2, Sign::Plus, Context::default())))]
    #[case(-47.29_f64, Amount(D128::from_parts(u128!(4729), -2, Sign::Minus, Context::default())))]
    #[case(0_f64, Amount(D128::from_parts(u128!(0), -2, Sign::Plus, Context::default())))]
    #[case(-0_f64, Amount(D128::from_parts(u128!(0), -2, Sign::Minus, Context::default())))]
    fn convert_f64_to_amount2(#[case] input: f64, #[case] expected: Amount<2>) {
        let amount: Amount<2> = input.into();
        let diff = (amount - expected).0.abs().to_f64();

        assert!(
            diff < F64_THRESHOLD,
            "{amount} - {expected} ({}) to be lesser than {F64_THRESHOLD}",
            diff
        );
    }

    #[rstest]
    #[case(Amount::new_scaled_i32(1234), 1234)]
    #[case(Amount::new_scaled_i32(-4729), -4729)]
    #[case(Amount::new_scaled_i32(0), 0)]
    fn convert_amount2_to_i32(#[case] input: Amount<2>, #[case] expected: i32) {
        let amount: i32 = input.into();
        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case(Amount::new_scaled_i32(1234), 1234)]
    #[case(Amount::new_scaled_i32(-4729), -4729)]
    #[case(Amount::new_scaled_i32(0), 0)]
    fn convert_amount2_to_i64(#[case] input: Amount<2>, #[case] expected: i64) {
        let amount: i64 = input.into();
        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case(Amount::new_f64(12.34_f64), 12.34_f64)]
    #[case(Amount::new_f64(-47.29_f64), -47.29_f64)]
    #[case(Amount::new_f64(0_f64), 0_f64)]
    #[case(Amount::new_f64(-0_f64), -0_f64)]
    fn convert_amount2_to_f64(#[case] input: Amount<2>, #[case] expected: f64) {
        let amount: f64 = input.into();
        let diff = (amount - expected).abs();

        assert!(
            diff < F64_THRESHOLD,
            "{amount} - {expected} ({}) to be lesser than {F64_THRESHOLD}",
            diff
        );
    }

    #[rstest]
    #[case(Amount::new_scaled_i32(10000), 2, Amount::new_scaled_i32(20000))]
    #[case(Amount::new_scaled_i32(10000), -2, Amount::new_scaled_i32(-20000))]
    #[case(Amount::new_scaled_i32(-120), 6, Amount::new_scaled_i32(-720))]
    fn mul_amount2_with_i32(#[case] lhs: Amount<2>, #[case] rhs: i32, #[case] expected: Amount<2>) {
        let amount = lhs * rhs;

        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case(Amount::new_scaled_i32(10000), 0.20_f64, Amount::new_scaled_i32(2000))]
    #[case(Amount::new_scaled_i32(-2500), 0.10_f64, Amount::new_scaled_i32(-250))]
    #[case(Amount::new_scaled_i32(450), 0_f64, Amount::new_scaled_i32(0))]
    #[case(Amount::new_scaled_i32(0), 2_f64, Amount::new_scaled_i32(0))]
    fn mul_amount2_with_f64(#[case] lhs: Amount<2>, #[case] rhs: f64, #[case] expected: Amount<2>) {
        let amount = lhs * rhs;
        let diff = (amount - expected).0.abs().to_f64();

        assert!(
            diff < F64_THRESHOLD,
            "{amount} - {expected} ({}) to be lesser than {F64_THRESHOLD}",
            diff
        );
    }

    #[rstest]
    #[case(Amount::new_scaled_i32(10000), 2, Amount::new_scaled_i32(5000))]
    #[case(Amount::new_scaled_i32(10000), -2, Amount::new_scaled_i32(-5000))]
    #[case(Amount::new_scaled_i32(-1200), 6, Amount::new_scaled_i32(-200))]
    #[should_panic(expected = "Attempt to divide Amount by zero")]
    #[case::panic(Amount::new_scaled_i32(2), 0, Amount::new_scaled_i32(0))]
    fn div_amount2_with_i32(#[case] lhs: Amount<2>, #[case] rhs: i32, #[case] expected: Amount<2>) {
        let amount = lhs / rhs;

        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case::no_panic(Amount::new_scaled_i32(10000), 0.20_f64, Amount::new_scaled_i32(50000))]
    #[case::no_panic(Amount::new_scaled_i32(-2500), 0.10_f64, Amount::new_scaled_i32(-25000))]
    #[case::no_panic(Amount::new_scaled_i32(0), 2_f64, Amount::new_scaled_i32(0))]
    #[should_panic(expected = "Attempt to divide Amount by zero")]
    #[case::panic(Amount::new_scaled_i32(2), 0_f64, Amount::new_scaled_i32(0))]
    fn div_amount2_with_f64(#[case] lhs: Amount<2>, #[case] rhs: f64, #[case] expected: Amount<2>) {
        let amount = lhs / rhs;
        let diff = (amount - expected).0.abs().to_f64();

        assert!(
            diff < F64_THRESHOLD,
            "{amount} - {expected} ({}) to be lesser than {F64_THRESHOLD}",
            diff
        );
    }

    #[rstest]
    #[case(Amount::new_scaled_i32(10000), Amount::new_scaled_i32(-10000))]
    #[case(Amount::new_scaled_i32(-2500), Amount::new_scaled_i32(2500))]
    #[case(Amount::new_scaled_i32(0), Amount::new_f64(-0_f64))]
    #[case(Amount::new_f64(-0_f64), Amount::new_scaled_i32(0))]
    fn neg_amount2(#[case] input: Amount<2>, #[case] expected: Amount<2>) {
        let amount = -input;

        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case(
        Amount::new_scaled_i32(10000),
        Amount::new_scaled_i32(250),
        Amount::new_scaled_i32(10250)
    )]
    #[case(Amount::new_scaled_i32(10000), Amount::new_scaled_i32(-250), Amount::new_scaled_i32(9750))]
    #[case(Amount::new_scaled_i32(-120), Amount::new_scaled_i32(650), Amount::new_scaled_i32(530))]
    fn add_amount2(#[case] lhs: Amount<2>, #[case] rhs: Amount<2>, #[case] expected: Amount<2>) {
        let amount = lhs + rhs;

        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case(
        Amount::new_scaled_i32(10000),
        Amount::new_scaled_i32(250),
        Amount::new_scaled_i32(9750)
    )]
    #[case(Amount::new_scaled_i32(10000), Amount::new_scaled_i32(-250), Amount::new_scaled_i32(10250))]
    #[case(Amount::new_scaled_i32(-120), Amount::new_scaled_i32(650), Amount::new_scaled_i32(-770))]
    fn sub_amount2(#[case] lhs: Amount<2>, #[case] rhs: Amount<2>, #[case] expected: Amount<2>) {
        let amount = lhs - rhs;

        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case(
        Amount::new_scaled_i32(10000),
        Amount::new_scaled_i32(250),
        Amount::new_scaled_i32(25000)
    )]
    #[case(Amount::new_scaled_i32(10000), Amount::new_scaled_i32(-250), Amount::new_scaled_i32(-25000))]
    #[case(Amount::new_scaled_i32(-120), Amount::new_scaled_i32(650), Amount::new_scaled_i32(-780))]
    fn mul_amount2(#[case] lhs: Amount<2>, #[case] rhs: Amount<2>, #[case] expected: Amount<2>) {
        let amount = lhs * rhs;

        assert_eq!(amount, expected);
    }

    #[rstest]
    #[case::no_panic(
        Amount::new_scaled_i32(10000),
        Amount::new_scaled_i32(250),
        Amount::new_scaled_i32(4000)
    )]
    #[case::no_panic(Amount::new_scaled_i32(10000), Amount::new_scaled_i32(-250), Amount::new_scaled_i32(-4000))]
    #[case::no_panic(Amount::new_scaled_i32(-120000), Amount::new_scaled_i32(60000), Amount::new_scaled_i32(-200))]
    #[case::no_panic(
        Amount::new_scaled_i32(0),
        Amount::new_scaled_i32(500),
        Amount::new_scaled_i32(0)
    )]
    #[should_panic(expected = "Attempt to divide Amount by zero")]
    #[case::panic(
        Amount::new_scaled_i32(500),
        Amount::new_scaled_i32(0),
        Amount::new_scaled_i32(0)
    )]
    fn div_amount2(#[case] lhs: Amount<2>, #[case] rhs: Amount<2>, #[case] expected: Amount<2>) {
        let amount = lhs / rhs;

        assert_eq!(amount, expected);
    }
}
