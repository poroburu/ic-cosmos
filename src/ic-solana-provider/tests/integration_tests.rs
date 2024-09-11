mod common;

use {
    crate::common::{init, random_principal},
    candid::encode_one,
    pocket_ic::PocketIc,
};

#[test]
fn test_get_balance() {
    std::env::set_var("IC_SOLANA_PROVIDER_PATH", "ic-solana-provider.wasm.gz");

    let ic = PocketIc::new();

    let canister_id = init(&ic);

    let res = ic
        .update_call(
            canister_id,
            random_principal(),
            "sol_getBalance",
            encode_one("AAAAUrmaZWvna6vHndc5LoVWUBmnj9sjxnvPz5U3qZGY").unwrap(),
        )
        .unwrap();

    println!("{:#?}", res);
}
