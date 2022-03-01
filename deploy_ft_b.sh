#!/bin/bash
# near delete token_b.$ID $ID; # delete account if already exists
near create-account token_b.$ID --masterAccount=$ID --initialBalance=20;
NEAR_ENV=testnet near deploy --wasmFile res/ft.wasm --accountId=token_b.$ID
