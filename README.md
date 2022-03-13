# Near AMM contract

#### This contract shows the logic of the exchange of tokens using AMM
#### You can see an example of how contracts work in [simulation tests](tests/sim)

#### First you need to deploy FT tokens (example: deploy_ft_a.sh, deploy_ft_b.sh)

#### Next you need to setup FT and get tokens

```
export ID = <your root account ID>

# Create contract with default meta
near call token_a.$ID new_default_meta '{"owner_id": "token_a.<ID>","total_supply": "1000000"}' --accountId $ID;

# Set storage deposit
near call token_a.$ID storage_deposit '{"account_id": "alice.<ID>"}' --accountId $ID --deposit 1 --gas 25000000000000;
near call token_a.$ID storage_deposit '{"account_id": "amm.<ID>"}' --accountId $ID --deposit 1 --gas 25000000000000;

# Send tokens to Alice
near call token_a.$ID ft_transfer '{"receiver_id": "alice.<ID>", "amount": "500000"}' --accountId token_a.$ID --depositYocto 1;

# For contract B, do the same
```
#### Next, you need to deploy and setup the AMM contract

```
# Init contract 
near call amm.$ID new '{
    "token_a_contract": "token_a.<ID>",
    "token_b_contract": "token_b.<ID>",
    "token_a_metadata": {
                          "spec": "ft-1.0.0",
                          "name": "Token A",
                          "symbol": "A",
                          "icon": null,
                          "reference": null,
                          "reference_hash": null,
                          "decimals": 4
                        },

    "token_b_metadata": {
                          "spec": "ft-1.0.0",
                          "name": "Token B",
                          "symbol": "B",
                          "icon": null,
                          "reference": null,
                          "reference_hash": null,
                          "decimals": 4
                        }
            }' --accountId amm.$ID;


# Set storage deposit to Alice
near call amm.$ID storage_deposit '{"token_name":"token_a.<ID>","account_id": "alice.<ID>"}' --accountId amm.$ID --deposit 1 --gas 25000000000000;
```


For send tokens from FT to AMM use FT.ft_transfer_call

For add tokens to pool use AMM.add_tokens_to_pool

For exclude tokens from pool use AMM.exclude_tokens_from_pool

For swap tokens use AMM.swap

For withdraw tokens use AMM.withdraw_tokens


## Test
```
cargo test --all
```

## Problems
Floating point calculation not implemented. Rounding in Swap does not work correctly
