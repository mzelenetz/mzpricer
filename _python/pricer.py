import math

class OptionType:
    """An enum-like class representing the type of option."""
    Call = 1
    Put = -1

class TimeDuration:
    """
    A class to represent a duration of time, used to calculate time to expiration in years.
    
    Attributes:
        value (float): The magnitude of the time duration (e.g., 90 for 90 days).
        factor (float): The number of time units in a year (e.g., 365, 252, etc.).
    """
    def __init__(self, value: float, factor: float):
        self.value = value
        self.factor = factor

    def to_years(self) -> float:
        """Calculates the total time to expiration (T) in years."""
        return self.value / self.factor

def option_price(
    s: float, 
    k: float, 
    t: TimeDuration, 
    r: float, 
    sigma: float, 
    n: int, 
    cp: int
) -> float:
    """
    Calculates the price of an American option using the Binomial Tree model.

    The implementation follows the Cox-Ross-Rubinstein (CRR) framework with early exercise
    checks via backward induction.

    Args:
        s (float): Current stock price.
        k (float): Strike price.
        t (TimeDuration): Time to expiration object.
        r (float): Risk-free interest rate (annualized).
        sigma (float): Volatility of the underlying asset (annualized).
        n (int): Number of steps (periods) in the binomial tree.
        cp (int): Option type (1 for Call, -1 for Put).

    Returns:
        float: The calculated American option price.
    """
    if n <= 0:
        raise ValueError("Number of steps (n) must be a positive integer.")
    
    # Total time to expiration in years
    t_years = t.to_years()
    # Length of each time step (delta_t)
    delta_t = t_years / n

    # --- Binomial Model Parameters ---
    
    # Calculate the up (u) and down (d) factors (Volatility based)
    # math.exp(x) is equivalent to x.exp() in Rust/f64
    u = math.exp(sigma * math.sqrt(delta_t))
    d = 1.0 / u

    # Calculate the risk-neutral probability (p)
    a = math.exp(r * delta_t)
    p = (a - d) / (u - d)

    # Discount Factor for a single step
    df = math.exp(-r * delta_t)

    # --- Setup for Backward Induction ---

    # Determine the sign and base for payoff calculation based on OptionType
    # cp will be 1.0 for Call and -1.0 for Put
    sign = float(cp)
    base = -k if cp == OptionType.Call else k

    # Initialize the array for option prices at the final time step (T)
    # This array will hold prices at n + 1 nodes
    option_prices = [0.0] * (n + 1)
    
    # --- 1. Calculate all possible stock prices at maturity (T) ---
    stock_prices = [0.0] * (n + 1)
    
    # Price after 'i' up moves and 'n - i' down moves
    for i in range(n + 1):
        num_up_moves = i
        num_down_moves = n - i
        stock_prices[i] = s * (u ** num_up_moves) * (d ** num_down_moves)

    # --- 2. Calculate the option price (payoff) at maturity (T) ---
    for i in range(n + 1):
        # Payoff: max(sign * S_T + base, 0)
        option_prices[i] = max(sign * stock_prices[i] + base, 0.0)

    # --- 3. Backward Induction and Check for Early Exercise (American Option) ---
    # Iterate backwards through time steps (from T-1 down to 0)
    for i in range(n - 1, -1, -1):
        # We re-calculate the option prices for the time step 'i'
        for j in range(i + 1):
            # Continuation Value (European value): Expected discounted payoff
            # option_prices[j+1] is the up node, option_prices[j] is the down node from time i+1
            continuation_value = (p * option_prices[j + 1] + (1.0 - p) * option_prices[j]) * df
            
            # Stock price at the current node (time step i, j up moves)
            num_up_moves = j
            num_down_moves = i - j
            s_i = s * (u ** num_up_moves) * (d ** num_down_moves)
            
            # Intrinsic Value (Value if exercised now)
            intrinsic_value = max(sign * s_i + base, 0.0)
            
            # The value of the American option is max(Intrinsic Value, Continuation Value)
            option_prices[j] = max(intrinsic_value, continuation_value)

        # The option_prices array is effectively truncated for the next iteration 
        # as we only update indices 0 through i
        
    # The final result is the option price at time t=0 (option_prices[0])
    return option_prices[0]

# --- Example Usage ---
# t_duration = TimeDuration(value=90.0, factor=365.0) # 90 days / 365 days per year
# price = option_price(
#     s=50.0,          # Stock Price
#     k=50.0,          # Strike Price
#     t=t_duration,    # Time Duration (T)
#     r=0.05,          # Risk-Free Rate
#     sigma=0.30,      # Volatility
#     n=100,           # Number of Steps
#     cp=OptionType.Call
# )
# print(f"Option Price: {price}")