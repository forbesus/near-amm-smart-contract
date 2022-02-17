use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::{AccountId, Balance, env, log, near_bindgen};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Swap {
    token_a: FungibleToken,
    token_b: FungibleToken,
}

impl Default for Swap {
    fn default() -> Self {
        let token_a = init_wallets(env::current_account_id(), U128::from(10), 1);
        let token_b = init_wallets(env::current_account_id(), U128::from(20), 2);
        Self {token_a, token_b }
    }
}

fn init_wallets(account_id: AccountId, balance: U128, prefix: u8) -> FungibleToken {
    let mut a = FungibleToken::new(prefix);
    a.total_supply = balance.0;
    a.accounts.insert(&account_id, &balance.0);
    a
}

#[near_bindgen]
impl Swap {
    pub fn get_balance_a(&self, account_id: &AccountId) -> Balance {
        self.token_a.accounts.get(account_id).expect("Account not found")
    }

    pub fn get_balance_b(&self, account_id: &AccountId) -> Balance {
        self.token_b.accounts.get(account_id).expect("Account not found")
    }

    pub fn get_total_a_tokens(&self) -> Balance {
        self.get_balance_a(&env::current_account_id())
    }

    pub fn get_total_b_tokens(&self) -> Balance {
        self.get_balance_b(&env::current_account_id())
    }

    pub fn init_my_account(&mut self) {
        let account_id = env::signer_account_id();
        let init_balance = 0;
        self.token_a.accounts.insert(&account_id, &init_balance);
        self.token_b.accounts.insert(&account_id, &init_balance);
        log!("Success! Your balance: A[{}] B[{}]", self.get_balance_a(&account_id), self.get_balance_b(&account_id))
    }

    pub fn get_some_tokens(&mut self, x: U128, y: U128) {
        let account_id = env::signer_account_id();
        let owner_id = env::current_account_id();
        self.token_a.internal_transfer(&owner_id, &account_id,x.0, None);
        self.token_b.internal_transfer(&owner_id, &account_id,y.0, None);
        log!("Success! Your balance: A[{}] B[{}]", self.get_balance_a(&account_id), self.get_balance_b(&account_id))
    }

    pub fn buy_b_tokens(&mut self, sell_amount: U128) {
        let account_id = env::signer_account_id();
        let owner_id = env::current_account_id();
        let x = self.get_total_a_tokens();
        let y = self.get_total_b_tokens();
        self.token_a.internal_transfer(&account_id, &owner_id, sell_amount.0, None);
        let dy = calc_dy(x, y, sell_amount.0);
        self.token_b.internal_transfer(&owner_id, &account_id, dy, None);
        log!("Success! Your balance: A[{}] B[{}]", self.get_balance_a(&account_id), self.get_balance_b(&account_id))
    }

    pub fn buy_a_tokens(&mut self, sell_amount: U128) {
        let account_id = env::signer_account_id();
        let owner_id = env::current_account_id();
        let y = self.get_total_a_tokens();
        let x = self.get_total_b_tokens();
        self.token_b.internal_transfer(&account_id, &owner_id, sell_amount.0, None);
        let buy_amount = calc_dy(x, y, sell_amount.0);
        self.token_a.internal_transfer(&owner_id, &account_id, buy_amount, None);
        log!("Success! Your balance: A[{}] B[{}]", self.get_balance_a(&account_id), self.get_balance_b(&account_id))
    }
}

fn calc_dy(x: u128, y: u128, dx: u128) -> u128 {
    y - (x * y + ((x + dx) >> 1)) / (x + dx)
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{testing_env, VMContext};
    use near_sdk::test_utils::{VMContextBuilder};

    use super::*;

    fn get_owner_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("client".parse().unwrap())
            .current_account_id("owner".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    fn test_full() {
        let context = get_owner_context(false);
        testing_env!(context.clone());
        let signer_account_id =  AccountId::from(context.signer_account_id);

        // init wallets
        // token a = 10
        // token b = 20

        let mut contract = Swap::default();

        // check init
        assert_eq!(contract.token_a.total_supply, 10);
        assert_eq!(contract.token_b.total_supply, 20);

        // check method
        assert_eq!(contract.get_total_a_tokens(), 10);
        assert_eq!(contract.get_total_b_tokens(), 20);


        // init client account
        contract.init_my_account();

        // get some tokens for client account for trade
        contract.get_some_tokens(U128::from(5), U128::from(5));

        // check client accaunt balance
        assert_eq!(contract.get_balance_a(&signer_account_id), 5);
        assert_eq!(contract.get_balance_b(&signer_account_id), 5);

        // sell five A tokens and receive 7 B tokens
        contract.buy_b_tokens(U128::from(5));
        assert_eq!(contract.get_balance_a(&signer_account_id), 0);
        assert_eq!(contract.get_balance_b(&signer_account_id), 12);


        // return 7 B tokens and receive 5 A tokens
        contract.buy_a_tokens(U128::from(7));
        assert_eq!(contract.get_balance_a(&signer_account_id), 5);
        assert_eq!(contract.get_balance_b(&signer_account_id), 5);

    }
    #[test]
    fn check_calculator() {
        assert_eq!(calc_dy(0, 2000, 1000), 2000);
        // Check round. if round is not working result should be 3
        assert_eq!(calc_dy(0, 4, 10), 4);
    }
}
