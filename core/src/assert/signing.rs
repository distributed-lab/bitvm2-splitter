use crate::treepp::*;

use bitcoin_splitter::split::intermediate_state::IntermediateState;
use bitcoin_winternitz::u32::{checksig_verify_script, Message, PublicKey, SecretKey, Signature};
use rand::{rngs::SmallRng, SeedableRng};

/// Maximum value of the stack element
const MAX_STACK_ELEMENT_VALUE: u32 = (1 << 31) - 1;

/// Struct handling information about a single u32 element in the state array.
/// Namely, besides the element itself, it also contains the public key, secret key,
/// and the signature of the element.
#[derive(Clone, Copy, Debug)]
pub struct SignedStackElement {
    pub stack_element: u32,
    pub encoding: Message,
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub signature: Signature,
}

impl SignedStackElement {
    /// Creates a new [`SignedStackElement`] by signing the given stack element
    fn sign(stack_element: u32) -> Self {
        // Creating a keypair
        // TODO(@ZamDimon): Reconsider rng usage
        let mut rng = SmallRng::from_entropy();
        let secret_key = SecretKey::random(&mut rng);
        let public_key = secret_key.public_key();

        // Signing the message
        let message = Message::from_u32(stack_element);
        let signature = secret_key.sign(&message);

        Self {
            stack_element,
            encoding: message,
            public_key,
            secret_key,
            signature,
        }
    }
}

/// Struct holding the intermediate state of the script execution.
///
/// Note that the intermediate state itself is just an array of
/// u32 values (both in mainstack and altstack), but this struct
/// also contains the public keys, secret keys, and signatures
/// of the elements in the state array.
#[derive(Clone, Debug)]
pub struct SignedIntermediateState {
    pub stack: Vec<SignedStackElement>,
    pub altstack: Vec<SignedStackElement>,
}

impl SignedIntermediateState {
    /// Creates a new IntermediateStateHolder from the given intermediate state
    pub fn sign(state: &IntermediateState) -> Self {
        let stack = state.to_bytes().stack_as_u32();
        let altstack = state.to_bytes().altstack_as_u32();

        // Now, verifying that all elements are below 1<<31 - 1
        for element in stack.iter().chain(altstack.iter()) {
            assert!(*element <= MAX_STACK_ELEMENT_VALUE, "element is too large");
        }

        // Signing each element
        let stack = stack.into_iter().map(SignedStackElement::sign).collect();
        let altstack = altstack.into_iter().map(SignedStackElement::sign).collect();

        Self { stack, altstack }
    }

    /// Returns the total length of the stack and altstack
    pub fn total_len(&self) -> usize {
        self.stack.len() + self.altstack.len()
    }

    /// Script that pushes zipped signature and message to the stack for
    /// each signed element in the stack and altstack.
    pub fn witness_script(&self) -> Script {
        script! {
            // Pushing the stack
            for element in self.stack.clone() {
                { element.signature.to_script_sig() }
            }

            // Pushing the altstack
            for element in self.altstack.clone().into_iter().rev() {
                { element.signature.to_script_sig() }
            }
        }
    }

    /// Script for verification of the witness script. Additionally,
    /// the verification leaves the original stack and altstack of
    /// the intermediate state.
    pub fn verification_script_toaltstack(&self) -> Script {
        script! {
            // For each element, we need to push the public key and run the
            // Winternitz verification script
            for element in self.altstack.clone() {
                { checksig_verify_script(&element.public_key) }
                { Message::recovery_script() }
                OP_TOALTSTACK
            }

            // Do the same for the mainstack
            for element in self.stack.clone().into_iter().rev() {
                { checksig_verify_script(&element.public_key) }
                { Message::recovery_script() }
                OP_TOALTSTACK
            }
        }
    }

    /// Script that pops the elements from the altstack after
    /// the verification of the witness script.
    pub fn verification_script_fromaltstack(&self) -> Script {
        script! {
            // Currently, the altstack contains the following elements:
            // { altstack_elements, stack_elements }
            // Thus, we can simply pop top stack elements and call it a day
            for _ in 0..self.stack.len() {
                OP_FROMALTSTACK
            }
        }
    }

    /// Script that verifies the witness script and the public keys
    /// of the elements in the stack and altstack. The verification
    /// leaves the original stack and altstack of the intermediate state.
    pub fn verification_script(&self) -> Script {
        script! {
            { self.verification_script_toaltstack() }
            { self.verification_script_fromaltstack() }
        }
    }
}
