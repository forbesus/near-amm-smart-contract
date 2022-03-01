use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::{AccountId, env, log, near_bindgen, PromiseOrValue, PanicOnDefault};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_sdk::env::panic_str;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Swap {
    // tokens A and B
    pub token_a: FungibleToken,
    pub token_b: FungibleToken,
    // here the proportions of the investment in the pool are stored
    pub token_ab: FungibleToken,
    // contract name of token A issuer
    pub a_token_contract: AccountId,
    // contract name of token B issuer
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

    // Buy B tokens using liquidity pool
    // sell_amount - amount of A token which we want to sell
    pub fn buy_b_tokens(&mut self, sell_amount: U128) {
        swap_tokens(&mut self.token_a,
                    &mut self.token_b,
                    sell_amount,
                    &env::current_account_id(),
                    &env::predecessor_account_id());
        log!("Success! Your balance: A[{}] B[{}]",
             self.token_a.internal_unwrap_balance_of(&env::predecessor_account_id()),
             self.token_b.internal_unwrap_balance_of(&env::predecessor_account_id()))
    }

    // Buy A tokens using liquidity pool
    // sell_amount - amount of B token which we want to sell
    pub fn buy_a_tokens(&mut self, sell_amount: U128) {
        swap_tokens(&mut self.token_b,
                    &mut self.token_a,
                    sell_amount,
                    &env::current_account_id(),
                    &env::predecessor_account_id());
        log!("Success! Your balance: A[{}] B[{}]",
             self.token_a.internal_unwrap_balance_of(&env::predecessor_account_id()),
             self.token_b.internal_unwrap_balance_of(&env::predecessor_account_id()))
    }

    // adding tokens to the liquidity pool. Tokens can only be added in proportion to the amount in the pool
    pub fn add_to_liquidity_pool(&mut self, token_a_amount: U128, token_b_amount: U128) {
        // get current state of pool
        let pool_a_balance = self.token_a.internal_unwrap_balance_of(&env::current_account_id());
        let pool_b_balance = self.token_b.internal_unwrap_balance_of(&env::current_account_id());
        // we can add tokens to the pool only by proportionally increasing them
         if pool_a_balance * &token_b_amount.0 == pool_b_balance * &token_a_amount.0 {
            self.token_a.internal_transfer(&env::predecessor_account_id(),&env::current_account_id(), token_a_amount.0, None);
            self.token_b.internal_transfer(&env::predecessor_account_id(),&env::current_account_id(), token_b_amount.0, None);
            log!("Tokens has been added to liquidity pool");
            create_account_if_not_exists(&mut self.token_ab, &env::predecessor_account_id());
            let price = &token_a_amount.0 * &token_b_amount.0;
            self.token_ab.internal_deposit(&env::predecessor_account_id(), price);
            log!("Price {} has been added to account {}", price, &env::predecessor_account_id());
        } else {
            panic_str("incorrect proportions for replenishing the liquidity pool")
        }
    }

    pub fn print_state(&self) {
        let a_tokens = self.token_a.ft_balance_of(env::predecessor_account_id()).0;
        let b_tokens = self.token_b.ft_balance_of(env::predecessor_account_id()).0;
        log!("Your tokens: A:{} B:{}", a_tokens, b_tokens);
        let a_tokens = self.token_a.internal_unwrap_balance_of(&env::current_account_id()) as f64;
        let b_tokens = self.token_b.internal_unwrap_balance_of(&env::current_account_id()) as f64;
        log!("Total pool size: A:{} B:{}", a_tokens, b_tokens);
    }

    // here we are excluding all tokens of signed account from
    // liquidity pool and return those tokens back to predecessor_account_id
    // in the right proportion
    pub fn exclude_tokens_from_liquidity_pool(&mut self) {
        let a = self.token_a.internal_unwrap_balance_of(&env::current_account_id()) as f64;
        let b = self.token_b.internal_unwrap_balance_of(&env::current_account_id()) as f64;
        let predecessor_account_id = env::predecessor_account_id();
        let share_amount = get_share_amount(&self.token_ab, predecessor_account_id.clone());
        // calc token amount to exclude in depend on proportion
        let a = (a * &share_amount) as u128;
        let b = (b * &share_amount) as u128;
        self.token_ab.internal_withdraw(&predecessor_account_id,
                                        self.token_ab.internal_unwrap_balance_of(&predecessor_account_id));
        self.token_a.internal_transfer(&env::current_account_id(), &predecessor_account_id, a, None);
        self.token_b.internal_transfer(&env::current_account_id(), &predecessor_account_id, b, None);

    }

    #[payable]
    pub fn withdraw_a_tokens(&mut self, amount: U128) {
        withdraw_tokens(&mut self.token_a, &self.a_token_contract, amount)
    }

    #[payable]
    pub fn withdraw_b_tokens(&mut self, amount: U128) {
        withdraw_tokens(&mut self.token_b, &self.b_token_contract, amount)
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Swap {
    #[payable]
    fn ft_on_transfer(&mut self,
                      sender_id: AccountId,
                      amount: U128,
                      msg: String) -> PromiseOrValue<U128> {
        log!("Received transfer. From: {} amount: {} message: {}", sender_id, amount.0, msg);

        // check tokens issuer
        if env::predecessor_account_id() == self.b_token_contract {
            log!("Deposit {} B tokens to {}", amount.0, sender_id);
            create_account_if_not_exists(&mut self.token_b, &sender_id);
            self.token_b.internal_deposit(&sender_id, amount.0)
        } else if env::predecessor_account_id() == self.a_token_contract {
            log!("Deposit {} A tokens to {}", amount.0, sender_id);
            create_account_if_not_exists(&mut self.token_a, &sender_id);
            self.token_a.internal_deposit(&sender_id, amount.0)
        } else {
            panic_str("Wrong contract")
        }
        return PromiseOrValue::Value(U128::from(0_u128))
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Swap {
    #[payable]
    fn ft_resolve_transfer(&mut self, sender_id: AccountId, receiver_id: AccountId, amount: U128) -> U128 {
        log!("Execute transfer resolver. Sender: {} receiver: {} amount: {}", sender_id, receiver_id, amount.0);
        U128::from(0)
    }
}

fn swap_tokens(from: &mut FungibleToken, to: &mut FungibleToken, amount: U128,
               pool_owner_id: &AccountId,
               account_id: &AccountId) {
    let x = from.internal_unwrap_balance_of(&pool_owner_id);
    let y = to.internal_unwrap_balance_of(&pool_owner_id);
    from.internal_transfer(&account_id, &pool_owner_id, amount.0, None);
    let buy_amount = calc_dy(&x, &y, &amount.0);
    to.internal_transfer(&pool_owner_id, &account_id, buy_amount, None);
}

fn create_account_if_not_exists(token: &mut FungibleToken, account_id: &AccountId){
    if !token.accounts.contains_key(account_id) {
        log!{"Create account {}", account_id}
        token.internal_register_account(account_id);
    }
}

// here we get the share of our investment in the pool
fn get_share_amount(token: &FungibleToken, account_id: AccountId) -> f64 {
    let balance = token.ft_balance_of(account_id).0 as f64;
    let total_supply: f64 = token.ft_total_supply().0 as f64;
    let res: f64 = balance / total_supply;
    res
}

pub fn calc_dy(x: &u128, y: &u128, dx: &u128) -> u128 {
    y - (x * y + ((x + dx) >> 1)) / (x + dx)
}

fn withdraw_tokens(token: &mut FungibleToken, receiver_id: &AccountId, amount: U128) {
    create_account_if_not_exists(token, receiver_id);
    token.ft_transfer_call(receiver_id.clone(), amount, None, "".parse().unwrap());
    token.internal_withdraw(receiver_id, amount.0.clone());
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{testing_env, VMContext};
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::json_types::U128;
    use near_sdk::AccountId;

    use super::*;

    fn get_owner_context() -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("client".parse().unwrap())
            .current_account_id("owner".parse().unwrap())
            .attached_deposit(100000)
            .is_view(false)
            .build()
    }

    #[test]
    fn test_pool_add_and_exclude() {
        let client = get_owner_context();
        testing_env!(client.clone());
        let predecessor_account_id = AccountId::from(client.predecessor_account_id);
        let current_account_id =  AccountId::from(client.current_account_id);
        let a_token_contract: AccountId = "a.testnet".parse().unwrap();
        let b_token_contract: AccountId = "b.testnet".parse().unwrap();

        let mut contract = Swap::new(a_token_contract, b_token_contract);

        contract.token_a.internal_register_account(&predecessor_account_id);
        contract.token_b.internal_register_account(&predecessor_account_id);

        contract.token_a.internal_deposit(&predecessor_account_id, 10);
        contract.token_b.internal_deposit(&predecessor_account_id, 10);

        contract.add_to_liquidity_pool(U128::from(1), U128::from(2));

        assert_eq!(contract.token_a.ft_balance_of(predecessor_account_id.clone()).0, 9);
        assert_eq!(contract.token_b.ft_balance_of(predecessor_account_id.clone()).0, 8);

        assert_eq!(contract.token_a.ft_balance_of(current_account_id.clone()).0, 1);
        assert_eq!(contract.token_b.ft_balance_of(current_account_id.clone()).0, 2);

        contract.add_to_liquidity_pool(U128::from(2), U128::from(4));

        contract.exclude_tokens_from_liquidity_pool();

        assert_eq!(contract.token_a.ft_balance_of(predecessor_account_id.clone()).0, 10);
        assert_eq!(contract.token_b.ft_balance_of(predecessor_account_id.clone()).0, 10);

        assert_eq!(contract.token_a.ft_balance_of(current_account_id.clone()).0, 0);
        assert_eq!(contract.token_b.ft_balance_of(current_account_id.clone()).0, 0);
    }

    #[test]
    #[should_panic]
    fn test_add_to_poll_wrong_proportions() {
        let client = get_owner_context();
        testing_env!(client.clone());
        let predecessor_account_id = AccountId::from(client.predecessor_account_id);
        let a_token_contract: AccountId = "a.testnet".parse().unwrap();
        let b_token_contract: AccountId = "b.testnet".parse().unwrap();

        let mut contract = Swap::new(a_token_contract, b_token_contract);

        contract.token_a.internal_register_account(&predecessor_account_id);
        contract.token_b.internal_register_account(&predecessor_account_id);

        contract.token_a.internal_deposit(&predecessor_account_id, 10);
        contract.token_b.internal_deposit(&predecessor_account_id, 10);

        contract.add_to_liquidity_pool(U128::from(2), U128::from(2));
        contract.add_to_liquidity_pool(U128::from(1), U128::from(2));
    }

    #[test]
    fn test_swap_tokens() {
        let pool_owner = accounts(1);
        let client_account = accounts(2);
        let mut from = FungibleToken::new(1);
        from.internal_register_account(&pool_owner);
        from.internal_register_account(&client_account);

        from.internal_deposit(&pool_owner, 10);
        from.internal_deposit(&client_account, 1);

        let mut to = FungibleToken::new(2);
        to.internal_register_account(&pool_owner);
        to.internal_register_account(&client_account);
        to.internal_deposit(&pool_owner, 20);

        // there are 10 A tokens and 20 B tokens in Pool
        assert_eq!(from.ft_balance_of(pool_owner.clone()).0, 10);
        assert_eq!(to.ft_balance_of(pool_owner.clone()).0, 20);

        // we have 1 A token and 0 B tokens
        // we sell 1 A tokens
        swap_tokens(&mut from,
                    &mut to,
                    U128::from(1),
                    &pool_owner,
                    &client_account);

        // there are 11 A tokens and 18 B tokens in Pool
        assert_eq!(from.ft_balance_of(pool_owner.clone()).0, 11);
        assert_eq!(to.ft_balance_of(pool_owner.clone()).0, 18);

        // now we have 0 A tokens and 2 B tokens
        assert_eq!(from.ft_balance_of(client_account.clone()).0, 0);
        assert_eq!(to.ft_balance_of(client_account.clone()).0, 2);
    }

    #[test]
    fn test_calc_share_amount() {
        let alice = accounts(1);
        let bob = accounts(2);
        let mut token = FungibleToken::new(1);
        token.internal_register_account(&alice);
        token.internal_register_account(&bob);
        token.internal_deposit(&alice, 10);
        token.internal_deposit(&bob, 40);
        let alice_share = get_share_amount(&token, alice);
        let bob_share = get_share_amount(&token, bob);

        assert_eq!(alice_share, 0.2);
        assert_eq!(bob_share, 0.8);
    }

    #[test]
    fn check_calculator() {
        assert_eq!(calc_dy(&0, &2000, &1000), 2000);
        // Check round. if round is not working result should be 3
        assert_eq!(calc_dy(&0, &4, &10), 4);
    }
}
