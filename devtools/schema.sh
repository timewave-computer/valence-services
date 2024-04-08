#!/usr/bin/env bash

CONTRACTS_DIR=$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )/../contracts
echo $CONTRACTS_DIR

# Schema for account
CONTRACT="account"
cd "$CONTRACTS_DIR/$CONTRACT" || { echo "No $CONTRACT dir" ; exit 1; }
cargo schema || { echo "Failed doing schema $CONTRACT" ; exit 1; }
rm -r schema/raw

# Schema for rebalancer
CONTRACT="rebalancer"
cd "$CONTRACTS_DIR/services/$CONTRACT" || { echo "No $CONTRACT dir" ; exit 1; }
cargo schema || { echo "Failed doing schema $CONTRACT" ; exit 1; }
rm -r schema/raw

# Schema for rebalancer
CONTRACT="services_manager"
cd "$CONTRACTS_DIR/$CONTRACT" || { echo "No $CONTRACT dir" ; exit 1; }
cargo schema || { echo "Failed doing schema $CONTRACT" ; exit 1; }
rm -r schema/raw

# Schema for auction
CONTRACT="auction"
cd "$CONTRACTS_DIR/auction/$CONTRACT" || { echo "No $CONTRACT dir" ; exit 1; }
cargo schema || { echo "Failed doing schema $CONTRACT" ; exit 1; }
rm -r schema/raw

# Schema for auction
CONTRACT="auctions_manager"
cd "$CONTRACTS_DIR/auction/$CONTRACT" || { echo "No $CONTRACT dir" ; exit 1; }
cargo schema || { echo "Failed doing schema $CONTRACT" ; exit 1; }
rm -r schema/raw

# Schema for auction
CONTRACT="price_oracle"
cd "$CONTRACTS_DIR/auction/$CONTRACT" || { echo "No $CONTRACT dir" ; exit 1; }
cargo schema || { echo "Failed doing schema $CONTRACT" ; exit 1; }
rm -r schema/raw