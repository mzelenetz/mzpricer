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



# -------------------------------------------------------------
# 6. Greeks
# -------------------------------------------------------------
def setup():
    S = [100.0, 100.0]
    K = [100.0, 100.0]
    T = [TimeDuration(365, 365)] * 2
    R = [0.05] * 2
    SIG = [0.20] * 2
    CP = [OptionType.Call, OptionType.Put]

    results, errors = mzpricer.option_greeks( S, K, T, R, SIG, CP, precision=2000)

    return results, errors

def test_vector_vegas():
    results, errors = setup()

    print(results)
    assert all(e == 0 for e in errors)

    vegas = [r['vega'] for r in results]

    print("Vegas:", vegas)
    for result in results:
        assert abs(result['vega'] - 0.37524) < 0.002
    diff = abs(vegas[0] - vegas[1])
    assert diff < 1e-2

def test_vector_deltas():
    results, errors = setup()

    assert all(e == 0 for e in errors)

    deltas = [r['delta'] for r in results]

    print("Deltas:", deltas)
    delta_sum = deltas[0] - deltas[1]
    print(delta_sum)
    assert abs(delta_sum - 1) > 0.001, "Delta sum is too close to 1.0, suggesting European parity"
    assert deltas[1] < 0, "Put Delta non-negative"
    assert deltas[0] > 0, "Call Delta negative"


def test_vector_theta():
    results, _ = setup()

    thetas = [r['theta'] for r in results]
    expt_theta = -0.01757

    print("Theta:", thetas)
    assert abs(expt_theta - thetas[0]) < 0.01, "Theta too far from expected"

def test_vector_gamma():
    results, _ = setup()

    gammas = [r['gamma'] for r in results]
    expt_gamma = 0.01876

    print("Gamma:", gammas)
    assert abs(expt_gamma - gammas[0]) < 0.01, "Gamma too far from expected"

def test_vector_rho():
    results, _ = setup()
    print(results)
    rhos = [r['rho'] for r in results]
    expt_rho = 0.53232

    print("Rho:", rhos)
    assert abs(expt_rho - rhos[0]) < 0.001, "Rho too far from expected"