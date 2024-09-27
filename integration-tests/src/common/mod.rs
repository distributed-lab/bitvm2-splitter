use std::collections::HashMap;

use bitcoincore_rpc::{
    bitcoin::{
        address::{NetworkChecked, NetworkUnchecked},
        Address, Amount,
    },
    json::AddressType,
    jsonrpc::{self, error::RpcError},
    Error, RpcApi,
};
use ini::Ini;
use once_cell::sync::Lazy;

/// Store at compile time the configuration file of local Bitcoind
/// node, and parse it at start of the runtime.
pub(crate) static BITCOIN_CONFIG: Lazy<Ini> =
    Lazy::new(|| Ini::load_from_str(include_str!("../../../configs/bitcoind.conf")).unwrap());

/// Bitcoin params client to local node.
///
/// Parameters are read from local ./configs/bitcoind.conf file.
pub(crate) static BITCOIN_CLIENT_PARAMS: Lazy<(String, bitcoincore_rpc::Auth)> = Lazy::new(|| {
    let config = &BITCOIN_CONFIG;

    let regtest_section = config.section(Some("regtest")).unwrap();
    let port = regtest_section.get("rpcport").unwrap();
    let url = format!("http://127.0.0.1:{port}");

    let username = regtest_section.get("rpcuser").unwrap();
    let password = regtest_section.get("rpcpassword").unwrap();

    (
        url,
        bitcoincore_rpc::Auth::UserPass(username.to_owned(), password.to_owned()),
    )
});

/// initialize bitcoin client from params
pub(crate) fn init_bitcoin_client() -> eyre::Result<bitcoincore_rpc::Client> {
    let (mut url, auth) = BITCOIN_CLIENT_PARAMS.clone();

    url.push_str(&format!("/wallet/{}", WALLET_NAME));

    bitcoincore_rpc::Client::new(&url, auth).map_err(Into::into)
}

/// Wallet name which will be used in tests.
pub(crate) const WALLET_NAME: &str = "bitvm2-tests-wallet";

/// Address label which will be used in tests.
pub(crate) const ADDRESS_LABEL: &str = "bitvm2-tests-label";

/// Init wallet if one is not initialized.
pub(crate) fn init_wallet() -> eyre::Result<Address<NetworkChecked>> {
    let client = init_bitcoin_client()?;

    // init wallet
    match client.create_wallet(WALLET_NAME, None, None, None, None) {
        Ok(_) => {}
        Err(Error::JsonRpc(jsonrpc::Error::Rpc(RpcError { code: -4, .. }))) => {}
        Err(err) => return Err(err.into()),
    };

    // Get existing address, create one if is there is none.
    let address = match get_addresses_by_label()? {
        Some(addrs) => addrs.0.into_keys().next().unwrap(),
        None => client.get_new_address(Some(ADDRESS_LABEL), Some(AddressType::Bech32m))?,
    }
    .assume_checked();

    fund_address(&address)?;

    Ok(address)
}

pub(crate) const MIN_REQUIRED_AMOUNT: Amount = Amount::from_sat(1_0000_0000);

/// Fund address with minimum required amount of BTC.
pub(crate) fn fund_address(address: &Address<NetworkChecked>) -> eyre::Result<()> {
    let client = init_bitcoin_client()?;

    // if already has enough, leave
    if client.get_balance(None, None)? >= MIN_REQUIRED_AMOUNT {
        return Ok(());
    }

    // if it's only the fresh instance, generate initial 101 blocks
    if client.get_block_count()? <= 2 {
        client.generate_to_address(101, address)?;
    }

    // otherwise geneate blocks until address would have anough
    for i in 0..101 {
        client.generate_to_address(i, address)?;
        if client.get_balance(None, None)? >= MIN_REQUIRED_AMOUNT {
            return Ok(());
        }
    }

    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(transparent)]
struct GetAddressesByLabel(HashMap<Address<NetworkUnchecked>, serde_json::Value>);

fn get_addresses_by_label() -> eyre::Result<Option<GetAddressesByLabel>> {
    let client = init_bitcoin_client()?;

    match client.call("getaddressesbylabel", &[ADDRESS_LABEL.into()]) {
        Ok(value) => Ok(Some(value)),
        Err(Error::JsonRpc(jsonrpc::Error::Rpc(RpcError { code: -11, .. }))) => Ok(None),
        Err(err) => Err(err.into()),
    }
}
