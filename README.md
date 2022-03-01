# Near swap tokens smart-contract

### This contract shows the logic of the exchange of tokens within one account

#### For client contract use signed account id

#### Launch

```
export ID = <your root account ID>

# compile code and build wasm file into "res" folder
./build.sh 

# create account "contract_account.$ID" and deploy smart-contract to this account 
./deploy.sh

# create account $ID for token A and token B
near call contract_account.$ID init_my_account '' --accountId $ID

# get some tokens for $ID account
near call contract_account.$ID get_some_tokens '{"x":"5", "y":"5"}' --accountId $ID

# buy B tokens for five A tokens
near call contract_account.$ID buy_b_tokens '{"sell_amount":"5"}' --accountId $ID

# buy A tokens for seven B tokens
near call contract_account.$ID buy_a_tokens '{"sell_amount":"7"}' --accountId $ID

```

#### Test
```
cargo test
```
