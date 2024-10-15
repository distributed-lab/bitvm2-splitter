use bitcoin::TxIn;
use bitcoin_splitter::{split::intermediate_state::IntermediateState, treepp::Script};

use crate::winternitz::PublicKey;

pub mod disprove_script;
pub mod signing;

#[cfg(test)]
mod tests;

pub struct AssertTransaction<const N: usize> {
    // Inputs which are used in the transaction.
    pub inputs: Vec<TxIn>,
    // Subprograms $f_{i}$ that will be verified in the transaction.
    pub subprograms: [Script; N],
    // Intermidiate states $z_i$.
    pub states: [IntermediateState; N],
    // Winternitz public keys that are used for verification of
    // related to intermidiate states' signatures.
    //
    // In paper specified as $\mathsf{pk}_{z_i}$.
    pub states_pubkeys: [PublicKey; N],
}
