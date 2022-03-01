use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, PanicOnDefault, PromiseOrValue};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        total_supply: U128,
    ) -> Self {
        let owner_id = env::current_account_id();
        assert!(!env::state_exists(), "Already initialized");
        let mut this = Self {token: FungibleToken::new(b"t".to_vec())};
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.0);
        this
    }

    pub fn get_some_tokens(&mut self, amount: U128) {
        self.create_account_if_not_exists(&env::predecessor_account_id());
        self.token.internal_transfer(&env::current_account_id(),
                                     &env::predecessor_account_id(),
                                     amount.0,
                                     None);
    }

    // create account and send tokens
    #[payable]
    pub fn transfer_call(&mut self, receiver_id: AccountId, amount: U128) -> PromiseOrValue<U128> {
        self.create_account_if_not_exists(&receiver_id);
        self.token.ft_transfer_call(receiver_id, amount, None, "".parse().unwrap())
    }

    fn create_account_if_not_exists(&mut self, account_id: &AccountId){
        if !self.token.accounts.contains_key(account_id) {
            self.token.internal_register_account(account_id);
        }
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: u128) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: u128) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    #[payable]
    fn ft_on_transfer(&mut self,
                      sender_id: AccountId,
                      amount: U128,
                      msg: String) -> PromiseOrValue<U128> {
        log!("Received transfer. From: {} amount: {} message: {}", sender_id, amount.0, msg);
        self.token.internal_transfer(&env::predecessor_account_id(),
                                     &sender_id,
                                     amount.0,
                                     None);
        return PromiseOrValue::Value(U128::from(0_u128))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new(U128::from(TOTAL_SUPPLY));
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(0)).0, TOTAL_SUPPLY);
    }


    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new( TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.token.internal_register_account(&accounts(2));
        contract.get_some_tokens(U128::from(transfer_amount));
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, 0);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}