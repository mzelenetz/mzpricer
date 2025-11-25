use mzpricer_core::{
    OptionType,
    TimeDuration,
    PriceError,
    greeks,
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_and_vega() {
        let s = vec![100.0, 100.0];
        let k = vec![100.0, 100.0];
        let t = vec![
            TimeDuration { value: 365.0, factor: 365.0 },
            TimeDuration { value: 365.0, factor: 365.0 },
        ];
        let r = vec![0.05, 0.05];
        let sigma = vec![0.20, 0.20];
        let cp = vec![OptionType::Call, OptionType::Put];
        let precision = 2000; // High precision for better accuracy

        let (greeks_results, errors) = greeks(&s, &k, &t, &r, &sigma, &cp, precision);
        let call_greeks = &greeks_results[0];
        let put_greeks = &greeks_results[1];

        println!("Call Delta: {}", call_greeks.delta);
        println!("Put Delta: {}", put_greeks.delta);
        
        // Assertions:
        assert!(errors.iter().all(|e| matches!(e, PriceError::None)));

        // 1. Check Delta range: Call Delta should be (0, 1), Put Delta should be (-1, 0)
        assert!(call_greeks.delta > 0.0 && call_greeks.delta < 1.0);
        assert!(put_greeks.delta < 0.0 && put_greeks.delta > -1.0);

        // 2. Check Delta Parity: For American options, Call Delta is NOT exactly Put Delta + 1.0
        // We check that they are CLOSE, but the difference is NOT exactly 1.0.
        let delta_sum = call_greeks.delta - put_greeks.delta;
        
        // The theoretical Black-Scholes delta parity value is approx 1.0 (e^(-r*T) = e^(-0.05*1) = 0.9512)
        // Since we have no dividends, the parity should be close to 1.0
        assert!((delta_sum - 1.0).abs() > 0.001, 
                "Delta sum is too close to 1.0, suggesting European parity: {}", delta_sum);
        
        // For the provided numbers: 0.6273939891023872 - (-0.3976399276233167) = 1.0250339167257039
        // This value is > 1.0, which is a known possibility for American options, confirming the model works.

        // 3. Check for approximate equality between Call and Put Vega (as previously discussed)
        let vega_diff = (call_greeks.vega - put_greeks.vega).abs();
        assert!(vega_diff < 0.0005, "Call and Put Vegas differ by more than tolerance: {}", vega_diff);
    }
}