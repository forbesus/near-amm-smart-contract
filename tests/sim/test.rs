use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{call, DEFAULT_GAS, to_yocto, transaction::ExecutionStatus, view};

use crate::utils::{AMM_ID, FT_A_ID, FT_B_ID, init, register_user};

#[test]
fn simulate_total_supply() {
    let initial_balance = to_yocto("100");
    let (_, ft, _, _, _) = init(initial_balance);
    let total_supply: U128 = view!(ft.ft_total_supply()).unwrap_json();
    assert_eq!(initial_balance, total_supply.0);
}


#[test]
fn simulate_simple_transfer() {
    let transfer_amount = to_yocto("100");
    let initial_balance = to_yocto("100000");
    let (root, ft, _, _, alice) = init(initial_balance);

    // Transfer from root to alice.
    // Uses default gas amount, `near_sdk_sim::DEFAULT_GAS`
    call!(
        root,
        ft.ft_transfer(alice.account_id(), transfer_amount.into(), None),
        deposit = 1
    ).assert_success();

    let root_balance: U128 = view!(ft.ft_balance_of(root.account_id())).unwrap_json();
    let alice_balance: U128 = view!(ft.ft_balance_of(alice.account_id())).unwrap_json();
    assert_eq!(initial_balance - transfer_amount, root_balance.0);
    assert_eq!(transfer_amount, alice_balance.0);
}

#[test]
fn send_tokens_to_amm_and_withdraw() {
    let transfer_to_alice_amount_a = 100_000_u128;
    let transfer_to_amm_amount_a = 50_000_u128;

    let transfer_to_alice_amount_b = 200_000_u128;
    let transfer_to_amm_amount_b = 100_000_u128;

    let initial_balance = 1_000_000_u128;
    let (root, ft_a, ft_b, amm, alice) = init(initial_balance);

    // Transfer from root to alice at FT.
    call!(
        root,
        ft_a.ft_transfer(alice.account_id(), transfer_to_alice_amount_a.into(), None),
        deposit = 1
    ).assert_success();

    call!(
        root,
        ft_b.ft_transfer(alice.account_id(), transfer_to_alice_amount_b.into(), None),
        deposit = 1
    ).assert_success();

    // Open storage in AMM for Alice

    call!(root, amm.storage_deposit(ft_a.account_id(), alice.account_id(), None), deposit = near_sdk::env::storage_byte_cost() * 125).assert_success();
    call!(root, amm.storage_deposit(ft_b.account_id(), alice.account_id(), None), deposit = near_sdk::env::storage_byte_cost() * 125).assert_success();


    // Send A-tokens to AMM
    call!(
        alice,
        ft_a.ft_transfer_call(AMM_ID.parse().unwrap(), transfer_to_amm_amount_a.into(), None, "".to_string()),
        deposit = 1
    ).assert_success();

    // Send B-tokens to AMM
    call!(
        alice,
        ft_b.ft_transfer_call(AMM_ID.parse().unwrap(), transfer_to_amm_amount_b.into(), None, "".to_string()),
        deposit = 1
    ).assert_success();

    // Check FT root balance without sent tokens

    let root_balance_a: U128 = view!(ft_a.ft_balance_of(root.account_id())).unwrap_json();
    assert_eq!(initial_balance - transfer_to_alice_amount_a, root_balance_a.0);

    let root_balance_b: U128 = view!(ft_b.ft_balance_of(root.account_id())).unwrap_json();
    assert_eq!(initial_balance - transfer_to_alice_amount_b, root_balance_b.0);

    // Check alice balance in FT
    let alice_balance_ft_a: U128 = view!(ft_a.ft_balance_of(alice.account_id())).unwrap_json();
    let alice_balance_ft_b: U128 = view!(ft_b.ft_balance_of(alice.account_id())).unwrap_json();

    assert_eq!(transfer_to_alice_amount_a - transfer_to_amm_amount_a, alice_balance_ft_a.0);
    assert_eq!(transfer_to_alice_amount_b - transfer_to_amm_amount_b, alice_balance_ft_b.0);

    // Check Alice Balance in AMM
    let alice_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();

    assert_eq!(transfer_to_amm_amount_a, alice_balance_amm_a.0);
    assert_eq!(transfer_to_amm_amount_b, alice_balance_amm_b.0);


    // withdraw all tokens back
    call!(
        alice,
        amm.withdraw_tokens(ft_a.account_id(), alice_balance_amm_a),
        gas = 300000000000000
    ).assert_success();

    call!(
        alice,
        amm.withdraw_tokens(ft_b.account_id(), alice_balance_amm_b),
        gas = 300000000000000
    ).assert_success();


    // Check alice balance in FT
    let alice_balance_ft_a: U128 = view!(ft_a.ft_balance_of(alice.account_id())).unwrap_json();
    let alice_balance_ft_b: U128 = view!(ft_b.ft_balance_of(alice.account_id())).unwrap_json();

    assert_eq!(alice_balance_ft_a.0, transfer_to_alice_amount_a);
    assert_eq!(alice_balance_ft_b.0, transfer_to_alice_amount_b);

    // Check Alice Balance in AMM (must be zero)
    let alice_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();

    assert_eq!(alice_balance_amm_a.0, 0);
    assert_eq!(alice_balance_amm_b.0, 0);
}


#[test]
fn test_add_and_exclude_from_pool() {
    let transfer_to_alice_amount_a = 100_000_u128;
    let transfer_to_amm_amount_a = 50_000_u128;

    let transfer_to_alice_amount_b = 200_000_u128;
    let transfer_to_amm_amount_b = 100_000_u128;

    let initial_balance = 1_000_000_u128;
    let (root, ft_a, ft_b, amm, alice) = init(initial_balance);

    // Transfer from root to alice at FT.
    call!(
        root,
        ft_a.ft_transfer(alice.account_id(), transfer_to_alice_amount_a.into(), None),
        deposit = 1
    ).assert_success();

    call!(
        root,
        ft_b.ft_transfer(alice.account_id(), transfer_to_alice_amount_b.into(), None),
        deposit = 1
    ).assert_success();

    // Open storage in AMM for Alice


    call!(root, amm.storage_deposit(ft_a.account_id(), alice.account_id(), None), deposit = near_sdk::env::storage_byte_cost() * 125).assert_success();
    call!(root, amm.storage_deposit(ft_b.account_id(), alice.account_id(), None), deposit = near_sdk::env::storage_byte_cost() * 125).assert_success();

    // Send A-tokens to AMM
    call!(
        alice,
        ft_a.ft_transfer_call(AMM_ID.parse().unwrap(), transfer_to_amm_amount_a.into(), None, "".to_string()),
        deposit = 1
    ).assert_success();

    // Send B-tokens to AMM
    call!(
        alice,
        ft_b.ft_transfer_call(AMM_ID.parse().unwrap(), transfer_to_amm_amount_b.into(), None, "".to_string()),
        deposit = 1
    ).assert_success();


    call!(root, amm.storage_deposit(amm.account_id(), alice.account_id(), None), deposit = near_sdk::env::storage_byte_cost() * 250).assert_success();

    // Add tokens to pool
    let send_a_tokens_to_pool = 10_000_u128;
    let send_b_tokens_to_pool = 20_000_u128;
    let alice_balance_amm_a_before: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b_before: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();
    call!(
        alice,
        amm.add_tokens_to_pool(ft_a.account_id(), send_a_tokens_to_pool.into(),  ft_b.account_id(), send_b_tokens_to_pool.into())
    ).assert_success();

    let alice_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_amm: U128 = view!(amm.ft_balance_of(amm.account_id(), alice.account_id())).unwrap_json();

    let owner_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), amm.account_id())).unwrap_json();
    let owner_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), amm.account_id())).unwrap_json();

    assert_eq!(alice_balance_amm_amm.0, send_a_tokens_to_pool + send_b_tokens_to_pool);
    assert_eq!(alice_balance_amm_a.0, alice_balance_amm_a_before.0 - send_a_tokens_to_pool);
    assert_eq!(alice_balance_amm_b.0, alice_balance_amm_b_before.0 - send_b_tokens_to_pool);

    assert_eq!(owner_balance_amm_a.0, send_a_tokens_to_pool);
    assert_eq!(owner_balance_amm_b.0, send_b_tokens_to_pool);

    call!(
        alice,
        amm.exclude_tokens_from_pool(ft_a.account_id(),ft_b.account_id())
    ).assert_success();

    let alice_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_amm: U128 = view!(amm.ft_balance_of(amm.account_id(), alice.account_id())).unwrap_json();

    let owner_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), amm.account_id())).unwrap_json();
    let owner_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), amm.account_id())).unwrap_json();

    assert_eq!(alice_balance_amm_amm.0, 0);
    assert_eq!(alice_balance_amm_a.0, alice_balance_amm_a_before.0);
    assert_eq!(alice_balance_amm_b.0, alice_balance_amm_b_before.0);

    assert_eq!(owner_balance_amm_a.0, 0);
    assert_eq!(owner_balance_amm_b.0, 0);
}


#[test]
fn test_swap_tokens() {
    let transfer_to_alice_amount_a = 100_000_u128;
    let transfer_to_amm_amount_a = 50_000_u128;

    let transfer_to_alice_amount_b = 200_000_u128;
    let transfer_to_amm_amount_b = 100_000_u128;

    let initial_balance = 1_000_000_u128;
    let (root, ft_a, ft_b, amm, alice) = init(initial_balance);

    // Transfer from root to alice at FT.
    call!(
        root,
        ft_a.ft_transfer(alice.account_id(), transfer_to_alice_amount_a.into(), None),
        deposit = 1
    ).assert_success();

    call!(
        root,
        ft_b.ft_transfer(alice.account_id(), transfer_to_alice_amount_b.into(), None),
        deposit = 1
    ).assert_success();

    // Open storage in AMM for Alice


    call!(root, amm.storage_deposit(ft_a.account_id(), alice.account_id(), None), deposit = near_sdk::env::storage_byte_cost() * 125).assert_success();
    call!(root, amm.storage_deposit(ft_b.account_id(), alice.account_id(), None), deposit = near_sdk::env::storage_byte_cost() * 125).assert_success();

    // Send A-tokens to AMM
    call!(
        alice,
        ft_a.ft_transfer_call(AMM_ID.parse().unwrap(), transfer_to_amm_amount_a.into(), None, "".to_string()),
        deposit = 1
    ).assert_success();

    // Send B-tokens to AMM
    call!(
        alice,
        ft_b.ft_transfer_call(AMM_ID.parse().unwrap(), transfer_to_amm_amount_b.into(), None, "".to_string()),
        deposit = 1
    ).assert_success();


    call!(root, amm.storage_deposit(amm.account_id(), alice.account_id(), None), deposit = near_sdk::env::storage_byte_cost() * 250).assert_success();

    // Add tokens to pool
    let send_a_tokens_to_pool = 30_000_u128;
    let send_b_tokens_to_pool = 10_000_u128;
    let alice_balance_amm_a_before: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b_before: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();
    call!(
        alice,
        amm.add_tokens_to_pool(ft_a.account_id(), send_a_tokens_to_pool.into(),  ft_b.account_id(), send_b_tokens_to_pool.into())
    ).assert_success();

    let alice_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_amm: U128 = view!(amm.ft_balance_of(amm.account_id(), alice.account_id())).unwrap_json();
    let owner_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), amm.account_id())).unwrap_json();
    let owner_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), amm.account_id())).unwrap_json();

    assert_eq!(alice_balance_amm_amm.0, send_a_tokens_to_pool + send_b_tokens_to_pool);
    assert_eq!(alice_balance_amm_a.0, alice_balance_amm_a_before.0 - send_a_tokens_to_pool);
    assert_eq!(alice_balance_amm_b.0, alice_balance_amm_b_before.0 - send_b_tokens_to_pool);
    assert_eq!(owner_balance_amm_a.0, send_a_tokens_to_pool);
    assert_eq!(owner_balance_amm_b.0, send_b_tokens_to_pool);

    // Swap tokens

    let sell_token = ft_a.account_id();
    let buy_token = ft_b.account_id();
    let sell_token_amount = 10_000_u128;

    let alice_balance_amm_a_prev: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b_prev: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_amm_prev: U128 = view!(amm.ft_balance_of(amm.account_id(), alice.account_id())).unwrap_json();
    let owner_balance_amm_a_prev: U128 = view!(amm.ft_balance_of(ft_a.account_id(), amm.account_id())).unwrap_json();
    let owner_balance_amm_b_prev: U128 = view!(amm.ft_balance_of(ft_b.account_id(), amm.account_id())).unwrap_json();

    let outcome = call!(
        alice,
        amm.swap(buy_token, sell_token, sell_token_amount.into())
    );
    outcome.assert_success();
    let buy_amount: U128 = outcome.unwrap_json();

    let alice_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), alice.account_id())).unwrap_json();
    let alice_balance_amm_amm: U128 = view!(amm.ft_balance_of(amm.account_id(), alice.account_id())).unwrap_json();
    let owner_balance_amm_a: U128 = view!(amm.ft_balance_of(ft_a.account_id(), amm.account_id())).unwrap_json();
    let owner_balance_amm_b: U128 = view!(amm.ft_balance_of(ft_b.account_id(), amm.account_id())).unwrap_json();

    assert_eq!(alice_balance_amm_a.0, alice_balance_amm_a_prev.0 - sell_token_amount);
    assert_eq!(alice_balance_amm_b.0, alice_balance_amm_b_prev.0 + buy_amount.0);
    assert_eq!(alice_balance_amm_amm.0, alice_balance_amm_amm_prev.0);
    assert_eq!(owner_balance_amm_a.0, owner_balance_amm_a_prev.0 + sell_token_amount);
    assert_eq!(owner_balance_amm_b.0, owner_balance_amm_b_prev.0 - buy_amount.0);
}