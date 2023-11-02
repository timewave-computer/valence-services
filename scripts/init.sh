#!/bin/bash

CHAIN=$1
shift
COMMAND=$1
shift

if [[ "$CHAIN" == 'juno' ]]; then
  BINARY="junod"
  GAS_PRICES="0.025ujunox"
  OWNER_ADDR="juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn"

  CODE_ID_ACCOUNT=3811
  CODE_ID_SERVICES_MANAGER=3815
  CODE_ID_REBALANCER=3814
  CODE_ID_ORACLE=3813
  CODE_ID_AUCTION=3810
  CODE_ID_AUCTIONS_MANAGER=3812

  # Contracts addresses for init below
  ADDR_SERVICES_MANAGER="juno1h2md5367062ypuv93kpwyu84eaq04xx4lfmqwqp5fkqrwa66pynsk6qmk5"
  ADDR_AUCTIONS_MANAGER="juno1tp2n8fa9848355hfd98lufhm84sudlvnzwvsdsqtlahtsrdtl6astvrz9j"

  # General data per chain
  WHITELISTED_DENOMS='[\"ujunox\", \"factory/juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn/vuusdcx\"]'
  WHITELISTED_BASE_DENOMS='[\"ujunox\", \"factory/juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn/vuusdcx\"]'
elif [[ "$CHAIN" == 'neutron' || "$CHAIN" == 'ntrn' ]]; then
  BINARY="neutrond"
  GAS_PRICES="0.015untrn"
  OWNER_ADDR="neutron1phx0sz708k3t6xdnyc98hgkyhra4tp44et5s68"

  CODE_ID_ACCOUNT=435
  CODE_ID_SERVICES_MANAGER=439
  CODE_ID_REBALANCER=440
  CODE_ID_ORACLE=438
  CODE_ID_AUCTION=436
  CODE_ID_AUCTIONS_MANAGER=437

  # Contracts addresses for init below
  ADDR_SERVICES_MANAGER="neutron1g4ylhl0x2k5gjmd7vhyqv2q7cwhd6gmpwspgktlqcq8s38c7f3gs90rv07"
  ADDR_AUCTIONS_MANAGER="neutron1ufvps7qx984unupds40zg79rmju3cj7pueay30hxkdt6373c8g0q66qq84"

  # General data per chain
  WHITELISTED_DENOMS='["untrn", "ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"]'
  WHITELISTED_BASE_DENOMS='["untrn", "ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"]'
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
    --argjson account_code_id $CODE_ID_ACCOUNT \
    '{"whitelisted_code_ids": [$account_code_id]}')

  $BINARY tx wasm init $CODE_ID_SERVICES_MANAGER "$init_msg" --label "Valence services manager" \
    --admin $OWNER_ADDR --from $OWNER_ADDR $EXECUTE_FLAGS

################################################
############### Auctions Manager ###############
################################################
elif [[ "$COMMAND" == 'auctions-manager' ]]; then
  init_msg=$(jq -n \
    --argjson auction_code_id $CODE_ID_AUCTION \
    '{ auction_code_id: $auction_code_id,
       min_auction_amount: [["untrn", "20000"], ["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9", "10000"]]
      }')

  $BINARY tx wasm init $CODE_ID_AUCTIONS_MANAGER "$init_msg" --label "Valence auctions manager" \
    --admin $OWNER_ADDR --from $OWNER_ADDR $EXECUTE_FLAGS

################################################
################## Rebalancer ##################
################################################
elif [[ "$COMMAND" == 'rebalancer' ]]; then

  if [ -z "$ADDR_SERVICES_MANAGER" ]; then echo "[ERROR] Services manager address is missing for $CHAIN" && exit 1; fi
  if [ -z "$ADDR_AUCTIONS_MANAGER" ]; then echo "[ERROR] Auctions manager address is missing for $CHAIN" && exit 1; fi

  init_msg=$(jq -n \
    --arg services_manager_addr "$ADDR_SERVICES_MANAGER" \
    --arg auctions_manager_addr "$ADDR_AUCTIONS_MANAGER" \
    --argjson whitelist_denom "$WHITELISTED_DENOMS" \
    --argjson whitelist_base_denom "$WHITELISTED_BASE_DENOMS" \
    '{services_manager_addr: $services_manager_addr,
      auctions_manager_addr: $auctions_manager_addr,
      cycle_start: "0",
      denom_whitelist: $whitelist_denom,
      base_denom_whitelist: $whitelist_base_denom,
      cycle_period: 60,
      }')

  $BINARY tx wasm init $CODE_ID_REBALANCER "$init_msg" --label "Valence rebalancer" \
    --admin $OWNER_ADDR --from $OWNER_ADDR $EXECUTE_FLAGS

################################################
#################### Oracle ####################
################################################
elif [[ "$COMMAND" == 'oracle' ]]; then
  if [ -z "$ADDR_AUCTIONS_MANAGER" ]; then echo "[ERROR] Auctions manager address is missing for $CHAIN" && exit 1; fi

  init_msg=$(
    jq -n \
      --arg auctions_manager_addr "$ADDR_AUCTIONS_MANAGER" \
      '{auctions_manager_addr: $auctions_manager_addr}'
  )

  $BINARY tx wasm init $CODE_ID_ORACLE "$init_msg" --label "Valence oracle" \
    --admin $OWNER_ADDR --from $OWNER_ADDR $EXECUTE_FLAGS
else
  echo "Unknown command"
fi
