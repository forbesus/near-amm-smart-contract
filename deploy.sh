#!/bin/bash
#near delete contract_account.$ID $ID; # delete account if already exists
near create-account contract_account.$ID --masterAccount=$ID --initialBalance=50;
NEAR_ENV=testnet near deploy --wasmFile res/contract.wasm --accountId=contract_account.$ID
