//! Module containing the structure for scripts that we are going to use

use super::{core::naive_split, intermediate_state::IntermediateState};
use crate::treepp::*;

/// Structure that represents a pair of input and output scripts. Typically, the prover
/// wants to prove `script(input) == output`
pub struct IOPair<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize> {
    /// Input script containing the elements which will be fed to the main script
    pub input: Script,
    /// Output script containing the elements which will be compared to the output of the main script
    pub output: Script,
}

/// Structure that represents the result of splitting a script
pub struct SplitResult {
    /// Scripts (shards) that constitute the input script
    pub shards: Vec<Script>,
    /// Scripts that contain intermediate states (z values in the paper)
    pub intermediate_states: Vec<IntermediateState>,
}

impl SplitResult {
    /// Creates a new instance of the SplitResult
    pub fn new(shards: Vec<Script>, intermediate_states: Vec<IntermediateState>) -> Self {
        Self {
            shards,
            intermediate_states,
        }
    }

    /// Returns the last intermediate state, ignoring the possibility of the empty vector
    pub fn must_last_state(&self) -> &IntermediateState {
        self.intermediate_states
            .last()
            .expect("Intermediate states should not be empty")
    }

    /// Returns the total size of the states (stack + altstack)
    pub fn total_states_size(&self) -> usize {
        self.intermediate_states
            .iter()
            .map(|state| state.stack.len() + state.altstack.len())
            .sum()
    }
}

/// Trait that any script that can be split should implement
pub trait SplitableScript<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize> {
    const INPUT_SIZE: usize = INPUT_SIZE;
    const OUTPUT_SIZE: usize = OUTPUT_SIZE;

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
