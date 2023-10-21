#!/bin/bash

CHAIN=$1
shift
SERVICE_NAME=$1
shift
SERVICE_ADDR=$1
shift

if [[ "$CHAIN" == 'juno' ]]; then
  BINARY="junod"
  GAS_PRICES="0.025ujunox"
  OWNER_ADDR="juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn"

  ADDR_SERVICES_MANAGER="juno1gscdr8zw8njrqfad9m3jgw70s4zumqccka4k6cutlxen0krud08sxlqs9d"
elif [[ "$CHAIN" == 'neutron' || "$CHAIN" == 'ntrn' ]]; then
  BINARY="neutrond"
  GAS_PRICES="0.025ntrn"
  OWNER_ADDR="neutron17s47ltx2hth9w5hntncv70kvyygvg0qr4ug32g"

  # ADDR_SERVICES_MANAGER=""
else
  echo "Unknown chain"
fi

EXECUTE_FLAGS="--gas-prices $GAS_PRICES --gas auto --gas-adjustment 1.4 -y"

execute_msg=$(jq -n \
  --arg service_name "$SERVICE_NAME" \
  --arg service_Addr "$SERVICE_ADDR" \
  '{admin: {
      add_service: {
        name: $service_name,
        addr: $service_Addr
      }
    }}')

$BINARY tx wasm execute $ADDR_SERVICES_MANAGER "$execute_msg" --from $OWNER_ADDR $EXECUTE_FLAGS
