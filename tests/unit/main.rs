
#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{testing_env, VMContext};
    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::json_types::U128;
    use near_swap_token_contract::calc_dy;
    use near_swap_token_contract::Swap;
    use near_sdk::{AccountId, Balance, env, log, near_bindgen, PromiseOrValue};

    use super::*;
    use near_sdk::serde::de::value::U16Deserializer;

    fn get_owner_context() -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("client".parse().unwrap())
            .current_account_id("owner".parse().unwrap())
            .attached_deposit(50)
            .is_view(false)
            .build()
    }

    #[test]
    fn test_full() {
        let mut first_client_context = get_owner_context();
        testing_env!(first_client_context.clone());
        let signer_account_id =  AccountId::from(first_client_context.signer_account_id);
        let current_account_id =  AccountId::from(first_client_context.current_account_id);

        let mut contract = Swap::default();
        // check initial balance
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&current_account_id), 0);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&current_account_id), 0);

        //  client account for tokens A and B
        contract.create_account();

        // deposit 50 tokens
        contract.deposit_a_tokens();
        contract.deposit_b_tokens();

        contract.add_to_liquidity_pool(U128::from(10), U128::from(20));

        // check client account balance
        contract.buy_b_tokens(U128::from(5));
        // sold five A-tokens
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 35);
        // bought seven B-tokens
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 37);

        // sell seven B tokens and receive 5 A tokens
        contract.buy_a_tokens(U128::from(7));
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 40);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 30);
    }

    #[test]
    fn test_add_and_remove_from_liquidity_pool() {
        let mut first_client_context = get_owner_context();
        testing_env!(first_client_context.clone());
        let signer_account_id =  AccountId::from(first_client_context.signer_account_id);
        let current_account_id =  AccountId::from(first_client_context.current_account_id);
        let mut contract = Swap::default();

        // check method
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&current_account_id), 0);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&current_account_id), 0);

        // init client account
        contract.create_account();

        // deposit 50 tokens
        contract.deposit_a_tokens();
        contract.deposit_b_tokens();

        assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 50);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 50);

        contract.add_to_liquidity_pool(U128::from(10), U128::from(10));

        // append tokens to pool
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&current_account_id), 10);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&current_account_id), 10);
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 40);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 40);
        // check ab balance
        assert_eq!(contract.token_ab.internal_unwrap_balance_of(&signer_account_id), 100);

        // append more tokens to pool with good proportions
        contract.add_to_liquidity_pool(U128::from(5), U128::from(5));
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&current_account_id), 15);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&current_account_id), 15);
        // check ab balance
        assert_eq!(contract.token_ab.internal_unwrap_balance_of(&signer_account_id), 125);

        // exclude tokens from liquidity pool and return to accounts
        contract.exclude_tokens_from_liquidity_pool();
        assert_eq!(contract.token_ab.internal_unwrap_balance_of(&signer_account_id), 0);

        // check accounts
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 50);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 50);


        // append more tokens to pool with bad proportions

        // // check client accaunt balance
        // assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 5);
        // assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 5);
        //
        // // sell five A tokens and receive 7 B tokens
        // contract.buy_b_tokens(U128::from(5));
        // assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 0);
        // assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 12);


        // return 7 B tokens and receive 5 A tokens
        // contract.buy_a_tokens(U128::from(7));
        // assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 5);
        // assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 5);

    }


    #[test]
    #[should_panic]
    fn test_pool_invalid_proportion() {
        let mut first_client_context = get_owner_context();
        testing_env!(first_client_context.clone());
        let signer_account_id = AccountId::from(first_client_context.signer_account_id);
        let current_account_id =  AccountId::from(first_client_context.current_account_id);
        let mut contract = Swap::default();

        // check method
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&current_account_id), 0);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&current_account_id), 0);
        // init client account
        contract.create_account();
        // deposit 50 tokens
        contract.deposit_a_tokens();
        contract.deposit_b_tokens();

        assert_eq!(contract.token_a.internal_unwrap_balance_of(&signer_account_id), 50);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&signer_account_id), 50);

        contract.add_to_liquidity_pool(U128::from(10), U128::from(10));

        // append tokens to pool
        assert_eq!(contract.token_a.internal_unwrap_balance_of(&current_account_id), 10);
        assert_eq!(contract.token_b.internal_unwrap_balance_of(&current_account_id), 10);
        // append more tokens to pool with bad proportions
        contract.add_to_liquidity_pool(U128::from(1), U128::from(2));
    }

    #[test]
    fn check_calculator() {
        assert_eq!(calc_dy(0, 2000, 1000), 2000);
        // Check round. if round is not working result should be 3
        assert_eq!(calc_dy(0, 4, 10), 4);
    }
}
