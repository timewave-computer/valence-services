ACCOUNT_ADDR=$1
shift

execute_msg=$(jq -n \
  '{
  "register_to_service": {
    "service_name": "rebalancer",
    "data": "ewoiYmFzZV9kZW5vbSI6ICJpYmMvQzRDRkY0NkZENkRFMzVDQTRDRjRDRTAzMUU2NDNDOEZEQzlCQTRCOTlBRTU5OEU5QjBFRDk4RkUzQTIzMTlGOSIsCiJwaWQiOiB7ICJwIjogIjAuMSIsICJpIjogIjAiLCAiZCI6ICIwIiB9LAoidGFyZ2V0X292ZXJyaWRlX3N0cmF0ZWd5IjogInByb3BvcnRpb25hbCIsCiJ0YXJnZXRzIjogWwp7CiJkZW5vbSI6ICJpYmMvQzRDRkY0NkZENkRFMzVDQTRDRjRDRTAzMUU2NDNDOEZEQzlCQTRCOTlBRTU5OEU5QjBFRDk4RkUzQTIzMTlGOSIsCiJicHMiOiA1MDAwCn0sCnsgImRlbm9tIjogInVudHJuIiwgImJwcyI6IDUwMDAgfQpdCn0="
  }
}')

EXECUTE_FLAGS="--gas-prices 0.075untrn --gas auto --gas-adjustment 1.4 --output json -y"

neutrond tx wasm execute $ACCOUNT_ADDR "$execute_msg" --from neutron1phx0sz708k3t6xdnyc98hgkyhra4tp44et5s68 $EXECUTE_FLAGS
