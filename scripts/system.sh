#!/bin/bash

CHAIN=$1
shift
COMMAND=$1
shift

if [[ "$CHAIN" == 'juno' ]]; then
  BINARY="junod"
  GAS_PRICES="0.025ujunox"
  OWNER_ADDR="juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn"
  FEES="10000ujunox"

  REBALANCER_ADDR="juno18rpfddza4g3h5a05fzwq6xwepzh2t0twhetly4y5aqjyeh8cjflspa8fqr"
  AUCTIONS_MANAGER="juno1tp2n8fa9848355hfd98lufhm84sudlvnzwvsdsqtlahtsrdtl6astvrz9j"

elif [[ "$CHAIN" == 'neutron' || "$CHAIN" == 'ntrn' ]]; then
  BINARY="neutrond"
  GAS_PRICES="0.025ntrn"
  OWNER_ADDR="neutron17s47ltx2hth9w5hntncv70kvyygvg0qr4ug32g"
  FEES="1000untrn"

# REBALANCER_ADDR=""
# AUCTIONS_MANAGER=""

else
  echo "Unknown chain"
fi

# EXECUTE_FLAGS="--gas-prices $GAS_PRICES --gas auto --gas-adjustment 1.4 -y"
EXECUTE_FLAGS="--fees $FEES --gas auto --gas-adjustment 1.4 -y"

declare -A pair1=([pair1]="ujunox" [pair2]="factory/juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn/vuusdcx")
declare -A pair2=([pair1]="factory/juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn/vuusdcx" [pair2]="ujunox")

declare -a pairs=(
  pair1
  pair2
)

if [[ "$COMMAND" == 'update-prices' ]]; then

  declare -A price1=([pair1]="ujunox" [pair2]="factory/juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn/vuusdcx" [price]="0.5")
  declare -A price2=([pair1]="factory/juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn/vuusdcx" [pair2]="ujunox" [price]="2.0")

  declare -a prices=(
    price1
    price2
  )

  for ((i = 0; i < "${#prices[*]}"; i++)); do
    curr="${prices[$i]}"

    pair1="${curr}[pair1]"
    pair2="${curr}[pair2]"
    price="${curr}[price]"

    ./update_price.sh $CHAIN ${!pair1} ${!pair2} ${!price}

    sleep 3
  done

elif [[ "$COMMAND" == 'rebalance' ]]; then
  LIMIT=$1
  shift

  if [ -z "$PRICE" ]; then
    execute_msg=$(jq -n \
      '{system_rebalance: {}}')

  else
    execute_msg=$(jq -n \
      --arg limit "$LIMIT" \
      '{system_rebalance: {
      limit: $limit,
    }}')

  fi

  $BINARY tx wasm execute $REBALANCER_ADDR "$execute_msg" --from $OWNER_ADDR $EXECUTE_FLAGS

elif [[ "$COMMAND" == 'open-auctions' ]]; then
  END_BLOCK=$1
  shift
  START_BLOCK=$1
  shift

  for ((i = 0; i < "${#pairs[*]}"; i++)); do
    curr="${pairs[$i]}"

    pair1="${curr}[pair1]"
    pair2="${curr}[pair2]"

    if [ -z "$START_BLOCK" ]; then
      execute_msg=$(jq -n \
        --arg pair1 "${!pair1}" \
        --arg pair2 "${!pair2}" \
        --arg end_block "$END_BLOCK" \
        '{admin: {
        open_auction: {
          pair: [$pair1, $pair2],
          params: {
            end_block: $end_block,
          }
        }
      }}')

    else
      execute_msg=$(jq -n \
        --arg pair1 "${!pair1}" \
        --arg pair2 "${!pair2}" \
        --arg end_block "$END_BLOCK" \
        --arg start_block "$START_BLOCK" \
        '{admin: {
        open_auction: {
          pair: [$pair1, $pair2],
          params: {
            end_block: $end_block,
            start_block: $start_block,
          }
        }
      }}')

    fi

    $BINARY tx wasm execute $AUCTIONS_MANAGER "$execute_msg" --from $OWNER_ADDR $EXECUTE_FLAGS

    sleep 3
  done

elif [[ "$COMMAND" == 'close-auctions' ]]; then
  LIMIT=$1
  shift

  if [ -z "$LIMIT" ]; then echo "[ERROR] limit is missing" && exit 1; fi

  for ((i = 0; i < "${#pairs[*]}"; i++)); do
    curr="${pairs[$i]}"

    pair1="${curr}[pair1]"
    pair2="${curr}[pair2]"

    execute_msg=$(jq -n \
      --arg pair1 "${!pair1}" \
      --arg pair2 "${!pair2}" \
      --argjson limit $LIMIT \
      '{finish_auction: {
          pair: [$pair1, $pair2],
          limit: $limit,
        }}')
  done

  echo $BINARY tx wasm execute $AUCTIONS_MANAGER "$execute_msg" --from $OWNER_ADDR $EXECUTE_FLAGS
else
  echo "Unknown command"
fi
