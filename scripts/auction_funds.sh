BINARY="neutrond"
GAS_PRICES="0.075untrn"
EXECUTE_FLAGS="--gas-prices $GAS_PRICES --gas auto --gas-adjustment 1.4 --output json -y"

ADMIN_ADDR="neutron1phx0sz708k3t6xdnyc98hgkyhra4tp44et5s68"
AUCTIONS_MANAGER_ADDR="neutron13exc5wdc7y5qpqazc34djnu934lqvfw2dru30j52ahhjep6jzx8ssjxcyz"

NTRN_AMOUNT="1000001untrn"                                                                #1 NTRN
ATOM_AMOUNT="100001ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"  #0.1 ATOM
USDC_AMOUNT="1000001ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81" #1 USDC
NEWT_AMOUNT="13000001factory/neutron1p8d89wvxyjcnawmgw72klknr3lg9gwwl6ypxda/newt"         #13k NEWT

declare -a NTRN_PAIRS=(
    '["untrn","ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"]'
    '["untrn","ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81"]'
    '["untrn","factory/neutron1p8d89wvxyjcnawmgw72klknr3lg9gwwl6ypxda/newt"]'
)

declare -a ATOM_PAIRS=(
    '["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9","untrn"]'
    '["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9","ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81"]'
    '["ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9","factory/neutron1p8d89wvxyjcnawmgw72klknr3lg9gwwl6ypxda/newt"]'
)

declare -a USDC_PAIRS=(
    '["ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81","ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"]'
    '["ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81","untrn"]'
    '["ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81","factory/neutron1p8d89wvxyjcnawmgw72klknr3lg9gwwl6ypxda/newt"]'
)

declare -a NEWT_PAIRS=(
    '["factory/neutron1p8d89wvxyjcnawmgw72klknr3lg9gwwl6ypxda/newt","ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"]'
    '["factory/neutron1p8d89wvxyjcnawmgw72klknr3lg9gwwl6ypxda/newt","untrn"]'
    '["factory/neutron1p8d89wvxyjcnawmgw72klknr3lg9gwwl6ypxda/newt","ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81"]'
)

for pair in "${NTRN_PAIRS[@]}"; do
    echo "Auctioning $NTRN_AMOUNT for pair: $pair"

    EXECUTE_MSG=$(
        jq -n \
            --argjson pair $pair \
            '{auction_funds:{
                    pair: $pair,
                }}'
    )

    $BINARY tx wasm execute $AUCTIONS_MANAGER_ADDR "$EXECUTE_MSG" --from $ADMIN_ADDR --amount $NTRN_AMOUNT $EXECUTE_FLAGS
    sleep 9
done

for pair in "${ATOM_PAIRS[@]}"; do
    echo "Auctioning $ATOM_AMOUNT for pair: $pair"

    EXECUTE_MSG=$(
        jq -n \
            --argjson pair $pair \
            '{auction_funds:{
                    pair: $pair,
                }}'
    )

    $BINARY tx wasm execute $AUCTIONS_MANAGER_ADDR "$EXECUTE_MSG" --from $ADMIN_ADDR --amount $ATOM_AMOUNT $EXECUTE_FLAGS
    sleep 9
done

for pair in "${USDC_PAIRS[@]}"; do
    echo "Auctioning $USDC_AMOUNT for pair: $pair"

    EXECUTE_MSG=$(
        jq -n \
            --argjson pair $pair \
            '{auction_funds:{
                    pair: $pair,
                }}'
    )

    $BINARY tx wasm execute $AUCTIONS_MANAGER_ADDR "$EXECUTE_MSG" --from $ADMIN_ADDR --amount $USDC_AMOUNT $EXECUTE_FLAGS
    sleep 9
done

for pair in "${NEWT_PAIRS[@]}"; do
    echo "Auctioning $NEWT_AMOUNT for pair: $pair"

    EXECUTE_MSG=$(
        jq -n \
            --argjson pair $pair \
            '{auction_funds:{
                    pair: $pair,
                }}'
    )

    $BINARY tx wasm execute $AUCTIONS_MANAGER_ADDR "$EXECUTE_MSG" --from $ADMIN_ADDR --amount $NEWT_AMOUNT $EXECUTE_FLAGS
    sleep 9
done
