WHITELISTED_BASE_DENOMS='[\"untrn\", \"ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9\"]'

execute_msg=$(jq -n \
  --arg whitelist_denom "$WHITELISTED_BASE_DENOMS" \
  '{admin: {update_base_denom_whitelist: {
      to_add: [],
      to_remove: [$whitelist_denom],
    }}}')

EXECUTE_FLAGS="--gas-prices 0.015untrn --gas auto --gas-adjustment 1.4 --output json -y"

echo neutrond tx wasm execute neutron1jreurhf7g43l0zdxu26fa8aahnjxyng8sjh5vvwjpn4lucwq8tsq7jxl5t "$execute_msg" --from neutron1phx0sz708k3t6xdnyc98hgkyhra4tp44et5s68 $EXECUTE_FLAGS
