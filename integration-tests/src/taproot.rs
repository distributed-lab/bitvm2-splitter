use bitcoincore_rpc::bitcoin::{
    key::KeyPair,
    opcodes::all::{OP_CHECKSIG, OP_DUP, OP_EQUALVERIFY, OP_HASH160},
    script::Builder,
    secp256k1::Scalar,
    ScriptBuf,
};
use bitcoincore_rpc::{
    bitcoin::{
        absolute::LockTime,
        hashes::{hash160::Hash as Hash160, Hash as _},
        key::Secp256k1,
        secp256k1::{Message, SecretKey},
        sighash::{Prevouts, SighashCache, TapSighashType},
        taproot::{LeafVersion, TaprootBuilder},
        Address, Amount, Network, OutPoint, PublicKey, Sequence, Transaction, TxIn, TxOut, Witness,
    },
    RawTx,
};
// use bitcoin_splitter::treepp::*;
use bitcoincore_rpc::RpcApi;
use once_cell::sync::Lazy;

use crate::common::{init_bitcoin_client, init_wallet};

static PUBKEY: Lazy<PublicKey> = Lazy::new(|| {
    "021e37ec522cb319c66e1a21077a2ba05c070efa5c018d5bc8d002250f5ca0c7dc"
        .parse()
        .unwrap()
});

static SECKEY: Lazy<SecretKey> = Lazy::new(|| {
    "50c8f972285ad27527d79c80fe4df1b63c1192047713438b45758ea4e110a88b"
        .parse()
        .unwrap()
});

fn p2pkh_script(pubkey: PublicKey) -> ScriptBuf {
    Builder::new()
        .push_opcode(OP_DUP)
        .push_opcode(OP_HASH160)
        .push_slice(Hash160::hash(&pubkey.inner.x_only_public_key().0.serialize()).as_byte_array())
        .push_opcode(OP_EQUALVERIFY)
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

#[test]
fn test_simple_singature_script() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let secp_ctx = Secp256k1::new();

    let tweak = Scalar::ONE;
    let pubkey2 = PUBKEY.inner.add_exp_tweak(&secp_ctx, &tweak).unwrap();

    let script1 = p2pkh_script(*PUBKEY);
    let script2 = p2pkh_script(PublicKey::new(pubkey2));

    // FIXME(Velnbur): actually use unspendable key
    let unspendable_internal_key = PUBKEY.inner.x_only_public_key().0;
    // let taproot_spending_info =
    //     TaprootBuilder::with_huffman_tree([(4, script1.clone()), (6, script2.clone())])?
    //         .finalize(&secp_ctx, unspendable_internal_key)
    //         .unwrap();
    let taproot_spending_info =
        TaprootBuilder::new()
            .add_leaf(1, script1.clone())?
            .add_leaf(1, script2.clone())?
            .finalize(&secp_ctx, unspendable_internal_key)
            .unwrap();    

    let sender_address =
        Address::p2tr_tweaked(taproot_spending_info.output_key(), Network::Regtest);

    println!("{}", taproot_spending_info.merkle_root().unwrap());

    let client = init_bitcoin_client()?;
    let address = init_wallet()?;

    let txid = client.send_to_address(
        &sender_address,
        Amount::from_sat(10_000),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    client.generate_to_address(6, &address)?;
    println!("transaction to spend: {txid}");

    let prev_tx = client.get_raw_transaction(&txid, None)?;

    let mut tx = Transaction {
        version: 1,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::new(txid, 0),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ZERO,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: 7000,
            script_pubkey: sender_address.script_pubkey(),
        }],
    };

    let leaf_hash = script1.tapscript_leaf_hash();
    let sighash = SighashCache::new(&tx).taproot_script_spend_signature_hash(
        0,
        &Prevouts::All(&[&prev_tx.output[0]]),
        leaf_hash,
        TapSighashType::Default,
    )?;

    let signature = secp_ctx.sign_schnorr(
        &Message::from_slice(sighash.as_byte_array().as_slice())?,
        &KeyPair::from_secret_key(&secp_ctx, &SECKEY),
    );

    let control_block = &taproot_spending_info
        .control_block(&(script1.clone(), LeafVersion::TapScript))
        .unwrap();

    assert!(control_block.verify_taproot_commitment(
        &secp_ctx,
        taproot_spending_info.output_key().into(),
        &script1
    ));

    let mut witness = Witness::new();
    witness.push(signature.as_ref());
    witness.push(PUBKEY.inner.x_only_public_key().0.serialize());
    witness.push(script1.as_bytes());
    witness.push(control_block.serialize());

    tx.input[0].witness = witness;

    println!("spending transaction hex:\n {}", tx.raw_hex());
    let txid = client.send_raw_transaction(&tx)?;
    println!("spending transaction id: {txid}");

    Ok(())
}

// fn new_unspendable_internal_key(_secp_ctx: &Secp256k1<All>) -> Result<XOnlyPublicKey, eyre::Error> {
//     // let tweak = Scalar::ONE;

//     let h = Sha256::hash(&GENERATOR_X);

//     let mut buf = [0u8; 64];
//     buf[0..32].copy_from_slice(h.as_byte_array().as_slice());

//     let ffi_key = unsafe {
//         bitcoincore_rpc::bitcoin::secp256k1::ffi::XOnlyPublicKey::from_array_unchecked(buf)
//     };

//     let unspendable_internal_key = XOnlyPublicKey::from(ffi_key);

//     Ok(unspendable_internal_key)
// }
