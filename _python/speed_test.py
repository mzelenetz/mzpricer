from mzpricer import OptionType, TimeDuration
from pricer import option_price
import option_pricer as cpricer
import mzpricer
import time 

# Standard market parameters for tests
S = 100.0   # Current Stock Price
K = 100.0   # Strike Price
T = 1.0     # Time to Expiration (1 year)
R = 0.05    # Risk-Free Rate (5%)
SIGMA = 0.20 # Volatility (20%)

duration = TimeDuration(value=252, factor=252) # 1 year duration with 252 days in a year

# Tolerance for floating point comparison (Binomial is an approximation)
DELTA = 0.05 # 5 cents tolerance for N=100 steps

start_time = time.perf_counter()
res = mzpricer.option_price(S, K, duration, R, SIGMA, OptionType.Call, 100)
end_time = time.perf_counter()

elapsed_time = end_time - start_time

from pricer import OptionType as OptionType2
t_duration = TimeDuration(value=252.0, factor=252.0) # 90 days / 365 days per year

start_time2 = time.perf_counter()
price = option_price(
    s=S,          # Stock Price
    k=K,          # Strike Price
    t=t_duration,    # Time Duration (T)
    r=R,          # Risk-Free Rate
    sigma=SIGMA,      # Volatility
    n=100,           # Number of Steps
    cp=OptionType2.Call
)
end_time2 = time.perf_counter()

elapsed_time2 = end_time2 - start_time2

t_duration_c = cpricer.TimeDuration(value=252.0, factor=252.0) # 90 days / 365 days per year
start_time3 = time.perf_counter()
price_c = cpricer.option_price_cython(
    s=S,          # Stock Price
    k=K,          # Strike Price
    t=t_duration_c,    # Time Duration (T)
    r=R,          # Risk-Free Rate
    sigma=SIGMA,      # Volatility
    n=100,           # Number of Steps
    cp=1.0
)
end_time3 = time.perf_counter()

elapsed_time3 = end_time3 - start_time3
# Print the execution time
print(f"Rust Execution time:        {elapsed_time:.6f} seconds. Result: {res:.8f}")
print(f"Pure Python Execution time: {elapsed_time2:.6f} seconds. Result: {price:.8f}")
print(f"Cython Execution time:      {elapsed_time3:.6f} seconds. Result: {price_c:.8f}")