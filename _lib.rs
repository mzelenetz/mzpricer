use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub enum OptionType {
    Call,
    Put,
}

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub struct TimeDuration {
    #[pyo3(get)]
    pub value: f64,
    #[pyo3(get)]
    pub factor: f64, // The number of time units in a year (e.g., 365, 252, 1638).
}

#[derive(Clone, Copy, Debug)]
pub enum PriceError {
    None,
    NonConvergence,
    BadParams,
}


impl IntoPy<PyObject> for PriceError {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            PriceError::None           => "None".into_py(py),
            PriceError::NonConvergence => "NonConvergence".into_py(py),
            PriceError::BadParams      => "BadParams".into_py(py),
        }
    }
}

#[pymethods]
impl TimeDuration {
    #[new]
    fn new(value: f64, factor: f64) -> Self {
        TimeDuration { value, factor }
    }

    /// Calculates the total time to expiration (T) in years.
    pub fn to_years(&self) -> f64 {
        self.value / self.factor
    }
}

#[pyclass]
#[derive(Clone, Copy, Debug)]
pub struct StockPrice {
    #[pyo3(get)]
    pub spot_price: f64,
    #[pyo3(get)]
    pub dividend_amout: f64,
    #[pyo3(get)]
    pub time_to_dividend_days: f64,
    #[pyo3(get)]
    pub rate: f64,
}

#[pymethods]
impl StockPrice {
    const FACTOR: f64 = 365.0; // Default to 365 days in a year
    #[new]
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

#[pyfunction]
fn option_price(
    s: &PyAny,
    k: &PyAny,
    t: &PyAny,
    r: &PyAny,
    sigma: &PyAny,
    cp: &PyAny,
    precision: Option<usize>,
) -> PyResult<PyObject> {

    let prec = precision.unwrap_or(500);

    // scalar input ------------------------------------------------
    if let (Ok(s), Ok(k), Ok(t), Ok(r), Ok(sigma), Ok(cp)) = (
        s.extract::<f64>(),
        k.extract::<f64>(),
        t.extract::<TimeDuration>(),
        r.extract::<f64>(),
        sigma.extract::<f64>(),
        cp.extract::<OptionType>(),
    ) {
        let price = option_price_scalar(s, k, &t, r, sigma, cp, prec);
        return Python::with_gil(|py| {
            Ok(price.into_py(py))
        });
    }
    
    // vector input ------------------------------------------------
    let s_vec    = s.extract::<Vec<f64>>()?;
    let k_vec    = k.extract::<Vec<f64>>()?;
    let t_vec    = t.extract::<Vec<TimeDuration>>()?;
    let r_vec    = r.extract::<Vec<f64>>()?;
    let sigma_vec= sigma.extract::<Vec<f64>>()?;
    let cp_vec   = cp.extract::<Vec<OptionType>>()?;

    let (prices, errors) =
        option_price_vector(s_vec, k_vec, t_vec, r_vec, sigma_vec, cp_vec, prec);

    Python::with_gil(|py| {
        Ok((prices, errors).into_py(py))
    })

}

fn option_price_scalar(
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

fn option_price_vector(
    s_vec: Vec<f64>,
    k_vec: Vec<f64>,
    t_vec: Vec<TimeDuration>,
    r_vec: Vec<f64>,
    sigma_vec: Vec<f64>,
    cp_vec: Vec<OptionType>,
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

fn option_price_(s: f64, k: f64, delta_t: f64, r: f64, sigma: f64, cp: OptionType, n: usize) -> f64 {
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

#[pyfunction]
fn vega(s: f64, k: f64, t: &TimeDuration, r: f64, sigma: f64, cp: OptionType, n: usize, bump: f64) -> f64 {
  // Calcuate the vega
  let delta_t = t.to_years() / n as f64;
  let option_price_bump_up = option_price_(s, k, delta_t, r, sigma + bump, cp, n);
  let option_price_bump_down = option_price_(s, k, delta_t, r, sigma - bump, cp, n);
  let vega = (option_price_bump_up - option_price_bump_down) / (2.0 * bump);
  vega
}

#[pyfunction]
fn option_iv(
    price: &PyAny,
    s: &PyAny,
    k: &PyAny,
    t: &PyAny,
    r: &PyAny,
    candidate_sigma: &PyAny,
    cp: &PyAny,
    precision: Option<usize>,
) -> PyResult<PyObject> {

    let prec = precision.unwrap_or(500);

    // scalar input ------------------------------------------------
    if let (Ok(price), Ok(s), Ok(k), Ok(t), Ok(r), Ok(candidate_sigma), Ok(cp)) = (
        price.extract::<f64>(),
        s.extract::<f64>(),
        k.extract::<f64>(),
        t.extract::<TimeDuration>(),
        r.extract::<f64>(),
        candidate_sigma.extract::<f64>(),
        cp.extract::<OptionType>(),
    ) {
        let iv = option_iv_scalar(price, s, k, &t, r, candidate_sigma, cp, prec);
        return Python::with_gil(|py| {
            Ok(iv.into_py(py))
        });
    }
    
    // vector input ------------------------------------------------
    let price_vec = price.extract::<Vec<f64>>()?;
    let s_vec    = s.extract::<Vec<f64>>()?;
    let k_vec    = k.extract::<Vec<f64>>()?;
    let t_vec    = t.extract::<Vec<TimeDuration>>()?;
    let r_vec    = r.extract::<Vec<f64>>()?;
    let sigma_vec= candidate_sigma.extract::<Vec<f64>>()?;
    let cp_vec   = cp.extract::<Vec<OptionType>>()?;

    let (ivs, errors) =
        option_iv_vector(price_vec, s_vec, k_vec, t_vec, r_vec, sigma_vec, cp_vec, prec);

    Python::with_gil(|py| {
        Ok((ivs, errors).into_py(py))
    })

}

fn option_iv_scalar(
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

fn option_iv_vector(
    price_vec: Vec<f64>,
    s_vec: Vec<f64>,
    k_vec: Vec<f64>,
    t_vec: Vec<TimeDuration>,
    r_vec: Vec<f64>,
    sigma_vec: Vec<f64>,
    cp_vec: Vec<OptionType>,
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


fn option_iv_(price: f64, s: f64, k: f64, t: &TimeDuration, r: f64, mut candidate_sigma: f64, cp: OptionType, precision: Option<usize>) -> f64 {
  // This function calculates the implied volatility given the option price. 
  // Using the Newton-Raphson method 
  const MAX_ITER: usize = 100;
  const SENSITIVITY: f64 = 0.00001;
  const VEGA_BUMP: f64 = 0.001;
  let n: usize = precision.unwrap_or(500); // Default 
  let delta_t = t.to_years() / n as f64;

  let mut test_price = option_price_(s, k, delta_t, r, candidate_sigma, cp, n);

  for i in 0..MAX_ITER {
    let error = test_price - price;
    if error.abs() < SENSITIVITY {
        // println!("Converged after {} iterations", i);
        return candidate_sigma;
    }
    // Update vega value
    let vega_value = vega(s, k, t, r, candidate_sigma, cp, n, VEGA_BUMP);

    // Safety check: Avoid division by zero
    if vega_value.abs() < 1e-10 {
        // Handle cases where vega is zero (e.g., deep ITM/OTM options)
        eprintln!("Warning: Vega too small. Aborting IV calculation.");
        return candidate_sigma; 
    }

    candidate_sigma = candidate_sigma - error / vega_value;
    test_price = option_price_(s, k, delta_t, r, candidate_sigma, cp,  n);

    }

    eprintln!("Warning: Did not converge after {} iterations. Returning best guess.", MAX_ITER);
    candidate_sigma
}

#[pymodule]
fn mzpricer(_py: Python, m: &PyModule) -> PyResult<()> {
    // Expose the Rust function 'option_price' to Python as 'option_price'
    m.add_class::<TimeDuration>()?;
    m.add_class::<StockPrice>()?;
    m.add_class::<OptionType>()?;
    m.add_function(wrap_pyfunction!(option_iv, m)?)?;
    m.add_function(wrap_pyfunction!(option_price, m)?)?;
    Ok(())
}
