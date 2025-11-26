# MZPricer

## Installation
Ensure rust in installed.
Ensure uv in installed.

To build the library: `uv pip install -e mzpricer-py`
To run the tests: `uv run python -m pytest mzpricer-py/tests`

## Run It
To check that it runs: 
`python python/demo.py`


## Release Build
1. From in mzpricer-py: `uv run --active maturin build --release`
1. `uv pip install target/wheels/mzpricer-*.whl`

## To add a new function
- Implement the function in the core library (pricer.rs)
    - Make sure there is an entry in 
```rust
#[pymodule]
fn mzpricer(_py: Python, m: &PyModule) -> PyResult<()> {}
```
- Add to library (lib.rs)
- Write tests (pricer_tests.rs)
- Add to `__init__.py` (mzpricer-py/mzpricer/__init__.py)
- Implement in `mzpricer-py/lib.rs`