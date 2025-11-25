import pytest
import mzpricer
from mzpricer import option_price, option_iv, TimeDuration, StockPrice
from enum import IntEnum


class OptionType(IntEnum):
    Call = 0
    Put = 1


# -------------------------------------------------------------
# 1. Scalar Pricing
# -------------------------------------------------------------
def test_scalar_pricing():
    S = 100.0
    K = 100.0
    T = TimeDuration(value=365, factor=365)
    R = 0.05
    SIGMA = 0.20

    px_call = mzpricer.option_price(S, K, T, R, SIGMA, OptionType.Call)
    px_put  = mzpricer.option_price(S, K, T, R, SIGMA, OptionType.Put)

    assert abs(px_call - 10.44658514) < 0.10
    assert abs(px_put  - 6.08881011)  < 0.10


# -------------------------------------------------------------
# 2. Scalar IV
# -------------------------------------------------------------
def test_scalar_iv():
    S = 100.0
    K = 100.0
    T = TimeDuration(value=365, factor=365)
    R = 0.05
    SIGMA = 0.20

    px = mzpricer.option_price(S, K, T, R, SIGMA, OptionType.Call)
    iv = mzpricer.option_iv(px, S, K, T, R, 0.40, OptionType.Call)

    assert abs(iv - 0.20) < 0.002


# -------------------------------------------------------------
# 3. Vector Pricing
# -------------------------------------------------------------
def test_vector_pricing():
    S = [100.0, 100.0, 100.0, 100.0]
    K = [100.0, 100.0, 110.0, 110.0]
    T = [TimeDuration(365, 365)] * 4
    R = [0.05, 0.05, 0.05, 0.05]
    SIG = [0.20, 0.20, 0.20, 0.20]
    CP = [OptionType.Call, OptionType.Put, OptionType.Call, OptionType.Put]

    prices, errors = mzpricer.option_price(S, K, T, R, SIG, CP, precision=500)

    assert all(e == 0 for e in errors)

    assert abs(prices[0] - 10.44658514) < 0.10
    assert abs(prices[1] - 6.08881011)  < 0.10
    assert abs(prices[2] - 6.042219267573585) < 0.10
    assert abs(prices[3] - 11.974393469523353) < 0.10


# -------------------------------------------------------------
# 4. Vector Implied Vols
# -------------------------------------------------------------
def test_vector_iv():
    S = [100.0, 100.0, 100.0, 100.0]
    K = [100.0, 100.0, 110.0, 110.0]
    T = [TimeDuration(365, 365)] * 4
    R = [0.05] * 4
    SIG = [0.20] * 4
    CP = [OptionType.Call, OptionType.Put, OptionType.Call, OptionType.Put]

    prices, _ = mzpricer.option_price(S, K, T, R, SIG, CP, precision=500)
    guesses = [0.40] * 4

    ivs, errors = mzpricer.option_iv(prices, S, K, T, R, guesses, CP, precision=500)

    assert all(e == 0 for e in errors)

    for iv in ivs:
        assert abs(iv - 0.20) < 0.002


# -------------------------------------------------------------
# 5. s_prime correctness
# -------------------------------------------------------------
def test_s_prime():
    sp = StockPrice(spot_price=105, dividend_amount=5, time_to_dividend_days=365, rate=0.05)
    s_adj = sp.s_prime()

    import math
    expected = 105 - 5 * math.exp(-0.05 * 1.0)

    assert abs(s_adj - expected) < 1e-9
