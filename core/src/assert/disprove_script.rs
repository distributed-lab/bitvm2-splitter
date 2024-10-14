use crate::treepp::*;
use super::signing::SignedIntermediateState;

use bitcoin_splitter::split::intermediate_state::IntermediateState;

/// Script letting challengers spend the **Assert** transaction
/// output if the operator computated substates incorrectly.
///
/// This a typed version of [`Script`] can be easily converted into it.
///
/// The script structure in general is simple:
/// ## Witness:
/// ```
/// { Enc(z[i+1]) and Sig[i+1] } // Zipped
/// { Enc(z[i]) and Sig[i] }     // Zipped
/// ```
///
/// ## Script:
/// ```
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
pub struct DisproveScript {
    pub script_witness: Script,
    pub script_pubkey: Script,
}

impl DisproveScript {
    /// Given the previous and current states, and the function that was executed,
    /// creates a new DisproveScript according to the BitVM2 protocol.
    pub fn new(from: IntermediateState, to: IntermediateState, function: Script) -> Self {
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
            { function } // This leaves f[i](z[i]).mainstack in the mainstack and z[i+1] in the altstack
            { to_signed.verification_script_fromaltstack() } // This leaves z[i+1].mainstack and f[i](z[i]).mainstack in the mainstack, while f[i](z[i]).altstack and z[i+1].alstack is in the altstack
            
            // 3. Checking if z[i+1] == f(z[i])
            // 3.1. First, check mainstack equality
            for j in (0..to_signed.stack.len()).into_iter().rev() {
                { j } OP_ROLL OP_EQUALVERIFY // This checks if z[i+1][j] == f(z[i])[j] for each j
            }
            // 3.2. Pop all elements from the altstack
            for _ in 0..2*to_signed.altstack.len() {
                OP_FROMALTSTACK
            }
            // 3.3. Compare altstack elements in the mainstack
            for j in (0..to_signed.altstack.len()).into_iter().rev() {
                { j } OP_ROLL OP_EQUALVERIFY // This checks if z[i+1][j] == f(z[i])[j] for each j
            }

            OP_TRUE
        };

        Self {
            script_witness,
            script_pubkey,
        }
    }
}
