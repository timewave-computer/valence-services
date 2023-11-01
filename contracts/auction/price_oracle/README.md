# Price oracle
This contract is a price oracle for the auctions.

Its a very basic oracle that allows our auction to get an initial price for the auction, but also enable the oracle to take the average prices from the auction itself.

It does have a way to take prices from a 3rd party source, to avoid cold start problem or price freshness issues.

## Update price
```rust
UpdatePrice {
  pair: Pair,
  price: Option<Decimal>
}
```

`pair` - The pair to update the price for. Ex: `(ATOM, NTRN)` or `["ATOM", "NTRN"]` in json.

`price` - Optional, the price of the pair, if set, it will update the price manually, if not set, it will try to get the price from the auction, only if the auction has been run at least 3 times, and has 3 past prices.

We want to avoid relying on manually price updates as much as possible, but this is a way to bootstrap the oracle, or to update the price if the auction doesn't have a fresh enough price for us.

# Get price
```rust
#[returns(GetPriceResponse)]
GetPrice { pair: Pair },
```

`pair` - The pair to get the price for. Ex: `("uatom", "untrn")` or `["uatom", "untrn"]` in json.

Response is:
```rust
pub struct GetPriceResponse {
    pub price: Decimal,
    pub time: Timestamp,
}
```

`price` - The price of the pair.

`time` - The time the price was last updated. This allows any contract who rely on this price, to determine how fresh the price is, and if they want to use it or not.
