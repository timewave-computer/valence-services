# Neutron Contracts

## Code ids

- auctions manager = `1762`
- auction = `1763`
- price oracle = `1764`
- rebalancer = `1765`
- services manager = `1766`
- account = `1767`

# Juno Contracts

## Code ids

- auctions manager = `3812`
- auction = `3810`
- price oracle = `3813`
- rebalancer = `3814`
- services manager = `3815`
- account = `3811`

## Token factory

Token that is created by the token factory is the one that is used in our tests on juno

vuusdcx - `factory/juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn/vuusdcx`

## Owner

The Owner address is the admin of the contracts as well as the account owner.

To add the owner to the keys (in order to use scripts on testnet):

```
junod keys add valence-owner --recover
// Enter mnemonic from below
```

- Owner Juno - `juno17s47ltx2hth9w5hntncv70kvyygvg0qr83zghn`
- Owner Neutron - `neutron17s47ltx2hth9w5hntncv70kvyygvg0qr4ug32g`
- Owner mnemonic - `comfort decrease casual olive mountain joke timber concert leg salt stereo ticket trim plunge matter steak glory above neither hospital agent spoil kick split`

## Addresses

### Juno Contracts

- Services manager - `juno1h2md5367062ypuv93kpwyu84eaq04xx4lfmqwqp5fkqrwa66pynsk6qmk5`
- Auctions manager - `juno1tp2n8fa9848355hfd98lufhm84sudlvnzwvsdsqtlahtsrdtl6astvrz9j`
- Rebalancer - `juno18rpfddza4g3h5a05fzwq6xwepzh2t0twhetly4y5aqjyeh8cjflspa8fqr`
- Account - `juno1rs76w568qe8z4vn9sxch7da84uauul5aek05n29tldmdra3dfk9qrar5ze`
- Oracle - `juno14vgs85az6xlfzkczzq06agk2tv8zkdxqdue4gs08h0f60smu3jjqfryaj2`

#### Auctions

- (ujunox, vuusdcx) - ``
- (vuusdcx, ujunox) - ``

# FIGMA - flow

1. init a services manager
2. init a auctions manager
3. init services (rebalancer)
4. add service to manager
5. init account
6. init auctions
7. init price oracle
8. set initial prices for auctions
