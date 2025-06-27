use fastnum::D128;
use std::sync::LazyLock;

static DATA: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "548.15", "83.15", "805.28", "142.66", "107.19", "852.18", "50.29", "781.65", "887.29",
        "988.73",
    ]
});

pub mod amount;

fn get_total_d128(data: &[&'static str]) -> D128 {
    let mut accumulator = D128::from_f64(0.0);

    data.iter().for_each(|s| {
        let value: f64 = s.parse().unwrap();

        accumulator = accumulator.add(D128::from_f64(value));
    });

    accumulator
}

fn get_total_f64(data: &[&'static str]) -> f64 {
    let mut accumulator = 0.0;

    data.iter().for_each(|s| {
        let value: f64 = s.parse().unwrap();

        accumulator += value;
    });

    accumulator
}

#[cfg(test)]
mod tests {
    use crate::ex_3::{DATA, get_total_d128, get_total_f64};

    #[should_panic(expected = "assertion `left == right` failed")]
    #[test]
    fn demo_f64_precision_loss() {
        let data = DATA.clone();

        let r_d128 = get_total_d128(&data);
        let r_f64 = get_total_f64(&data);

        assert_eq!(r_d128.round(20).to_string(), format!("{:.20}", r_f64));

        // NOTE: is this a sufficient test to show f64 precision loss? What's a better way to illustrate this, i.e. a larger dataset?
        //
        // test ex_3::tests::demo_f64_precision_loss ...
        //     thread 'ex_3::tests::demo_f64_precision_loss' panicked at src/ex_3:50:9:
        //     assertion `left == right` failed
        // left: "5246.56999999999985817567"
        // right: "5246.56999999999970896170"
        // FAILED
    }
}
