use amm::AMMContract;
use ft::FtContractContract as FtContract;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FT_METADATA_SPEC,
};
use near_sdk::serde_json::json;
use near_sdk_sim::{deploy, init_simulator, to_yocto, ContractAccount, UserAccount};

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    FT_WASM_BYTES => "res/ft.wasm",
    AMM_WASM_BYTES => "res/amm.wasm",
}

pub const FT_A_ID: &str = "token_a";
pub const FT_B_ID: &str = "token_b";
pub const AMM_ID: &str = "amm";

// Register the given `user` with FT contract
pub fn register_user(contract_id: &str, user: &near_sdk_sim::UserAccount) {
    user.call(
        contract_id.parse().unwrap(),
        "storage_deposit",
        &json!({
            "account_id": user.account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS / 2,
        near_sdk::env::storage_byte_cost() * 1250, // attached deposit
    )
    .assert_success();
}

pub fn init(
    initial_balance: u128,
) -> (
    UserAccount,
    ContractAccount<FtContract>,
    ContractAccount<FtContract>,
    ContractAccount<AMMContract>,
    UserAccount,
) {
    let root = init_simulator(None);
    let meta = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "FT".to_string(),
        symbol: "EXAMPLE".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 3,
    };

    // Init Token A contract
    let token_a_contract = deploy!(
        contract: FtContract,
        contract_id: FT_A_ID,
        bytes: &FT_WASM_BYTES,
        signer_account: root,
        init_method: new(
            root.account_id(),
            initial_balance.into(),
            meta.clone()
        )
    );

    // Init Token B contract
    let token_b_contract = deploy!(
        contract: FtContract,
        contract_id: FT_B_ID,
        bytes: &FT_WASM_BYTES,
        signer_account: root,
        init_method: new(
            root.account_id(),
            initial_balance.into(),
            meta.clone()
        )
    );
    let alice = root.create_user("alice".parse().unwrap(), to_yocto("100"));
    register_user(FT_A_ID, &alice);
    register_user(FT_B_ID, &alice);

    // Init AMM contract
    let amm_contract = deploy!(
        contract: AMMContract,
        contract_id: AMM_ID,
        bytes: &AMM_WASM_BYTES,
        signer_account: root,
        init_method: new(
            token_a_contract.account_id(),
            token_b_contract.account_id(),
            meta.clone(),
            meta
        )
    );
    register_user(FT_A_ID, &amm_contract.user_account);
    register_user(FT_B_ID, &amm_contract.user_account);
    (
        root,
        token_a_contract,
        token_b_contract,
        amm_contract,
        alice,
    )
}
