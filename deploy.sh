#!/bin/bash
near delete contract_account1.$ID $ID; # delete account if already exists
near create-account contract_account1.$ID --masterAccount=$ID --initialBalance=50;
NEAR_ENV=testnet near deploy --wasmFile res/near_swap_token_contract.wasm --accountId=contract_account1.$ID;
NEAR_ENV=testnet near call token_a.$ID new '{"total_supply": "100"}' --accountId $ID;
NEAR_ENV=testnet near call token_a.$ID add_tokens '{"amount":"50"}' --accountId $ID;