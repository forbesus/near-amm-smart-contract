# Near AMM swap tokens

### This contract shows the logic of the exchange of tokens using AMM

### To exchange tokens, you must first issue them. To do this, you need to deploy two contracts that are engaged in issuing tokens

```
export ID = <your root account ID>

# compile code and build WASM files into "res" folder
./build.sh

# deploy smart contract for issue A tokens
./deploy_ft_a.sh

# deploy smart contract for issue B tokens
./deploy_ft_b.sh

# deploy AMM smart contract for exchange tokens
./deploy_amm.sh

```
### Next, you need to initialize the AMM contract

```
near call amm.$ID new '{"a_contract":"<ISSUER_A_TOKENS_CONTRACT>", "b_contract": "<ISSUER_B_TOKENS_CONTRACT>"}' --accountId $ID;

```
### Next, you need to issue tokens and send it to AMM contract

```
# issue 100 A-tokens
near call token_a.$ID new '{"total_supply": "100"}' --accountId $ID;

# get 50 tokens to account
near call token_a.$ID get_some_tokens '{"amount":"50"}' --accountId $ID;

# send 10 tokens to AMM-contract
near call token_a.$ID transfer_call '{"receiver_id": "<AMM CONNTRACT>", "amount": "10"}' --accountId $ID --amount 0.000000000000000000000001 --gas 300000000000000;


# The same operation must be repeated for the contract B


# issue 100 B-tokens
near call token_b.$ID new '{"total_supply": "100"}' --accountId $ID;

# get 50 tokens to account
near call token_b.$ID get_some_tokens '{"amount":"50"}' --accountId $ID;

# send 10 tokens to AMM-contract
near call token_b.$ID transfer_call '{"receiver_id": "<AMM CONNTRACT>", "amount": "10"}' --accountId $ID --amount 0.000000000000000000000001 --gas 300000000000000;

```

#### Now AMM contract have 10 A tokens and 10 B tokens

#### For show current state of AMM contract call:
```
near call amm.$ID print_state '{}' --accountId $ID;
# Log: Your tokens: A:10 B:10
# Log: Total pool size: A:0 B:0
```

#### To add tokens to the pool you need to execute this command
```
near call amm.$ID add_to_liquidity_pool '{"token_a_amount":"<A_TOKENS_AMOUNT>", "token_b_amount": "<B_TOKENS_AMOUNT>"}' --accountId $ID;
```

#### To exclude tokens from the pool

```
near call amm.$ID exclude_tokens_from_liquidity_pool '{}' --accountId $ID;
```
#### To exchange tokens using a pool (it must not be empty)

```
# A to B
near call amm.$ID buy_b_tokens '{"sell_amount":"<A_TOKENS_AMOUNT>"}' --accountId $ID;

# B to A
near call amm.$ID buy_a_tokens '{"sell_amount":"<B_TOKENS_AMOUNT>"}' --accountId $ID;
```
#### To withdraw tokens from the AMM contract back to token issuing contracts

```
NEAR_ENV=testnet near call amm.$ID withdraw_a_tokens '{"amount":"<AMOUNT_TO_WITHDRAW>"}' --accountId $ID --amount 0.000000000000000000000001 --gas 300000000000000;
NEAR_ENV=testnet near call amm.$ID withdraw_b_tokens '{"amount":"<AMOUNT_TO_WITHDRAW>"}' --accountId $ID --amount 0.000000000000000000000001 --gas 300000000000000;
```

## Test
```
cargo test --all
```
