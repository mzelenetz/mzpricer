use mzpricer_core::{
    OptionType,
    TimeDuration,
    PriceError,
    option_price_scalar,
    option_price_vector,
    option_iv_scalar,
    option_iv_vector,
};

// ----------- simple sanity test ------------
#[test]
fn test_vector_pricing() {
    let s = vec![100.0, 100.0];
    let k = vec![100.0, 100.0];
    let t = vec![
        TimeDuration { value: 365.0, factor: 365.0 },
        TimeDuration { value: 365.0, factor: 365.0 },
    ];
    let r = vec![0.05, 0.05];
    let sigma = vec![0.20, 0.20];
    let cp = vec![OptionType::Call, OptionType::Put];
    let precision = 500;

    let (prices, errors) =
        option_price_vector(&s, &k, &t, &r, &sigma, &cp, precision);

    assert!(errors.iter().all(|e| matches!(e, PriceError::None)));

    println!("Prices: {:?}", prices);
    assert!((prices[0] - 10.44658514).abs() < 0.10);
    assert!((prices[1] - 6.088810110703188).abs() < 0.10);
}

#[test]
fn test_vector_iv() {
    let s = vec![100.0, 100.0];
    let k = vec![100.0, 110.0];
    let t = vec![
        TimeDuration { value: 365.0, factor: 365.0 },
        TimeDuration { value: 365.0, factor: 365.0 },
    ];
    let r = vec![0.05, 0.05];
    let sigma = vec![0.20, 0.20];
    let cp = vec![OptionType::Call, OptionType::Put];
    let precision = 500;

    let (px, _) = option_price_vector(&s, &k, &t, &r, &sigma, &cp, precision);

    let guess = vec![0.40, 0.40];

    let (ivs, _) =
        option_iv_vector(&px, &s, &k, &t, &r, &guess, &cp, precision);

    assert!((ivs[0] - 0.20).abs() < 0.002);
}
