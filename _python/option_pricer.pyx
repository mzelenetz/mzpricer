# distutils: language=c
# cython: boundscheck=False
# cython: wraparound=False
# cython: cdivision=True

import numpy as np
cimport numpy as cnp
import math

# Define C-level type aliases for clarity
# These are necessary for the TimeDuration class attributes and function arguments
ctypedef double f64
ctypedef int i32
ctypedef unsigned int u32

# --- Class Definitions ---

# The OptionType class is simple enough that we can keep it as a pure Python class
# or just use integers (1 and -1) throughout the Cython function for maximum speed.
# We'll rely on integers for the speed optimization.

# The TimeDuration class must be defined with cdef attributes to be efficient.
cdef class TimeDuration:
    """
    A Cython class to represent a duration of time.
    cdef attributes make access and storage C-speed.
    """
    cdef f64 value
    cdef f64 factor

    def __init__(self, f64 value, f64 factor):
        # We assign the values to the cdef attributes in the Python __init__
        self.value = value
        self.factor = factor

    def to_years(self) -> f64:
        """Calculates the total time to expiration (T) in years."""
        return self.value / self.factor

# --- Core Pricing Function ---

cpdef f64 option_price_cython(
    f64 s,          # Current stock price
    f64 k,          # Strike price
    TimeDuration t, # Time to expiration object (Cython cdef class)
    f64 r,          # Risk-free interest rate (annualized)
    f64 sigma,      # Volatility of the underlying asset (annualized)
    u32 n,          # Number of steps (using u32 for unsigned integer)
    f64 cp          # Option type (1.0 for Call, -1.0 for Put)
) except? -2.0:
    """
    Calculates the price of an American option using the Binomial Tree model
    (CRR framework) with C-level speed.
    """
    if n == 0:
        # Use a distinguishable error return value
        raise ValueError("Number of steps (n) must be a positive integer.")

    # Access cdef attributes directly for speed
    cdef f64 t_years = t.value / t.factor
    cdef f64 delta_t = t_years / n

    # --- Binomial Model Parameters ---
    cdef f64 u, d, a, p, df, sign, base
    cdef u32 i, j
    cdef f64 s_i, continuation_value, intrinsic_value

    # Calculate the up (u) and down (d) factors
    u = math.exp(sigma * math.sqrt(delta_t))
    d = 1.0 / u

    # Calculate the risk-neutral probability (p)
    a = math.exp(r * delta_t)
    p = (a - d) / (u - d)

    # Discount Factor for a single step
    df = math.exp(-r * delta_t)

    # --- Setup for Backward Induction ---
    
    # Determine the sign and base for payoff calculation
    sign = cp # cp is 1.0 or -1.0
    # Base is -K for Call (sign=1.0) and K for Put (sign=-1.0)
    base = -k * sign

    # Use a NumPy array for the option prices for efficient memory management and C-level indexing
    # This array will hold prices at n + 1 nodes, representing the current time step.
    cdef cnp.ndarray[f64, ndim=1] option_prices = np.zeros(n + 1, dtype=np.float64)
    
    # --- 1. & 2. Calculate Stock Price and Option Payoff at Maturity (T) ---

    # We iterate backwards through the nodes (n up moves to 0 up moves)
    # The current stock price at node i is S * u^i * d^(n-i)
    for i in range(n + 1):
        # Calculate Stock Price S_T at node i
        s_i = s * (u ** i) * (d ** (n - i))
        
        # Calculate Option Payoff: max(sign * S_T + base, 0)
        option_prices[i] = max(sign * s_i + base, 0.0)

    # --- 3. Backward Induction and Check for Early Exercise (American Option) ---

    # Iterate backwards through time steps (from T-1 down to 0)
    for i in range(n - 1, -1, -1):
        # Iterate through nodes in the current time step (i)
        for j in range(i + 1):
            
            # Continuation Value (European value): Expected discounted payoff
            # option_prices[j+1] is the up node, option_prices[j] is the down node from time i+1
            expected_payoff = (p * option_prices[j + 1]) + ((1.0 - p) * option_prices[j])
            continuation_value = expected_payoff * df
            
            # Stock price at the current node (time step i, j up moves)
            # The number of down moves is i - j
            s_i = s * (u ** j) * (d ** (i - j))
            
            # Intrinsic Value (Value if exercised now)
            intrinsic_value = max(sign * s_i + base, 0.0)
            
            # The value of the American option is max(Intrinsic Value, Continuation Value)
            option_prices[j] = max(intrinsic_value, continuation_value)
        
        # We don't need to explicitly shorten the array, as the loop only reads up to i+1 
        # and writes up to i.

    # The final result is the option price at time t=0 (option_prices[0])
    return option_prices[0]