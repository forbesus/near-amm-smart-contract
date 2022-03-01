#!/bin/bash
# near delete token_a.$ID $ID; # delete account if already exists
near create-account token_a.$ID --masterAccount=$ID --initialBalance=20;
NEAR_ENV=testnet near deploy --wasmFile res/ft.wasm --accountId=token_a.$ID
