pub mod pricer;
pub use pricer::{
    OptionType,
    TimeDuration,
    PriceError,
    StockPrice,
    greeks,
    option_price_scalar,
    option_price_vector,
    option_iv_scalar,
    option_iv_vector,
    vega,
};