//! Module containing the structure for scripts that we are going to use

use bitcoin_scriptexec::Stack;

use crate::treepp::*;

use super::core::naive_split;

/// Structure that represents a pair of input and output scripts. Typically, the prover
/// wants to prove `script(input) == output`
pub struct IOPair<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize> {
    /// Input script containing the elements which will be fed to the main script
    pub input: Script,
    /// Output script containing the elements which will be compared to the output of the main script
    pub output: Script,
}

pub struct IntermediateResult {
    pub stack: Stack,
    pub altstack: Stack,
}

/// Structure that represents the result of splitting a script
pub struct SplitResult {
    /// Scripts (shards) that constitute the input script
    pub shards: Vec<Script>,
    /// Scripts that contain intermediate results (z values in the paper)
    pub intermediate_results: Vec<IntermediateResult>,
}

/// Trait that any script that can be split should implement
pub trait SplitableScript<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize> {
    /// Returns the main logic (f) of the script
    fn script() -> Script;

    /// Generates a random valid input for the script
    fn generate_valid_io_pair() -> IOPair<INPUT_SIZE, OUTPUT_SIZE>;

    /// Verifies that the input is valid for the script
    fn verify(input: Script, output: Script) -> bool {
        let script = script! {
            { input }
            { Self::script() }
            { output }

            // Now, we need to verify that the output is correct.
            // Since the output is not necessarily a single element, we check
            // elements one by one
            for i in (0..OUTPUT_SIZE).rev() {
                // { <a_1> <a_2> ... <a_n> <b_1> <b_2> ... <b_n> } <- we need to push element <a_n> to the top of the stack
                { i+1 } OP_ROLL
                OP_EQUALVERIFY
            }

            // If everything was verified correctly, we return true to mark the script as successful
            OP_TRUE
        };

        execute_script(script).success
    }

    /// Verifies that the input is valid for the script with random input and output
    fn verify_random() -> bool {
        let IOPair { input, output } = Self::generate_valid_io_pair();
        Self::verify(input, output)
    }

    /// Splits the script into smaller parts
    fn split(input: Script) -> SplitResult {
        naive_split(input, Self::script())
    }
}
