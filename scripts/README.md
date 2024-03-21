# Helper scripts

In this folder we have helper scripts that help us manage our systems on a testnet

The scripts can work on neutron and juno, but for testing with DAO DAO, we init contracts on juno but not on neutron yet.

Those scripts are just helpers to make it easier to deploy and play with the contract, they are not meant to be used on production

- Currently scripts have static values in the file for the messages parameters, so before running a script, you need to manually change the parameters to match your need.

## init.sh

This script helps us to init contracts on the chain we choose to.

Call:

```sh
./init.sh $CHAIN $COMMAND
```

Example to init an account:

```sh
./init.sh juno account
```

### Commands

Each command init a different contract, currently we have 4:

1. account
2. services-manager
3. auctions-manager
4. rebalancer

## new_auction.sh

This script helps us to create new auctions for pair.

You can modify the defaults in the file, or use the parameters:

- `-p | --pair` - the pair. Example: `["uatom", "untrn"]`
- `-as | --auction-strategy` - Auction strategy. Defaults: `{ "start_price_perc": 2000, "end_price_perc": 2000 }`
- `-ch | --chain-halt` - Chain halt config. Defaults: `{ "cap": "14400", "block_avg": "3" }`
- `-pf | --price-freshness` - Price freshness strategy. Defaults: `{ "limit": "3", "multipliers": [["2", "2"], ["1", "1.5"]] }`

Call:

```sh
./new_auction.sh $CHAIN -p $PAIR -as $AUCTION_STRATEGY -ch $CHAIN_HALT -pf $PRICE_FRESHNESS
```

Example:

```sh
./new_auction.sh juno -p $PAIR -as $AUCTION_STRATEGY -ch $CHAIN_HALT -pf $PRICE_FRESHNESS
```

## add_service_to_manager.sh
Helps with adding a service to the manager contract

Call:

```sh
./add_service_to_manager.sh $CHAIN $SERVICE_NAME $SERVICE_ADDR
```

Example to add rebalancer:

```sh
./add_service_to_manager.sh juno rebalancer juno15she5505reyvgvg9cz5g4k6y5ktxg4eja5tuytxmj0x0gs0cyjwq2sjgr0
```

## update_oracle_addr.sh
Helps with adding oracle address to the auctions manager

Call:

```sh
./update_oracle_addr.sh $CHAIN $ORACLE_ADDR
```

Example to add oracle address:

```sh
./update_oracle_addr.sh juno juno14vgs85az6xlfzkczzq06agk2tv8zkdxqdue4gs08h0f60smu3jjqfryaj2
```
