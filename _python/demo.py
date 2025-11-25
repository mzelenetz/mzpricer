from mzpricer import OptionType, TimeDuration, StockPrice
import mzpricer
import time 

# Standard market parameters for tests
S = 100.0   # Current Stock Price
s_pricer = StockPrice(spot_price=S, dividend_amount=0.0, time_to_dividend_days=0.0, rate=0.0) 
s_prime = s_pricer.s_prime()  # Adjusted stock price considering dividends
K = 100.0   # Strike Price
T = 1.0     # Time to Expiration (1 year)
R = 0.05    # Risk-Free Rate (5%)
SIGMA = 0.20 # Volatility (20%)

duration = TimeDuration(value=365, factor=365) # 1 year duration with 252 days in a year

# Tolerance for floating point comparison (Binomial is an approximation)
DELTA = 0.05 # 5 cents tolerance for N=100 steps

start_time = time.perf_counter()
res_c = mzpricer.option_price(s_prime, K, duration, R, SIGMA, OptionType.Call)
res_p = mzpricer.option_price(s_prime, K, duration, R, SIGMA, OptionType.Put)
end_time = time.perf_counter()

elapsed_time = end_time - start_time

print(f"Rust Execution time: {elapsed_time:.6f} seconds. Result Call: {res_c:.8f}")
print(f"Rust Execution time: {elapsed_time:.6f} seconds. Result Put: {res_p:.8f}")
 
#####

start_time = time.perf_counter()
res_iv = mzpricer.option_iv(res_c, s_prime, K, duration, R, .40, OptionType.Call)
end_time = time.perf_counter()

elapsed_time = end_time - start_time

print(f"Rust Execution time: {elapsed_time:.6f} seconds. Result: {res_iv:.8f}")

#####

start_time = time.perf_counter()
res_iv = mzpricer.option_iv(res_p, s_prime, K, duration, R, .40, OptionType.Put)
end_time = time.perf_counter()

elapsed_time = end_time - start_time

print(f"Rust Execution time: {elapsed_time:.6f} seconds. Result: {res_iv:.8f}")


#####

start_time = time.perf_counter()
results, errors = mzpricer.option_price([s_prime,s_prime,s_prime,s_prime], [K,K, 110.0, 110.0], [duration,duration,duration,duration], [R,R,R,R], [SIGMA,SIGMA,SIGMA,SIGMA], [OptionType.Call, OptionType.Put,OptionType.Call, OptionType.Put], precision=500)
end_time = time.perf_counter()

elapsed_time = end_time - start_time

print(f"Compute Prices: {elapsed_time:.6f} seconds. Result: {results} Errors: {errors}")

#####

start_time = time.perf_counter()
results_iv, errors_iv = mzpricer.option_iv(results, [s_prime,s_prime,s_prime,s_prime], [K,K, 110.0, 110.0], [duration,duration,duration,duration], [R,R,R,R], [.5,.7,.3,.8], [OptionType.Call, OptionType.Put,OptionType.Call, OptionType.Put], precision=500)
end_time = time.perf_counter()

elapsed_time = end_time - start_time

print(f"Compute Vols: {elapsed_time:.6f} seconds. Result: {results_iv} Errors: {errors_iv}")

#####

start_time = time.perf_counter()
results_iv, errors_iv = mzpricer.option_greeks([s_prime,s_prime,s_prime,s_prime], [K,K, 110.0, 110.0], [duration,duration,duration,duration], [R,R,R,R], [.5,.7,.3,.8], [OptionType.Call, OptionType.Put,OptionType.Call, OptionType.Put], precision=1000)
end_time = time.perf_counter()

elapsed_time = end_time - start_time

print(f"Compute Greeks: {elapsed_time:.6f} seconds. Result: {results_iv} Errors: {errors_iv}")
