use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

use mzpricer_core::{
    TimeDuration as RustTimeDuration,
    StockPrice as RustStockPrice,
    OptionType,
    PriceError,
    greeks,
    option_price_scalar,
    option_price_vector,
    option_iv_scalar,
    option_iv_vector,
};

#[pyclass(name = "TimeDuration")]
#[derive(Clone, Copy)]
pub struct PyTimeDuration {
    #[pyo3(get)]
    pub value: f64,
    #[pyo3(get)]
    pub factor: f64,
}

#[pymethods]
impl PyTimeDuration {
    #[new]
    fn new(value: f64, factor: f64) -> Self {
        PyTimeDuration { value, factor }
    }

    fn to_years(&self) -> f64 {
        self.value / self.factor
    }
}

impl PyTimeDuration {
    pub fn to_rust(&self) -> RustTimeDuration {
        RustTimeDuration {
            value: self.value,
            factor: self.factor,
        }
    }
}

#[pyclass(name = "StockPrice")]
#[derive(Clone)]
pub struct PyStockPrice {
    inner: RustStockPrice,
}

#[pymethods]
impl PyStockPrice {
    #[new]
    fn new(spot_price: f64, dividend_amount: f64, time_to_dividend_days: f64, rate: f64) -> Self {
        PyStockPrice {
            inner: RustStockPrice {
                spot_price,
                dividend_amout: dividend_amount,
                time_to_dividend_days,
                rate,
            }
        }
    }

    fn s_prime(&self) -> f64 {
        self.inner.s_prime()
    }
}

impl PyStockPrice {
    pub fn to_rust(&self) -> RustStockPrice {
        self.inner
    }
}


fn extract_durations(objs: &Bound<'_, PyAny>) -> PyResult<Vec<RustTimeDuration>> {
    let py_list: Vec<PyTimeDuration> = objs.extract()?;
    Ok(py_list.iter().map(|p| p.to_rust()).collect())
}

fn extract_optiontype_list(objs: &Bound<'_, PyAny>) -> PyResult<Vec<OptionType>> {
    let vals: Vec<usize> = objs.extract()?;
    Ok(vals.into_iter().map(|v| if v == 0 { OptionType::Call } else { OptionType::Put }).collect())
}

#[pyfunction]
fn option_price(
    s: &Bound<'_, PyAny>,
    k: &Bound<'_, PyAny>,
    t: &Bound<'_, PyAny>,
    r: &Bound<'_, PyAny>,
    sigma: &Bound<'_, PyAny>,
    cp: &Bound<'_, PyAny>,
    precision: Option<usize>,
) -> PyResult<PyObject> {
    let prec = precision.unwrap_or(500);

    Python::with_gil(|py| {
        // Scalar
        if let (Ok(s), Ok(k), Ok(tpy), Ok(r), Ok(sig), Ok(cp)) = (
            s.extract::<f64>(),
            k.extract::<f64>(),
            t.extract::<PyTimeDuration>(),
            r.extract::<f64>(),
            sigma.extract::<f64>(),
            cp.extract::<usize>(),
        ) {
            let rust_cp = if cp == 0 { OptionType::Call } else { OptionType::Put };
            let px = option_price_scalar(s, k, &tpy.to_rust(), r, sig, rust_cp, prec);
            return Ok(px.into_py(py));
        }

        // Vector
        let s_vec: Vec<f64> = s.extract()?;
        let k_vec: Vec<f64> = k.extract()?;
        let t_vec = extract_durations(t)?;
        let r_vec: Vec<f64> = r.extract()?;
        let sig_vec: Vec<f64> = sigma.extract()?;
        let cp_vec = extract_optiontype_list(cp)?;

        let (prices, errors) =
            option_price_vector(&s_vec, &k_vec, &t_vec, &r_vec, &sig_vec, &cp_vec, prec);

        let err_codes: Vec<usize> = errors.into_iter().map(|e| e as usize).collect();

        Ok((prices, err_codes).into_py(py))
    })
}

#[pyfunction]
fn option_iv(
    price: &Bound<'_, PyAny>,
    s: &Bound<'_, PyAny>,
    k: &Bound<'_, PyAny>,
    t: &Bound<'_, PyAny>,
    r: &Bound<'_, PyAny>,
    candidate_sigma: &Bound<'_, PyAny>,
    cp: &Bound<'_, PyAny>,
    precision: Option<usize>,
) -> PyResult<PyObject> {

    let prec = precision.unwrap_or(500);

    Python::with_gil(|py| {
        // Scalar case
        if let (Ok(price), Ok(s), Ok(k), Ok(tpy), Ok(r), Ok(sig0), Ok(cp_raw)) = (
            price.extract::<f64>(),
            s.extract::<f64>(),
            k.extract::<f64>(),
            t.extract::<PyTimeDuration>(),
            r.extract::<f64>(),
            candidate_sigma.extract::<f64>(),
            cp.extract::<usize>(),
        ) {
            let rust_cp = if cp_raw == 0 { OptionType::Call } else { OptionType::Put };
            let iv = option_iv_scalar(price, s, k, &tpy.to_rust(), r, sig0, rust_cp, prec);
            return Ok(iv.into_py(py));
        }

        // Vector case
        let price_vec: Vec<f64> = price.extract()?;
        let s_vec: Vec<f64> = s.extract()?;
        let k_vec: Vec<f64> = k.extract()?;
        let t_vec = extract_durations(t)?;
        let r_vec: Vec<f64> = r.extract()?;
        let sig_vec: Vec<f64> = candidate_sigma.extract()?;
        let cp_vec = extract_optiontype_list(cp)?;

        let (ivs, errors) =
            option_iv_vector(&price_vec, &s_vec, &k_vec, &t_vec, &r_vec, &sig_vec, &cp_vec, prec);

        let err_codes: Vec<usize> = errors.into_iter().map(|e| e as usize).collect();
        Ok((ivs, err_codes).into_py(py))
    })
}


#[pyfunction]
fn option_greeks(
    s: &Bound<'_, PyAny>,
    k: &Bound<'_, PyAny>,
    t: &Bound<'_, PyAny>,
    r: &Bound<'_, PyAny>,
    sigma: &Bound<'_, PyAny>,
    cp: &Bound<'_, PyAny>,
    precision: Option<usize>,
) -> PyResult<PyObject> {
    let prec = precision.unwrap_or(500);

    Python::with_gil(|py| {
        // Vector
        let s_vec: Vec<f64> = s.extract()?;
        let k_vec: Vec<f64> = k.extract()?;
        let t_vec = extract_durations(t)?;
        let r_vec: Vec<f64> = r.extract()?;
        let sig_vec: Vec<f64> = sigma.extract()?;
        let cp_vec = extract_optiontype_list(cp)?;

        let (results, errors) =
            greeks(&s_vec, &k_vec, &t_vec, &r_vec, &sig_vec, &cp_vec, prec);


        let err_codes: Vec<usize> = errors.into_iter().map(|e| e as usize).collect();

        let py_results = results
            .into_iter()
            .map(|g| {
                let d = PyDict::new_bound(py);
                d.set_item("delta", g.delta)?;
                d.set_item("gamma", g.gamma)?;
                d.set_item("vega", g.vega)?;
                d.set_item("theta", g.theta)?;
                Ok::<_, PyErr>(d)
            })
            .collect::<PyResult<Vec<_>>>()?
            .into_py(py);

        let py_err_codes = err_codes.into_py(py);
        Ok((py_results, py_err_codes).into_py(py))
    })
}


#[pymodule]
fn mzpricer(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTimeDuration>()?;
    m.add_class::<PyStockPrice>()?;

    m.add_function(wrap_pyfunction!(option_greeks, m)?)?;
    m.add_function(wrap_pyfunction!(option_price, m)?)?;
    m.add_function(wrap_pyfunction!(option_iv, m)?)?;
    

    Ok(())
}
