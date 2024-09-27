use bitcoincore_rpc::RpcApi;

use crate::common::{init_bitcoin_client, init_wallet, MIN_REQUIRED_AMOUNT};

#[test]
fn test_ensure_user_has_min_btc() -> eyre::Result<()> {
    color_eyre::install()?;

    let client = init_bitcoin_client()?;

    let _address = init_wallet()?;
    let balance = client.get_balance(None, None)?;

    assert!(balance > MIN_REQUIRED_AMOUNT, "current balance {}", balance);

    Ok(())
}
