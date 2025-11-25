from enum import IntEnum

# --------------------------
# Public Python OptionType
# --------------------------
class OptionType(IntEnum):
    Call = 0
    Put = 1

# --------------------------
# Re-export classes/functions 
# from the Rust extension
# --------------------------
from .mzpricer import (
    option_price,
    option_iv,
    option_greeks,
    TimeDuration,
    StockPrice,
)
