use super::signing::SignedIntermediateState;
use crate::{treepp::*, utils::OP_LONGNOTEQUAL};

use bitcoin_splitter::split::{
    core::SplitType, intermediate_state::IntermediateState, script::SplitableScript,
};

/// Script letting challengers spend the **Assert** transaction
/// output if the operator computated substates incorrectly.
///
/// This a typed version of [`Script`] can be easily converted into it.
///
/// The script structure in general is simple:
/// ## Witness:
/// ```no_run
/// { Enc(z[i+1]) and Sig[i+1] } // Zipped
/// { Enc(z[i]) and Sig[i] }     // Zipped
/// ```
///
/// ## Script:
/// ```no_run
/// { pk[i] }                // { Zip(Enc(z[i+1]), Sig[i+1]), Zip(Enc(z[i]), Sig[i]), pk[i] }
/// { OP_WINTERNITZVERIFY }  // { Zip(Enc(z[i+1]), Sig[i+1]), Enc(z[i]) }
/// { OP_RESTORE }           // { Zip(Enc(z[i+1]), Sig[i+1]), z[i] }
/// { OP_TOALTSTACK }        // { Zip(Enc(z[i+1]), Sig[i+1]) }
/// { pk[i+1] }              // { Zip(Enc(z[i+1]), Sig[i+1]), pk[i+1] }
/// { OP_WINTERNITZVERIFY }  // { Enc(z[i+1]) }
/// { OP_RESTORE }           // { z[i+1] }
/// { OP_FROMALTSTACK }      // { z[i+1] z[i] }
/// { fn[i] }                // { z[i+1] fn[i](z[i]) }
/// { OP_EQUAL }             // { z[i+1] == fn[i](z[i]) }
/// { OP_NOT }               // { z[i+1] != fn[i](z[i]) }
/// ```
#[derive(Debug, Clone)]
pub struct DisproveScript {
    pub script_witness: Script,
    pub script_pubkey: Script,
}

impl DisproveScript {
    /// Given the previous and current states, and the function that was executed,
    /// creates a new DisproveScript according to the BitVM2 protocol.
    pub fn new(from: &IntermediateState, to: &IntermediateState, function: &Script) -> Self {
        // Step 1.
        // First, we sign the states
        let from_signed = SignedIntermediateState::sign(from);
        let to_signed = SignedIntermediateState::sign(to);

        // Step 2.
        // Now, we form the witness script. Just pushing all
        // signatures + messages to the witness script
        let script_witness = script! {
            { from_signed.witness_script() } // Zipped Enc(z[i]) and Sig[i]
            { to_signed.witness_script() }   // Zipped Enc(z[i+1]) and Sig[i+1]
        };

        // Step 3.
        // Now, we form the script pubkey
        let script_pubkey = script! {
            // 1. Public key + verification of "to" state
            { to_signed.verification_script_toaltstack() } // This leaves z[i+1] in the altstack
            { from_signed.verification_script() } // This leaves z[i].mainstack in the mainstack, while (z[i+1], z[i].altstack) is still in the altstack

            // 2. Applying function and popping "to" state
            { function.clone() } // This leaves f[i](z[i]).mainstack in the mainstack and { z[i+1].altstack, f[i](z[i]).altstack } in the altstack
            for _ in 0..to_signed.altstack.len() {
                OP_FROMALTSTACK
            }
            { to_signed.verification_script_fromaltstack() } // This leaves z[i+1].mainstack and f[i](z[i]).mainstack in the mainstack, while f[i](z[i]).altstack and z[i+1].alstack is in the altstack

            // At tbis point, our stack consists of:
            // { f[i](z[i]).mainstack, f[i](z[i]).altstack, z[i+1].mainstack }
            // while the altstack has z[i+1].altstack.
            // Thus, we have to pick f[i](z[i]).mainstack to the top of the stack
            for _ in (0..to_signed.stack.len()).rev() {
                { to_signed.total_len() + to_signed.stack.len() - 1 } OP_ROLL
            }

            // At this point, we should have
            // { f[i](z[i]).altstack, z[i+1].mainstack, f[i](z[i]).mainstack }

            // 3. Checking if z[i+1] == f(z[i])
            // 3.1. Mainstack verification
            { OP_LONGNOTEQUAL(to_signed.stack.len()) }

            // 3.2. Altstack verification
            for _ in 0..to_signed.altstack.len() {
                OP_FROMALTSTACK
            }

            // Since currently our stack looks like:
            // { f[i](z[i]).altstack, {bit}, z[i+1].altstack, },
            // we need to push f[i](z[i]).altstack to the top of the stack
            for _ in 0..to_signed.altstack.len() {
                { 2*to_signed.altstack.len() } OP_ROLL
            }

            { OP_LONGNOTEQUAL(to_signed.altstack.len()) }
            OP_BOOLOR
        };

        Self {
            script_witness,
            script_pubkey,
        }
    }
}

/// Given the script and its input, does the following:
/// - Splits the script into shards
/// - For each shard, creates a DisproveScript
/// - Returns the list of DisproveScripts
pub fn form_disprove_scripts<const I: usize, const O: usize, S: SplitableScript<I, O>>(
    input: Script,
) -> Vec<DisproveScript> {
    // Splitting the script into shards
    let split_result = S::default_split(input.clone(), SplitType::default());

    assert_eq!(
        split_result.shards.len(),
        split_result.intermediate_states.len(),
        "Shards and intermediate states must have the same length"
    );

    (0..split_result.shards.len())
        .map(|i| {
            let from_state = if i == 0 {
                IntermediateState::from_inject_script(&input.clone())
            } else {
                split_result.intermediate_states[i - 1].clone()
            };

            DisproveScript::new(
                &from_state,
                &split_result.intermediate_states[i],
                &split_result.shards[i],
            )
        })
        .collect()
}
