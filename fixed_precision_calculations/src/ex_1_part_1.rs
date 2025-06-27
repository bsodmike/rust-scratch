use fastnum::D128;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, Default)]
pub struct Amount<const DECIMALS: usize>(D128);

impl<const DECIMALS: usize> Amount<DECIMALS> {
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
type Euros = Amount<0>;

/// A monetary amount in cents (2 decimal places).
#[allow(dead_code)]
type Cents = Amount<2>;

#[cfg(test)]
mod tests {
    use super::*;

    #[should_panic(expected = "assertion `left == right` failed")]
    #[test]
    fn simulate_rounding_failure() {
        let average: f64 = 56098.9;
        let r: D128 = D128::from(average) / D128::from(100);

        assert_eq!(r.to_string(), "560.99");

        // thread 'ex_1_part_1::tests::simulate_rounding_failure' panicked at src/ex_1_part_1.rs:44:9:
        //     assertion `left == right` failed
        // left: "560.98900000000001455191522836685180664"
        // right: "560.99"
    }

    #[test]
    fn simulate_rounding_failure_fixed() {
        let average: f64 = 56098.9;
        let r: D128 = D128::from(average) / D128::from(100);
        let r = r.round(2);

        assert_eq!(r.to_string(), "560.99");
    }

    #[should_panic(expected = "assertion `left == right` failed")]
    #[test]
    fn simulate_rounding_failure_converted() {
        let average: f64 = 56098.9;
        let r: D128 = D128::from(average) / D128::from(100);
        // let r = r.round(2);
        let euros: Euros = r.into();

        assert_eq!(euros.to_string(), "560.99");
        //
        // thread 'ex_1_part_1::tests::simulate_rounding_failure_converted' panicked at src/ex_1_part_1.rs:68:9:
        //     assertion `left == right` failed
        // left: "560.98900000000001455191522836685180664"
        // right: "560.99"
    }

    #[test]
    fn simulate_rounding_failure_converted_fixed() {
        let average: f64 = 56098.9;

        let r: D128 = D128::from(average) / D128::from(100);
        let euros: Euros = r.round(2).into();

        assert_eq!(euros.to_string(), "560.99");
    }
}
