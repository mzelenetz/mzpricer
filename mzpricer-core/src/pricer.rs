#[derive(Clone, Copy, Debug)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Clone, Copy, Debug)]
pub struct TimeDuration {
    pub value: f64,
    pub factor: f64, // this is total units in a year: 365, 252, etc.
}

#[derive(Clone, Copy, Debug)]
pub enum PriceError {
    None,
    NonConvergence,
    BadParams,
}


impl TimeDuration {
    fn new(value: f64, factor: f64) -> Self {
        TimeDuration { value, factor }
    }

    // Calculates the total time to expiration (T) in years.
    pub fn to_years(&self) -> f64 {
        self.value / self.factor
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StockPrice {
    pub spot_price: f64,
    pub dividend_amout: f64,
    pub time_to_dividend_days: f64,
    pub rate: f64,
}

impl StockPrice {
    const FACTOR: f64 = 365.0; // Default to 365 days in a year
    fn new(spot_price: f64, dividend_amout: f64, time_to_dividend_days: f64, rate: f64) -> Self {
        StockPrice { spot_price, dividend_amout, time_to_dividend_days, rate }
    }

    fn to_years(&self) -> f64 {
        self.time_to_dividend_days / Self::FACTOR
    }

    pub fn s_prime(&self) -> f64 {
        let t = self.to_years();
        let pv_div = self.dividend_amout * (-self.rate * t).exp();
        let s_prime = self.spot_price - pv_div;
        s_prime
    }
}


pub fn option_price_scalar(
    s: f64,
    k: f64,
    t: &TimeDuration,
    r: f64,
    sigma: f64,
    cp: OptionType,
    precision: usize,
) -> f64 {
    let delta_t = t.to_years() / precision as f64;
    option_price_(s, k, delta_t, r, sigma, cp, precision)
}

pub fn option_price_vector(
    s_vec: &[f64],
    k_vec: &[f64],
    t_vec: &[TimeDuration],
    r_vec: &[f64],
    sigma_vec: &[f64],
    cp_vec: &[OptionType],
    precision: usize,
) -> (Vec<f64>, Vec<PriceError>) {

    let n = s_vec.len();
    let mut prices = Vec::with_capacity(n);
    let mut errors = Vec::with_capacity(n);

    for i in 0..n {
        let delta_t = t_vec[i].to_years() / precision as f64;
        let price = option_price_(s_vec[i], k_vec[i], delta_t, r_vec[i], sigma_vec[i], cp_vec[i], precision);
        prices.push(price);
        errors.push(PriceError::None);
    }

    (prices, errors)
}

pub fn option_price_(s: f64, k: f64, delta_t: f64, r: f64, sigma: f64, cp: OptionType, n: usize) -> f64 {
    // Implementation of the binomial tree option pricing model for American options.
    // Separate the logic from the caller to allow for more flexible use of the function
    let (sign, base) = match cp {
        OptionType::Call => (1.0, -k),
        OptionType::Put  => (-1.0,  k),
    };

    // Calculate the up (u) and down (d) factors
    let u: f64 = (sigma * delta_t.sqrt()).exp();
    let d: f64 = 1.0/u;

    // Calculate the risk-neutral probability (p), this is used to dicount the expected payoff back to present value
    let a: f64 = (r * delta_t).exp();
    let p: f64 = (a - d) / (u - d);

    // Dicount Factor
    let df: f64 = (-r * delta_t).exp();

    // Initialize two arrays for the stock price and option price at each node
    let mut stock_price: Vec<f64> = vec![0.0; n + 1];
    let mut option_price: Vec<f64> = vec![0.0; n + 1];

    // 1. calculate all possible stock prices at maturity (time T) 
    stock_price[n] = s * u.powi(n as i32);
    for i in (0..n).rev() {
        stock_price[i] = stock_price[i + 1] * (d/u);
    }

    // 2. calculate the option price at maturity (time T) for each possible stock price
    for i in 0..=n {
        option_price[i] = (sign * stock_price[i] + base).max(0.0);
    }

    // 3. Check for early exercise 
    for i in (1..=n).rev() {
        for j in 0..i {
            // We are going through each time step at each node and calculating the option price at that node. 
            // Calculate the european value (ie. value if not exercised) 
            let continuation_value = (p * option_price[j + 1] + (1.0 - p) * option_price[j ]) * df;
            let time_step = i - 1;
            let num_down_moves = time_step - j;
            let s_i = s * u.powi(j as i32) * d.powi(num_down_moves as i32); // price at this node (time_step,j)
            let intrinsic_value = (sign * s_i + base).max(0.0);
            
            // Check EE. 
            // The value of the american is greater of the intrtinsic value and the continuation value
            option_price[j] = intrinsic_value.max(continuation_value);
        }
    };

    return option_price[0];
}

pub fn option_iv_scalar(
    price: f64,
    s: f64,
    k: f64,
    t: &TimeDuration,
    r: f64,
    candidate_sigma: f64,
    cp: OptionType,
    precision: usize,
) -> f64 {
    option_iv_(price, s, k, &t, r, candidate_sigma, cp, Some(precision))
}

pub fn option_iv_vector(
    price_vec: &[f64],
    s_vec: &[f64],
    k_vec: &[f64],
    t_vec: &[TimeDuration],
    r_vec: &[f64],
    sigma_vec: &[f64],
    cp_vec: &[OptionType],
    precision: usize,
) -> (Vec<f64>, Vec<PriceError>) {

    let n = s_vec.len();
    let mut ivs = Vec::with_capacity(n);
    let mut errors = Vec::with_capacity(n);

    for i in 0..n {
        let iv = option_iv_(price_vec[i], s_vec[i], k_vec[i], &t_vec[i], r_vec[i], sigma_vec[i], cp_vec[i], Some(precision));
        ivs.push(iv);
        errors.push(PriceError::None);
    }

    (ivs, errors)
}


pub fn option_iv_(price: f64, s: f64, k: f64, t: &TimeDuration, r: f64, mut candidate_sigma: f64, cp: OptionType, precision: Option<usize>) -> f64 {
  // This function calculates the implied volatility given the option price. 
  // Using the Newton-Raphson method 
  const MAX_ITER: usize = 100;
  const SENSITIVITY: f64 = 0.00001;
  const VEGA_BUMP: f64 = 0.001;
  let n: usize = precision.unwrap_or(500); // Default 
  let delta_t = t.to_years() / n as f64;

  let mut test_price = option_price_(s, k, delta_t, r, candidate_sigma, cp, n);

  for _ in 0..MAX_ITER {
    let error = test_price - price;
    if error.abs() < SENSITIVITY {
        return candidate_sigma;
    }
    // Update vega value
    let vega_value = vega_iv_finder(s, k, t, r, candidate_sigma, cp, n, VEGA_BUMP);

    // Safety check: Avoid division by zero
    if vega_value.abs() < 1e-10 {
        // TODO: Handle cases where vega is zero (e.g., deep ITM/OTM options)
        eprintln!("Warning: Vega too small. Aborting IV calculation.");
        return candidate_sigma; 
    }

    candidate_sigma = candidate_sigma - error / vega_value;
    test_price = option_price_(s, k, delta_t, r, candidate_sigma, cp,  n);

    }

    eprintln!("Warning: Did not converge after {} iterations. Returning best guess.", MAX_ITER);
    candidate_sigma
}


pub fn vega(s: f64, k: f64, t: &TimeDuration, r: f64, sigma: f64, cp: OptionType, n: usize, bump: f64) -> f64 {
  // Calcuate the vega
  let vega = vega_iv_finder(s, k, t, r, sigma, cp, n, bump);
  vega/100.0 // Return per 1% change in volatility
}

pub fn vega_iv_finder(s: f64, k: f64, t: &TimeDuration, r: f64, sigma: f64, cp: OptionType, n: usize, bump: f64) -> f64 {
  // Calcuate the vega
  let delta_t = t.to_years() / n as f64;
  let option_price_bump_up = option_price_(s, k, delta_t, r, sigma + bump, cp, n);
  let option_price_bump_down = option_price_(s, k, delta_t, r, sigma - bump, cp, n);
  let vega = (option_price_bump_up - option_price_bump_down) / (2.0 * bump);
  println!("Vega calculation: price up {}, price down {}, vega {}, delta_t {} bump {}", option_price_bump_up, option_price_bump_down, vega, delta_t, bump);
  vega
}

pub fn theta(
    s: f64,
    k: f64,
    t: &TimeDuration,
    r: f64,
    sigma: f64,
    cp: OptionType,
    precision: usize,
    price: Option<f64>, // Optional precomputed price
) -> f64 {
    let delta_t0 = t.to_years() / precision as f64;
    let delta_t1 = t.to_years() / precision as f64 + (1.0 / 365.0); // TODO: This is one calendar day later. I may want to make this more flexible

    let price_0 = match price {
        Some(p) => p,
        None => option_price_(s, k, delta_t0, r, sigma, cp, precision),
    };    
    let price_new = option_price_(s, k, delta_t1, r, sigma, cp, precision);
    let theta = (price_new - price_0) / (1.0 / 365.0);
    theta
}


pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
}

pub fn greeks(
    s_vec: &[f64],
    k_vec: &[f64],
    t_vec: &[TimeDuration],
    r_vec: &[f64],
    sigma_vec: &[f64],
    cp_vec: &[OptionType],
    precision: usize,
) -> (Vec<Greeks>, Vec<PriceError>) {
    let n = s_vec.len();
    let mut results = Vec::with_capacity(n);
    let mut errors = Vec::with_capacity(n);
    
    const S_BUMP: f64 = 0.001;
    const SIGMA_BUMP: f64 = 0.001;

    for i in 0..n {
        let s = s_vec[i];
        let k = k_vec[i];
        let t = &t_vec[i];
        let r = r_vec[i];
        let sigma = sigma_vec[i];
        let cp = cp_vec[i];
        
        let delta_t = t.to_years() / precision as f64;

        let price_0 = option_price_(s, k, delta_t, r, sigma, cp, precision);

        let price_up = option_price_(s + S_BUMP, k, delta_t, r, sigma, cp, precision);
        let price_down = option_price_(s - S_BUMP, k, delta_t, r, sigma, cp, precision);

        let delta_val = (price_up - price_down) / (2.0 * S_BUMP);

        let gamma_val = (price_up - 2.0 * price_0 + price_down) / (S_BUMP * S_BUMP);

        let vega_val = vega(s, k, t, r, sigma, cp, precision, SIGMA_BUMP);
        let theta_val = theta(s, k, t, r, sigma, cp, precision, Some(price_0));
        
        results.push(Greeks {
            delta: delta_val,
            gamma: gamma_val,
            vega: vega_val,
            theta: theta_val,
        });
        errors.push(PriceError::None);
    }

    (results, errors)
}
