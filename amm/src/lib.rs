use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::{AccountId, Balance, env, log, near_bindgen, PromiseOrValue, PanicOnDefault};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_sdk::env::panic;
use near_sdk::collections::{LookupMap};
use near_contract_standards::non_fungible_token::refund_deposit_to_account;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Swap {
    pub token_a: FungibleToken,
    pub token_b: FungibleToken,
    pub token_ab: FungibleToken,
    pub a_token_contract: AccountId,
    pub b_token_contract: AccountId
}

fn init_token(account_id: AccountId, balance: u128, prefix: u8) -> FungibleToken {
    let mut a = FungibleToken::new(prefix);
    a.accounts.insert(&account_id, &balance);
    a
}

#[near_bindgen]
impl Swap {

    #[init]
    pub fn new(a_contract: AccountId, b_contract: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let token_a = init_token(env::current_account_id(), 0, 1);
        let token_b = init_token(env::current_account_id(), 0, 2);
        let token_ab = init_token(env::current_account_id(), 0, 3);
        Self {token_a, token_b, token_ab, a_token_contract: a_contract, b_token_contract: b_contract }
     }

    // initialize client account for two tokens A and B
    pub fn create_account(&mut self) {
        self.token_a.internal_register_account(&env::signer_account_id());
        self.token_b.internal_register_account(&env::signer_account_id());
    }

    // Buy B tokens in liquidity pool
    // sell_amount - amount of A token which we want to sell
    pub fn buy_b_tokens(&mut self, sell_amount: U128) {
        let account_id = env::signer_account_id();
        let owner_id = env::current_account_id();
        let x = self.token_a.internal_unwrap_balance_of(&env::current_account_id());
        let y = self.token_b.internal_unwrap_balance_of(&env::current_account_id());
        self.token_a.internal_transfer(&account_id, &owner_id, sell_amount.0, None);
        let dy = calc_dy(x, y, sell_amount.0);
        self.token_b.internal_transfer(&owner_id, &account_id, dy, None);
        log!("Success! Your balance: A[{}] B[{}]",
             self.token_a.internal_unwrap_balance_of(&account_id),
             self.token_b.internal_unwrap_balance_of(&account_id))    }

    // Buy A tokens in liquidity pool
    // sell_amount - amount of B token which we want to sell
    pub fn buy_a_tokens(&mut self, sell_amount: U128) {
        let account_id = env::signer_account_id();
        let owner_id = env::current_account_id();
        let y = self.token_a.internal_unwrap_balance_of(&env::current_account_id());
        let x = self.token_b.internal_unwrap_balance_of(&env::current_account_id());
        self.token_b.internal_transfer(&account_id, &owner_id, sell_amount.0, None);
        let buy_amount = calc_dy(x, y, sell_amount.0);
        self.token_a.internal_transfer(&owner_id, &account_id, buy_amount, None);
        log!("Success! Your balance: A[{}] B[{}]",
             self.token_a.internal_unwrap_balance_of(&account_id),
             self.token_b.internal_unwrap_balance_of(&account_id))
    }


    // we can add tokens to the liquidity pool only 50/50 at the cost of A and B
    pub fn add_to_liquidity_pool(&mut self, token_a_amount: U128, token_b_amount: U128) {
        // get current state of pool
        let pool_a_balance = self.token_a.internal_unwrap_balance_of(&env::current_account_id());
        let pool_b_balance = self.token_b.internal_unwrap_balance_of(&env::current_account_id());
        // we can add tokens to the pool only by proportionally increasing them
        if pool_a_balance * &token_b_amount.0 == pool_b_balance * &token_a_amount.0 {
            self.token_a.internal_transfer(&env::signer_account_id(),&env::current_account_id(), token_a_amount.0, None);
            self.token_b.internal_transfer(&env::signer_account_id(),&env::current_account_id(), token_b_amount.0, None);
            log!("Tokens has been added to liquidity pool");

            // check existing of AB tokens
            if !self.token_ab.accounts.contains_key(&env::signer_account_id()) {
                self.token_ab.internal_register_account(&env::signer_account_id());
            }
            let price = &token_a_amount.0 * &token_b_amount.0;
            self.token_ab.internal_deposit(&env::signer_account_id(), price);
            log!("Price {} has been added to account {}", price, &env::signer_account_id());
        } else {
            panic!("incorrect proportions for replenishing the liquidity pool")
        }
    }


    // here we can exclude all tokens of signed account from liquidity pool
    pub fn exclude_tokens_from_liquidity_pool(&mut self) {
        let a = self.token_a.internal_unwrap_balance_of(&env::current_account_id()) as f64;
        let b = self.token_b.internal_unwrap_balance_of(&env::current_account_id()) as f64;
        let signer = env::signer_account_id();
        let share_amount = self.get_share_amount(&signer);
        let a = (a * &share_amount) as u128;
        let b = (b * &share_amount) as u128;
        self.token_ab.internal_withdraw(&signer, self.token_ab.internal_unwrap_balance_of(&signer));
        self.token_a.internal_deposit(&signer, a);
        self.token_b.internal_deposit(&signer, b);

    }

    #[payable]
    pub fn withdraw_a_tokens(&mut self, amount: U128) {
        create_account_if_not_exists(&mut self.token_a, &self.a_token_contract);
        self.token_a.ft_transfer_call(self.a_token_contract.clone(), amount, None, "".parse().unwrap());
        self.token_a.internal_withdraw(&self.a_token_contract, amount.0.clone())
    }

    #[payable]
    pub fn withdraw_b_tokens(&mut self, amount: U128) {
        create_account_if_not_exists(&mut self.token_b, &self.b_token_contract);
        self.token_b.ft_transfer_call(self.b_token_contract.clone(), amount, None, "".parse().unwrap());
        self.token_b.internal_withdraw(&self.b_token_contract, amount.0.clone())
    }

    // here we get the share of our investment in the pool
    #[private]
    fn get_share_amount(&self, account_id: &AccountId) -> f64 {
        let balance = self.token_ab.internal_unwrap_balance_of(&account_id) as f64;
        let total_supply: f64 = self.token_ab.ft_total_supply().0 as f64;
        let res: f64 = balance / total_supply;
        res
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Swap {
    #[payable]
    fn ft_on_transfer(&mut self,
                      sender_id: AccountId,
                      amount: U128,
                      msg: String) -> PromiseOrValue<U128> {
        // check tokens issuer
        if env::predecessor_account_id() == self.b_token_contract {
            log!("Deposit 1 {} tokens to {}", amount.0, sender_id);
            create_account_if_not_exists(&mut self.token_b, &sender_id);
            self.token_b.internal_deposit(&sender_id, amount.0)
        } else if env::predecessor_account_id() == self.a_token_contract {
            log!("Deposit 2 {} tokens to {}", amount.0, sender_id);
            create_account_if_not_exists(&mut self.token_a, &sender_id);
            self.token_a.internal_deposit(&sender_id, amount.0)
        } else {
            panic!("Wrong contract")

        }

        return PromiseOrValue::Value(U128::from(0_u128))
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Swap {
    #[payable]
    fn ft_resolve_transfer(&mut self, sender_id: AccountId, receiver_id: AccountId, amount: U128) -> U128 {
        log!("{} tokens sent from {} to {}", amount.0, sender_id, receiver_id);
        U128::from(0)
    }
}

fn create_account_if_not_exists(token: &mut FungibleToken, account_id: &AccountId){
    if !token.accounts.contains_key(account_id) {
        log!{"Create account {}", account_id}
        token.internal_register_account(account_id);
    }
}

pub fn calc_dy(x: u128, y: u128, dx: u128) -> u128 {
    y - (x * y + ((x + dx) >> 1)) / (x + dx)
}
