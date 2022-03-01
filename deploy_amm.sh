#!/bin/bash
near delete amm.$ID $ID; # delete account if already exists
near create-account amm.$ID --masterAccount=$ID --initialBalance=20;
NEAR_ENV=testnet near deploy --wasmFile res/amm.wasm --accountId=amm.$ID
