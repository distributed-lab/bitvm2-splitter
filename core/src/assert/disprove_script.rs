use bitvm2_splitter::treepp::*;

use crate::winternitz::{self, PublicKey};

/// Script which let's challengers spent the **Assert** transaction
/// output if operator computated substates incorrectly.
///
/// This a typed version of [`Script`] can be easily converted into it.
///
/// The script structure in general is simple:
///
/// ```no_run
/// // push public key's of intermidiate states
/// { pk_z[i-1] }
/// { pk_z[i] }
/// // Check Winternitz signatures expecting public
/// // keys and signatures from top of the stack
/// { winternitz::checksig_verify }
/// // compute result of the subrpogram
/// { f[i] }
/// OP_EQUAL
/// OP_VERIFY
/// ```
///
/// # TODO:
///
/// - [ ] Inlcude covenants
pub struct DisproveScript {
    intermidiate_state_pubkeys: [PublicKey; 2],
    subprogram: Script,
}

impl From<DisproveScript> for Script {
    fn from(disprove: DisproveScript) -> Self {
        let [ pk1, pk2 ] = disprove.intermidiate_state_pubkeys;
        script! {
            { pk1 }
            // { winternitz::checksig_verify }
            { pk2 }
        }
    }
}
