#!/bin/bash

COMMAND=$1
shift
CHAIN=$1
shift

if [[ "$CHAIN" == 'juno' ]]; then
  BINARY="junod"
  GAS_PRICES="0.025ujunox"
  OWNER_ADDR="juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn"

  CODE_ID_ACCOUNT=3794
  CODE_ID_SERVICES_MANAGER=3793
  CODE_ID_REBALANCER=3792
  CODE_ID_ORACLE=3791
  CODE_ID_AUCTION=3790
  CODE_ID_AUCTIONS_MANAGER=3789

  # Contracts addresses for init below
  ADDR_SERVICES_MANAGER="juno1gscdr8zw8njrqfad9m3jgw70s4zumqccka4k6cutlxen0krud08sxlqs9d"
  ADDR_AUCTIONS_MANAGER="juno1arszzw6yytxtq2l07eaqhuhradnmkdwftwc6vp3j3xaxgnlg3scq2fe4cn"

  # General data per chain
  WHITELISTED_DENOMS='[\"ujunox\", \"ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9\"]'
  WHITELISTED_BASE_DENOMS='[\"ujunox\", \"ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9\"]'
elif [[ "$CHAIN" == 'neutron' || "$CHAIN" == 'ntrn' ]]; then
  BINARY="neutrond"
  GAS_PRICES="0.025ntrn"
  OWNER_ADDR="neutron17s47ltx2hth9w5hntncv70kvyygvg0qr4ug32g"

  CODE_ID_ACCOUNT=1767
  CODE_ID_SERVICES_MANAGER=1766
  CODE_ID_REBALANCER=1765
  CODE_ID_ORACLE=1764
  CODE_ID_AUCTION=1763
  CODE_ID_AUCTIONS_MANAGER=1762

  # Contracts addresses for init below
  # ADDR_SERVICES_MANAGER=""
  # ADDR_AUCTIONS_MANAGER=""

  # General data per chain
  WHITELISTED_DENOMS='[\"untrn\"]'
  WHITELISTED_BASE_DENOMS='[\"untrn\"]'
else
  echo "Unknown chain"
fi

EXECUTE_FLAGS="--gas-prices $GAS_PRICES --gas auto --gas-adjustment 1.4 --output json -y"

################################################
################### Account ####################
################################################
if [[ "$COMMAND" == 'account' ]]; then
  if [ -z "$ADDR_SERVICES_MANAGER" ]; then echo "[ERROR] Services manager address is missing for $CHAIN" && exit 1; fi

  init_msg=$(jq -n \
    --arg services_manager "$ADDR_SERVICES_MANAGER" \
    '{
      services_manager: $services_manager
    }')

  $BINARY tx wasm init $CODE_ID_ACCOUNT "$init_msg" --label "Valence account" \
    --admin $OWNER_ADDR --from $OWNER_ADDR $EXECUTE_FLAGS

################################################
############### Services Manager ###############
################################################
elif [[ "$COMMAND" == 'services-manager' ]]; then
  init_msg=$(jq -n \
    '{}')

  $BINARY tx wasm init $CODE_ID_SERVICES_MANAGER "$init_msg" --label "Valence services manager" \
    --admin $OWNER_ADDR --from $OWNER_ADDR $EXECUTE_FLAGS

################################################
############### Services Manager ###############
################################################
elif [[ "$COMMAND" == 'auctions-manager' ]]; then
  init_msg=$(jq -n \
    --arg auction_code_id "$CODE_ID_AUCTION" \
    '{ auction_code_id: $auction_code_id,
       min_auction_amount: [["ujunox", "2000"], ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9", "1000"]]
      }')

  $BINARY tx wasm init $CODE_ID_AUCTIONS_MANAGER "$init_msg" --label "Valence auctions manager" \
    --admin $OWNER_ADDR --from $OWNER_ADDR $EXECUTE_FLAGS

################################################
################## Rebalancer ##################
################################################
elif [[ "$COMMAND" == 'rebalancer' ]]; then
  init_msg=$(jq -n \
    --arg services_manager_addr "$ADDR_SERVICES_MANAGER" \
    --arg auctions_manager_addr "$ADDR_AUCTIONS_MANAGER" \
    --arg whitelist_denom "$WHITELISTED_DENOMS" \
    --arg whitelist_base_denom "$WHITELISTED_BASE_DENOMS" \
    '{services_manager_addr: $services_manager_addr,
      auctions_manager_addr: $auctions_manager_addr,
      cycle_start: "0",
      denom_whitelist: [$whitelist_denom],
      base_denom_whitelist: [$whitelist_base_denom]
      }')

  $BINARY tx wasm init $CODE_ID_REBALANCER "$init_msg" --label "Valence rebalancer" \
    --admin $OWNER_ADDR --from $OWNER_ADDR $EXECUTE_FLAGS

else
  echo "Unknown command"
fi
