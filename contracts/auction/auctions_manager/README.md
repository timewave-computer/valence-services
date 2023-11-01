# Auctions Manager

This contract helps us to manage auctions and ease some of the overhead of managing multiple contracts.

The manager is a factory for new auctions that is only allowed to be called by the admin, its also the admin of all the auctions it creates, and a router contract in case you don't know the specific auction of a pair.

## Pair

`Pair(String, String)` - is a helper for creating a pair of tokens, each pair is unique and in auction context, the left token is the token that is being sold and the right token is the token you are buying with.

## Admin messages

Only the admin of the manager can call those messages.

### `NewAuction`:
This message instantiates a new auction contract and stores its address.

```rust
NewAuction {
  pair: Pair,
  min_auction_amount: Uint128,
  auction_strategy: AuctionStrategy,
}
```
* `Pair` - the pair this auction will work with
* `min_auction_amount` - the minimum amount of tokens that can be sent to the auction
* `auction_strategy` - the strategy that will be used for this auction, see the [auction strategy](../auction/README.md#auction-strategy) section for more details.

### `OpenAuction`:
This message opens an auction on the specified pair.

```rust
OpenAuction {
  pair: Pair,
  params: NewAuctionParams,
},
```

See section [StartAuction](../auction/README.md#StartAuction) for more details.

### `PauseAuctiuon { pair: Pair }`:
This message pauses the auction on the specified pair.

### `ResumeAuctiuon { pair: Pair }`:
This message resumes the auction on the specified pair if paused.

### `UpdateOracle { oracle_addr: String }`:
The message update the oracle address we have stored

## Executables

`AuctionFunds { pair: Pair }` - Send funds to be auctioned for a specific pair.

`WithdrawFunds { pair: Pair }` - Withdraw funds from a future auction.
