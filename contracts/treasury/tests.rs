use super::*;
use crate::treasury::Treasury;
use ink_lang as ink;

/// Test that setting handler works
#[ink::test]
fn set_handler_works() {
    let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

    let mut treasury = Treasury::new(accounts.alice);
    assert_eq!(treasury.nonce(), 0);
    assert_eq!(treasury.handler(), accounts.alice);

    assert_eq!(treasury.set_handler(accounts.bob, 1048), Ok(()));

    assert_eq!(treasury.nonce(), 1048);
    assert_eq!(treasury.handler(), accounts.bob);
}
