use std::{fs, str::FromStr as _};

use bitcoin::{
    consensus::{Decodable, Encodable as _},
    io::Cursor,
    key::Secp256k1,
    relative::Height,
    secp256k1::SecretKey,
    Address, Amount, CompressedPublicKey, Network, OutPoint, Transaction, TxOut,
};
use bitcoin_splitter::split::script::{IOPair, SplitableScript};
use bitcoin_testscripts::{
    int_mul_windowed::U254MulScript, square_fibonacci::SquareFibonacciScript, u29mul::U29MulScript,
};
use bitcoincore_rpc::{
    bitcoin::consensus::{Decodable as _, Encodable as _},
    RawTx as _, RpcApi,
};
use bitvm2_core::assert::{AssertTransaction, Options};
use once_cell::sync::Lazy;
use tracing_subscriber::fmt::format;

use crate::common::{init_bitcoin_client, init_wallet};

static SECKEY: Lazy<SecretKey> = Lazy::new(|| {
    "50c8f972285ad27527d79c80fe4df1b63c1192047713438b45758ea4e110a88b"
        .parse()
        .unwrap()
});

macro_rules! hex {
    ($tx:expr) => {{
        let mut buf = Vec::new();
        $tx.consensus_encode(&mut buf).unwrap();
        hex::encode(&buf)
    }};
}

macro_rules! txconv {
    ($tx:expr) => {{
        let mut buf = Vec::new();
        $tx.consensus_encode(&mut buf).unwrap();
        let mut cursor = std::io::Cursor::new(&buf);
        bitcoincore_rpc::bitcoin::Transaction::consensus_decode(&mut cursor)?.raw_hex()
    }};
}

fn test_script_payout_spending<const I: usize, const O: usize, S>() -> eyre::Result<()>
where
    S: SplitableScript<I, O>,
{
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let client = init_bitcoin_client()?;

    let address = init_wallet()?;

    let IOPair { input, .. } = S::generate_invalid_io_pair();

    let ctx = Secp256k1::new();

    let operator_pubkey = SECKEY.public_key(&ctx);
    let operator_xonly = operator_pubkey.x_only_public_key().0;

    let operator_p2wpkh_addr = Address::p2wpkh(
        &CompressedPublicKey::try_from(bitcoin::PublicKey::new(operator_pubkey)).unwrap(),
        Network::Regtest,
    );

    // TODO(Velnbur): fix version of bitcoincorerpc and Bitcoin for this...
    let operator_funding_txid = client.send_to_address(
        &bitcoincore_rpc::bitcoin::Address::from_str(&operator_p2wpkh_addr.to_string())
            .unwrap()
            .assume_checked(),
        bitcoincore_rpc::bitcoin::Amount::from_sat(71_000),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    let tx = client.get_raw_transaction(&operator_funding_txid, None)?;
    let tx = {
        let mut buf = Vec::new();
        tx.consensus_encode(&mut buf).unwrap();
        let mut cursor = Cursor::new(&buf);
        Transaction::consensus_decode(&mut cursor)?
    };

    println!("Txid: {}", operator_funding_txid);
    println!("Funding: {}", hex!(tx));
    client.generate_to_address(6, &address)?;

    // find txout
    let txid = tx.compute_txid();
    let (idx, funding_txout) = tx
        .output
        .into_iter()
        .enumerate()
        .find(|(_idx, out)| out.value == Amount::from_sat(71_000))
        .unwrap();

    let assert_tx = AssertTransaction::<{ I }, { O }, S>::with_options(
        input,
        operator_xonly,
        Amount::from_sat(70_000),
        Options {
            payout_locktime: Height::from(1),
        },
    );

    let atx = assert_tx.clone().spend_p2wpkh_input_tx(
        &ctx,
        &SECKEY,
        funding_txout.clone(),
        OutPoint::new(txid, idx as u32),
    )?;

    println!("Txid: {}", atx.compute_txid());
    // println!("AssertSize: {}", hex!(atx).len());
    println!("Assert: {}", hex!(atx));
    client.send_raw_transaction(txconv!(atx))?;
    client.generate_to_address(1, &address)?;

    let payout_tx = assert_tx.payout_transaction(
        &ctx,
        TxOut {
            value: Amount::from_sat(69_000),
            script_pubkey: funding_txout.script_pubkey,
        },
        atx.compute_txid(),
        &SECKEY,
    )?;

    println!("Txid: {}", payout_tx.compute_txid());
    println!("Payout: {}", hex!(payout_tx));
    client.send_raw_transaction(txconv!(payout_tx))?;
    client.generate_to_address(6, &address)?;

    Ok(())
}

fn test_script_disprove_distorted<const I: usize, const O: usize, S>() -> eyre::Result<()>
where
    S: SplitableScript<I, O>,
{
    color_eyre::install()?;
    let format = format().pretty();
    tracing_subscriber::fmt().event_format(format).init();

    let client = init_bitcoin_client()?;

    let address = init_wallet()?;

    let IOPair { input, .. } = S::generate_invalid_io_pair();

    let ctx = Secp256k1::new();

    let operator_pubkey = SECKEY.public_key(&ctx);
    let operator_xonly = operator_pubkey.x_only_public_key().0;

    let operator_p2wpkh_addr = Address::p2wpkh(
        &CompressedPublicKey::try_from(bitcoin::PublicKey::new(operator_pubkey)).unwrap(),
        Network::Regtest,
    );

    // TODO(Velnbur): fix version of bitcoincorerpc and Bitcoin for this...
    let operator_funding_txid = client.send_to_address(
        &bitcoincore_rpc::bitcoin::Address::from_str(&operator_p2wpkh_addr.to_string())
            .unwrap()
            .assume_checked(),
        bitcoincore_rpc::bitcoin::Amount::from_sat(100_000),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    let tx = client.get_raw_transaction(&operator_funding_txid, None)?;
    let tx = {
        let mut buf = Vec::new();
        tx.consensus_encode(&mut buf).unwrap();
        let mut cursor = Cursor::new(&buf);
        Transaction::consensus_decode(&mut cursor)?
    };

    println!("Txid: {}", operator_funding_txid);
    println!("Funding: {}", hex!(tx));
    client.generate_to_address(6, &address)?;

    // find txout
    let txid = tx.compute_txid();
    let (idx, funding_txout) = tx
        .output
        .into_iter()
        .enumerate()
        .find(|(_idx, out)| out.value == Amount::from_sat(100_000))
        .unwrap();

    let (assert_tx, _distored_idx) = AssertTransaction::<{ I }, { O }, S>::with_options_distorted(
        input,
        operator_xonly,
        Amount::from_sat(99_000),
        Options {
            payout_locktime: Height::from(1),
        },
    );

    let atx = assert_tx.clone().spend_p2wpkh_input_tx(
        &ctx,
        &SECKEY,
        funding_txout.clone(),
        OutPoint::new(txid, idx as u32),
    )?;

    println!("Txid: {}", atx.compute_txid());
    // println!("AssertSize: {}", hex!(atx).len());
    println!("Assert: {}", hex!(atx));
    client.send_raw_transaction(txconv!(atx))?;
    client.generate_to_address(1, &address)?;

    let disprove_txs = assert_tx.clone().disprove_transactions(
        &ctx,
        TxOut {
            value: Amount::from_sat(9_000),
            script_pubkey: funding_txout.script_pubkey,
        },
        atx.compute_txid(),
    )?;

    let number = disprove_txs.len();
    tracing::info!("Number of disprove txs: {}", number);
    for (idx, (_disprove_script, disprove_tx)) in disprove_txs.into_iter().enumerate() {
        let err = client.send_raw_transaction(txconv!(disprove_tx)).err();

        if let Some(err) = err {
            tracing::warn!(reason = %err, "Disprove Tx {idx} rejected...");
            if idx + 1 == number {
                panic!("no dispove transaction worked!");
            }
            continue;
        }

        let hexed = hex!(disprove_tx);
        println!("Txid: {}", disprove_tx.compute_txid());
        println!("DisproveSize: {}", hexed.len() / 2);
        fs::write("./disprove.hex", hexed)?;
        break;
    }

    client.generate_to_address(6, &address)?;

    Ok(())
}

#[test]
#[ignore = "Stack size limit exceeded"]
fn test_u254_mul_disprove() -> eyre::Result<()> {
    test_script_disprove_distorted::<
        { U254MulScript::INPUT_SIZE },
        { U254MulScript::OUTPUT_SIZE },
        U254MulScript,
    >()
}

#[test]
fn test_u254_mul_payout() -> eyre::Result<()> {
    test_script_payout_spending::<
        { U254MulScript::INPUT_SIZE },
        { U254MulScript::OUTPUT_SIZE },
        U254MulScript,
    >()
}

#[test]
fn test_square_fibonachi_disprove() -> eyre::Result<()> {
    test_script_disprove_distorted::<
        { SquareFibonacciScript::<1024>::INPUT_SIZE },
        { SquareFibonacciScript::<1024>::OUTPUT_SIZE },
        SquareFibonacciScript<1024>,
    >()
}

#[test]
fn test_u29mul_disprove() -> eyre::Result<()> {
    test_script_disprove_distorted::<
        { U29MulScript::INPUT_SIZE },
        { U29MulScript::OUTPUT_SIZE },
        U29MulScript,
    >()
}
