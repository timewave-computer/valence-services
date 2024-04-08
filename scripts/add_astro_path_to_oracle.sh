#!/bin/bash

# DENOMS
ATOM="ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"
NTRN="untrn"
USDC="ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81"
USDC_AXL="ibc/F082B65C88E4B6D5EF1DB243CDA1D331D002759E938A0F5CD3FFDC5D53B3E349"
NEWT="factory/neutron1p8d89wvxyjcnawmgw72klknr3lg9gwwl6ypxda/newt"

# POOLS
ATOM_NTRN_POOL="neutron1e22zh5p8meddxjclevuhjmfj69jxfsa8uu3jvht72rv9d8lkhves6t8veq"     # ATOM <-> NTRN
NTRN_USDC_AXL_POOL="neutron1l3gtxnwjuy65rzk63k352d52ad0f2sh89kgrqwczgt56jc8nmc3qh5kag3" # NTRN <-> USDC.AXL
USDC_AXL_USDC_POOL="neutron1ns2tcunrlrk5yk62fpl74ycazanceyfmmq7dlj6sq8n0rnkuvk7szstkyx" # USDC.AXL <-> USDC
NTRN_NEWT_POOL="neutron14798daa6d6ysq28fjzpg7jkykaseq7tlz6euleeegj02e0a8gkksltdm9l"     # NTRN <-> NEWT

# PATHS
## ATOM / NTRN
ATOM_NTRN_PATH='[{"denom1": "'$ATOM'", "denom2": "'$NTRN'","pool_address": "'$ATOM_NTRN_POOL'"}]'
NTRN_ATOM_PATH='[{"denom1": "'$NTRN'", "denom2": "'$ATOM'","pool_address": "'$ATOM_NTRN_POOL'"}]'

## USDC
ATOM_USDC_PATH='[{"denom1": "'$ATOM'", "denom2": "'$NTRN'","pool_address": "'$ATOM_NTRN_POOL'"},
                {"denom1": "'$NTRN'", "denom2": "'$USDC_AXL'","pool_address": "'$NTRN_USDC_AXL_POOL'"},
                {"denom1": "'$USDC_AXL'", "denom2": "'$USDC'","pool_address": "'$USDC_AXL_USDC_POOL'"}]'

NTRN_USDC_PATH='[{"denom1": "'$NTRN'", "denom2": "'$USDC_AXL'","pool_address": "'$NTRN_USDC_AXL_POOL'"},
                {"denom1": "'$USDC_AXL'", "denom2": "'$USDC'","pool_address": "'$USDC_AXL_USDC_POOL'"}]'

USDC_ATOM_PATH='[{"denom1": "'$USDC'", "denom2": "'$USDC_AXL'","pool_address": "'$USDC_AXL_USDC_POOL'"},
                {"denom1": "'$USDC_AXL'", "denom2": "'$NTRN'","pool_address": "'$NTRN_USDC_AXL_POOL'"},
                {"denom1": "'$NTRN'", "denom2": "'$ATOM'","pool_address": "'$ATOM_NTRN_POOL'"}]'

USDC_NTRN_PATH='[{"denom1": "'$USDC'", "denom2": "'$USDC_AXL'","pool_address": "'$USDC_AXL_USDC_POOL'"},
                {"denom1": "'$USDC_AXL'", "denom2": "'$NTRN'","pool_address": "'$NTRN_USDC_AXL_POOL'"}]'

## NEWT
ATOM_NEWT_PATH='[{"denom1": "'$ATOM'", "denom2": "'$NTRN'","pool_address": "'$ATOM_NTRN_POOL'"},
                {"denom1": "'$NTRN'", "denom2": "'$NEWT'","pool_address": "'$NTRN_NEWT_POOL'"}]'

NTRN_NEWT_PATH='[{"denom1": "'$NTRN'", "denom2": "'$NEWT'","pool_address": "'$NTRN_NEWT_POOL'"}]'

USDC_NEWT_PATH='[{"denom1": "'$USDC'", "denom2": "'$USDC_AXL'","pool_address": "'$USDC_AXL_USDC_POOL'"},
                {"denom1": "'$USDC_AXL'", "denom2": "'$NTRN'","pool_address": "'$NTRN_USDC_AXL_POOL'"},
                {"denom1": "'$NTRN'", "denom2": "'$NEWT'","pool_address": "'$NTRN_NEWT_POOL'"}]'

NEWT_ATOM_PATH='[{"denom1": "'$NEWT'", "denom2": "'$NTRN'","pool_address": "'$NTRN_NEWT_POOL'"},
                {"denom1": "'$NTRN'", "denom2": "'$ATOM'","pool_address": "'$ATOM_NTRN_POOL'"}]'

NEWT_NTRN_PATH='[{"denom1": "'$NEWT'", "denom2": "'$NTRN'","pool_address": "'$NTRN_NEWT_POOL'"}]'

NEWT_USDC_PATH='[{"denom1": "'$NEWT'", "denom2": "'$NTRN'","pool_address": "'$NTRN_NEWT_POOL'"}, 
                {"denom1": "'$NTRN'", "denom2": "'$USDC_AXL'","pool_address": "'$NTRN_USDC_AXL_POOL'"},
                {"denom1": "'$USDC_AXL'", "denom2": "'$USDC'","pool_address": "'$USDC_AXL_USDC_POOL'"}]'

# MSGS
## ATOM / NTRN
ATOM_NTRN_MSG=$(jq -n \
    --argjson pair '["'$ATOM'","'$NTRN'"]' \
    --argjson path "$ATOM_NTRN_PATH" \
    '{pair: $pair, path: $path}')

NTRN_ATOM_MSG=$(jq -n \
    --argjson pair '["'$NTRN'","'$ATOM'"]' \
    --argjson path "$NTRN_ATOM_PATH" \
    '{pair: $pair, path: $path}')

## USDC
ATOM_USDC_MSG=$(jq -n \
    --argjson pair '["'$ATOM'","'$USDC'"]' \
    --argjson path "$ATOM_USDC_PATH" \
    '{pair: $pair, path: $path}')

NTRN_USDC_MSG=$(jq -n \
    --argjson pair '["'$NTRN'","'$USDC'"]' \
    --argjson path "$NTRN_USDC_PATH" \
    '{pair: $pair, path: $path}')

USDC_ATOM_MSG=$(jq -n \
    --argjson pair '["'$USDC'","'$ATOM'"]' \
    --argjson path "$USDC_ATOM_PATH" \
    '{pair: $pair, path: $path}')

USDC_NTRN_MSG=$(jq -n \
    --argjson pair '["'$USDC'","'$NTRN'"]' \
    --argjson path "$USDC_NTRN_PATH" \
    '{pair: $pair, path: $path}')

## NEWT
ATOM_NEWT_MSG=$(jq -n \
    --argjson pair '["'$ATOM'","'$NEWT'"]' \
    --argjson path "$ATOM_NEWT_PATH" \
    '{pair: $pair, path: $path}')

NTRN_NEWT_MSG=$(jq -n \
    --argjson pair '["'$NTRN'","'$NEWT'"]' \
    --argjson path "$NTRN_NEWT_PATH" \
    '{pair: $pair, path: $path}')

USDC_NEWT_MSG=$(jq -n \
    --argjson pair '["'$USDC'","'$NEWT'"]' \
    --argjson path "$USDC_NEWT_PATH" \
    '{pair: $pair, path: $path}')

NEWT_ATOM_MSG=$(jq -n \
    --argjson pair '["'$NEWT'","'$ATOM'"]' \
    --argjson path "$NEWT_ATOM_PATH" \
    '{pair: $pair, path: $path}')

NEWT_NTRN_MSG=$(jq -n \
    --argjson pair '["'$NEWT'","'$NTRN'"]' \
    --argjson path "$NEWT_NTRN_PATH" \
    '{pair: $pair, path: $path}')

NEWT_USDC_MSG=$(jq -n \
    --argjson pair '["'$NEWT'","'$USDC'"]' \
    --argjson path "$NEWT_USDC_PATH" \
    '{pair: $pair, path: $path}')

# TODO: Modify the path here
execute_msg=$(jq -n \
    --argjson msg "$NEWT_USDC_MSG" \
    '{add_astro_path: $msg}')

ORACLE_ADDR="neutron1s8uqyh0mmh8g66s2dectf56c08y6fvusp39undp8kf4v678ededsy6tstf"
EXECUTE_FLAGS="--gas-prices 0.075untrn --gas auto --gas-adjustment 1.4 --output json -y"

neutrond tx wasm execute $ORACLE_ADDR "$execute_msg" --from neutron1phx0sz708k3t6xdnyc98hgkyhra4tp44et5s68 $EXECUTE_FLAGS
