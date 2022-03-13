extern crate core;

use std::cmp::max;
use std::str::FromStr;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement};
use near_sdk::{AccountId, Balance, env, Gas, log, near_bindgen, ONE_YOCTO, PanicOnDefault, Promise, PromiseOrValue, PromiseResult};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env::validator_total_stake;
use near_sdk::ext_contract;
use near_sdk::json_types::U128;

use crate::utils::{add_decimals, calc_dy, remove_decimals};

mod storage;
mod utils;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AMM {
    // tokens A and B (token, metadata, issuer)
    pub tokens: LookupMap<AccountId, (FungibleToken, FungibleTokenMetadata)>,
    // here the proportions of the investment in the pool are stored
    pub token_amm: FungibleToken,
}

fn init_token(account_id: &AccountId, prefix: Vec<u8>) -> FungibleToken {
    let mut a = FungibleToken::new(prefix);
    // a.total_supply = total_supply;
    a.internal_register_account(account_id);
    a
}

// define an interface for callbacks
#[ext_contract(ext_self)]
trait SelfContract {
    fn withdraw_tokens_callback(&mut self, token_name: String, amount: U128);
}

#[ext_contract(ext_ft)]
trait FtContract {
    fn ft_transfer(&self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}


#[near_bindgen]
impl AMM {
    #[init]
    pub fn new(token_a_contract: AccountId,
               token_b_contract: AccountId,
               token_a_metadata: FungibleTokenMetadata,
               token_b_metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let owner_id = env::current_account_id();
        let token_a = init_token(&owner_id,  b"a".to_vec() );
        let token_b = init_token(&owner_id,  b"b".to_vec() );
        let token_amm = init_token(&owner_id,  b"amm".to_vec());
        let mut tokens= LookupMap::new(b"m".to_vec());
        tokens.insert(&token_a_contract, &(token_a, token_a_metadata));
        tokens.insert(&token_b_contract, &(token_b, token_b_metadata));
        Self {
            tokens,
            token_amm,
        }
    }

    pub fn swap(&mut self, buy_token_name: AccountId, sell_token_name: AccountId, sell_amount: U128) -> U128 {
        if buy_token_name.eq(&sell_token_name) {
            panic!("Tokens can't be equals")
        }
        let mut buy_token = self.tokens.get(&buy_token_name).expect("Token not found");
        let mut sell_token = self.tokens.get(&sell_token_name).expect("Token not found");
        let pool_owner_id = &env::current_account_id();
        let user_account_id = &env::predecessor_account_id();

        let x = sell_token.0.internal_unwrap_balance_of(&pool_owner_id);
        let y = buy_token.0.internal_unwrap_balance_of(&pool_owner_id);

        sell_token.0.internal_transfer(&user_account_id, &pool_owner_id, sell_amount.0, None);

        let max_decimals = max(buy_token.1.decimals, sell_token.1.decimals);
        let x = add_decimals(x, max_decimals - sell_token.1.decimals);
        let y = add_decimals(y, max_decimals - buy_token.1.decimals);

        let buy_amount = calc_dy(x, y, sell_amount.0);
        let buy_amount = remove_decimals(buy_amount, max_decimals - buy_token.1.decimals);

        buy_token.0.internal_transfer(&pool_owner_id, &user_account_id, buy_amount, None);

        self.tokens.insert(&buy_token_name, &buy_token);
        self.tokens.insert(&sell_token_name, &sell_token);

        U128::from(buy_amount)

        // swap_tokens(&mut sell_token, &mut buy_token, sell_amount, pool_owner_id, token_owner_id);
    }

    // adding tokens to the liquidity pool. Tokens can only be added in proportion to the amount in the pool
    pub fn add_tokens_to_pool(&mut self, token_a_name: AccountId, token_a_amount: U128,
                              token_b_name: AccountId, token_b_amount: U128) {
        if token_a_name.eq(&token_b_name) {
            panic!("Tokens can't be equals")
        }
        let mut token_a = self.tokens.get(&token_a_name).expect("Token not found");
        let mut token_b = self.tokens.get(&token_b_name).expect("Token not found");

        let pool_owner_id = env::current_account_id();
        let payer_id = env::predecessor_account_id();

        // get current state of pool
        let pool_a_balance = token_a.0.internal_unwrap_balance_of(&pool_owner_id);
        let pool_b_balance = token_b.0.internal_unwrap_balance_of(&pool_owner_id);

        let max_decimals = max(token_a.1.decimals, token_b.1.decimals);
        // we can add tokens to the pool only by proportionally increasing them
         if pool_a_balance * &token_b_amount.0 == pool_b_balance * &token_a_amount.0 {
            token_a.0.internal_transfer(&payer_id,&pool_owner_id, token_a_amount.0, None);
            token_b.0.internal_transfer(&payer_id,&pool_owner_id, token_b_amount.0, None);

            log!("Tokens has been added to liquidity pool");
            let price = add_decimals(token_a_amount.0, max_decimals - token_a.1.decimals)
                + add_decimals(token_b_amount.0, max_decimals - token_a.1.decimals);
            self.token_amm.internal_deposit(&payer_id, price);
            log!("Price {} has been added to account {}", price, &payer_id);

             self.tokens.insert(&token_a_name, &token_a );
             self.tokens.insert(&token_b_name, &token_b );

         } else {
            panic!("incorrect proportions for replenishing the liquidity pool")
        }
    }


    // here we are excluding all tokens of signed account from
    // liquidity pool and return those tokens back to predecessor_account_id
    // in the right proportion
    pub fn exclude_tokens_from_pool(&mut self, token_a_name: AccountId, token_b_name: AccountId) {
        if token_a_name.eq(&token_b_name) {
            panic!("Tokens can't be equals")
        }
        let mut token_a = self.tokens.get(&token_a_name).expect("Token not found");
        let mut token_b = self.tokens.get(&token_b_name).expect("Token not found");


        let pool_owner_id = env::current_account_id();
        let pool_total_a = token_a.0.internal_unwrap_balance_of(&pool_owner_id);
        let pool_total_b = token_b.0.internal_unwrap_balance_of(&pool_owner_id);
        let predecessor_account_id = env::predecessor_account_id();
        // let share_amount = get_share_amount(&self.token_amm, predecessor_account_id.clone());
        // calc all owned tokens from pool
        let a = self.token_amm.total_supply * pool_total_a / self.token_amm.internal_unwrap_balance_of(&predecessor_account_id);
        let b = self.token_amm.total_supply * pool_total_b / self.token_amm.internal_unwrap_balance_of(&predecessor_account_id);
        // clear share value
        self.token_amm.internal_withdraw(&predecessor_account_id, self.token_amm.internal_unwrap_balance_of(&predecessor_account_id));
        // transfer tokens from pool to user wallet
        token_a.0.internal_transfer(&env::current_account_id(), &predecessor_account_id, a, None);
        token_b.0.internal_transfer(&env::current_account_id(), &predecessor_account_id, b, None);
        self.tokens.insert(&token_a_name, &token_a);
        self.tokens.insert(&token_b_name, &token_b);
    }

    pub fn withdraw_tokens(&self,
                       token_name: AccountId,
                       amount: U128) {
        let account_id = env::predecessor_account_id();
        if !self.tokens.contains_key(&token_name) {
            panic!("Token not supported");
        }
        ext_ft::ft_transfer(
            account_id,
            amount,
            None,
            token_name.clone(),
            1,
            Gas::from(5_000_000_000_000)
        ).then(ext_self::withdraw_tokens_callback(
            token_name.to_string(),
            amount,
            env::current_account_id(),
            0,
            Gas::from(5_000_000_000_000)
        ));
    }

    #[payable]
    pub fn withdraw_tokens_callback(&mut self, token_name: AccountId, amount: U128) {
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => "error!".to_string(),
            PromiseResult::Successful(_) => {
                let account_id = env::signer_account_id();
                let mut token = self.tokens.get(&token_name).unwrap();
                token.0.internal_withdraw(&account_id, amount.0);
                self.tokens.insert(&token_name, &token);
                "Ok".to_string()
            }
        };
    }

    pub fn ft_balance_of(&self, token_name: AccountId, account_id: AccountId) -> U128 {
        return if token_name == env::current_account_id() {
            self.token_amm.ft_balance_of(account_id)
        } else {
            self.tokens.get(&token_name).expect("Token not supported").0.ft_balance_of(account_id)
        }
    }

    // Storage implementation

    #[payable]
    pub fn storage_deposit(&mut self, token_name: AccountId, account_id: AccountId, registration_only: Option<bool>) {
        if token_name == env::current_account_id() {
            self.token_amm.storage_deposit(Some(account_id), registration_only);
        } else {
            let mut token = self.tokens.get(&token_name).unwrap();
            token.0.storage_deposit(Some(account_id), registration_only);
            self.tokens.insert(&token_name,&token);
        }
    }

    #[payable]
    fn storage_withdraw(&mut self, token_name: AccountId, amount: Option<U128>) -> StorageBalance {
        if token_name == env::current_account_id() {
            self.token_amm.storage_withdraw(amount)
        } else {
            let mut token = self.tokens.get(&token_name).unwrap();
            let storage_balance = token.0.storage_withdraw(amount);
            self.tokens.insert(&token_name,&token);
            storage_balance
        }
    }

    #[payable]
    fn storage_unregister(&mut self, token_name: AccountId, force: Option<bool>) -> bool {
        if token_name == env::current_account_id() {
            if let Some((_, _)) = self.token_amm.internal_storage_unregister(force) {
                return true
            }
        } else {
            let mut token = self.tokens.get(&token_name).unwrap();
            if let Some((_, _)) = token.0.internal_storage_unregister(force) {
                self.tokens.insert(&token_name,&token);
                return true
            }
        }
        return false
    }

    fn storage_balance_bounds(&self, token_name: AccountId) -> StorageBalanceBounds {
        if token_name == env::current_account_id() {
            self.token_amm.storage_balance_bounds()
        } else {
            let mut token = self.tokens.get(&token_name).unwrap();
            token.0.storage_balance_bounds()
        }
    }

    fn storage_balance_of(&self, token_name: AccountId, account_id: AccountId) -> Option<StorageBalance> {
        if token_name == env::current_account_id() {
            self.token_amm.storage_balance_of(account_id)
        } else {
            let mut token = self.tokens.get(&token_name).unwrap();
            token.0.storage_balance_of(account_id)
        }
    }


}

#[near_bindgen]
impl FungibleTokenReceiver for AMM {
    #[payable]
    fn ft_on_transfer(&mut self,
                      sender_id: AccountId,
                      amount: U128,
                      msg: String) -> PromiseOrValue<U128> {
        let token_name = &env::predecessor_account_id();
        let mut token = self.tokens.get(token_name).expect("Token not supported");
        token.0.internal_deposit(&sender_id, amount.0);
        self.tokens.insert(token_name, &token);
        PromiseOrValue::Value(U128::from(0_u128))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use std::cmp::max;
    use crate::{add_decimals, calc_dy, remove_decimals};

    #[test]
    fn test_add_decimals() {
        let decimals = add_decimals(50, 3);
        assert_eq!(decimals, 50_000);
    }

    #[test]
    fn test_remove_decimals() {
        let decimals = remove_decimals(50000, 3);
        assert_eq!(decimals, 50);
    }


    #[test]
    fn check_calculator() {
        let x = 1_000_000; // 3 numbers float
        let y = 40_000; // 1 number float
        let max_decimals = 3;
        let y = add_decimals(y, max_decimals - 1);
        let dy = calc_dy(x, y, 1_000_000);
        let dy = remove_decimals(dy, max_decimals - 1);
        assert_eq!(dy, 20_000);
    }
}
