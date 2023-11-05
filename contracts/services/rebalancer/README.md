# Rebalancer

This is the rebalancer contract, a single contract where accounts can register to and rebalance their current portfolio into their target portfolio.

## How to use

1. Create Valence account and deposit funds
2. Register the account to the rebalancer contract with desired config
3. The rebalancer will run daily rebalancing the account funds into the desired portfolio based on the PID parameters specified in the config.
4. You or the trustee can pause or deregister from the service at anytime.

## How to register

Rebalancer expects the following data structure:

```rust
pub struct RebalancerData {
    pub trustee: Option<String>,
    pub base_denom: String,
    pub targets: Vec<Target>,
    pub pid: PID,
    pub max_limit_bps: Option<u64>, // BPS
    pub target_override_strategy: TargetOverrideStrategy,
}
```

### Trustee

An optional trustee address that is only allowed to pause/resume the rebalancer for an account

### Base Denom

The base denom this account is calculating the portfolio in. This is the denom that will be used to calculate the portfolio value and the target value.

- Can only be one of the whitelisted denoms.

### Targets

A list of targets you want to rebalance into. Each target has a denom and a weight. The weight is the percentage of the portfolio you want to allocate to this target.

- Can only be one of the whitelisted denoms.

```rust
pub struct Target {
    pub denom: String,
    pub percentage: u64, // BPS
    pub min_balance: Option<Uint128>,
}
```

Each target needs to specify what is the denom, the percentage of the portfolio you want to allocate to this target and an optional min balance. The min balance is the minimum amount of funds you want to keep in this target.

### Order of that targets in the list

The order of the targets is very important mainly for 2 reasons:

1. Calculations happens in order of the targets in the list, so the first target will be calculated first, the higher the target is in the list, the higher the chance it will be rebalanced if limits are reached.
2. When `TargetOverrideStrategy::Priority` is used (read below), the priority is determined by the order of the targets in the list.

### PID

The PID parameters that will be used to calculate the rebalance amount. [Read more about PID](https://en.wikipedia.org/wiki/Proportional%E2%80%93integral%E2%80%93derivative_controller)

```rust
pub struct PID {
    pub p: String,
    pub i: String,
    pub d: String,
}
```

Each parameter is a string that represents a decimal number.

### Max Limit

An optional limit of the max amount of tokens we can sell during a single rebalance cycle. This is BPS from the total value of your portfolio, so if for example an account has 1000$, and the max limit is 1000 BPS (10%), the max amount of tokens that can be sold is 100$ (10% of 1000$).

### Target Override Strategy

In some cases the rebalancer will have to override the target percentage because of other priority settings (like min_balance of a specific target).

This field allows the account to choose what strategy to follow when overriding:

```rust
pub enum TargetOverrideStrategy {
    Proportional,
    Priority,
}
```

`Proportional` - will spread the override amount to all the other targets based on their weight.

`Priority` - will fulfil the override amount in order of priority. The priority is determined by the order of the targets in the targets list.

Example: A user has set the following rebalancing targets for tokens with denominations A, B, and C:

Target A - 25%, no min_balance
Target B - 25%, no min_balance
Target C - 50%, must maintain min_balance of 100 tokens.

Now suppose at one particular rebalancing interval, 100 tokens of the C denomination make up 60% of the total portfolio valuation. This a problem because the user has also required the Rebalancer to maintain 50% cumulatively in token A and B, with 25% of the total portfolio value in denomination A and another 25% in denomination B. Given that it can only maintain a total of 40% of the total value in A and B, what strategy should the Rebalancer pursue to reassign weights in A and B?

If we choose `Proportional` strategy, the 40% will be spread between Target A and Target B based on their weight, so each target will get 20%.

If we choose `Priority` strategy, the 40% will be allocated to Target A and Target B in order of priority, so Target A will get 25% and Target B will get 15%. The order of priority is determined by the order of the targets in the targets list.

## Target's min_balance

The minimum amount of tokens this target should have in the account, we never rebalance below this amount, and will rebalance to this amount overriding other targets percentage if needed.

This field is the top priority, and will override other targets if needed to fulfil the min_balance, if the min_balance amount is lower then the total value of the account, we will rebalance the max possible to the `min_balance` target.

`min_balance` can be only applied to a single target in the list.

## Call the system

To call the system you need to send a message to the rebalancer contract with the following structure:

```rust
SystemRebalance { limit: Option<u64> },
```

`limit` - Amount of accounts to run over in a single call, Default: `50`

We are using a "self-tracking system" that tracks the status of the processing, and set the status to "Finished" once we ran over all the account for the current cycle.

This allows anyone to call this message until all accounts has been proccessed.
