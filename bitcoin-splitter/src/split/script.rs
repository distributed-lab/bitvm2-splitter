//! Module containing the structure for scripts that we are going to use

use core::fmt;

use super::{
    core::{default_split, fuzzy_split, naive_split, SplitType, STACK_SIZE_INDEX},
    intermediate_state::IntermediateState,
};
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

impl fmt::Debug for SplitResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Number of intermediate states: {}",
            self.intermediate_states.len()
        )?;
        // Debugging first and last shards OPCODEs
        for (i, shard) in self.shards.iter().enumerate() {
            const MAX_CHARACTERS_TO_SHOW: usize = 100;
            let s = shard.to_asm_string();
            let first_opcodes = &s[..MAX_CHARACTERS_TO_SHOW];
            let last_opcodes = &s[s.len() - MAX_CHARACTERS_TO_SHOW..];

            writeln!(f, "Shard {}: {}...{}", i, first_opcodes, last_opcodes)?;
        }
        Ok(())
    }
}

impl SplitResult {
    /// Creates a new instance of the SplitResult
    pub fn new(shards: Vec<Script>, intermediate_states: Vec<IntermediateState>) -> Self {
        Self {
            shards,
            intermediate_states,
        }
    }

    /// Returns the number of intermediate states (and thus the number of shards)
    pub fn len(&self) -> usize {
        self.intermediate_states.len()
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
            .map(|state| state.size())
            .sum()
    }

    /// Returns the maximal size of the states (stack + altstack)
    pub fn max_states_size(&self) -> usize {
        self.intermediate_states
            .iter()
            .map(|state| state.size())
            .max()
            .unwrap_or(0)
    }

    /// Returns the maximal size of two adjacent states (stack + altstack)
    pub fn max_adjacent_states_size(&self) -> usize {
        self.intermediate_states
            .windows(2)
            .map(|states| states[0].size() + states[1].size())
            .max()
            .unwrap_or(0)
    }

    /// Returns the complexity index of the script splitting.
    /// The complexity index is the approximate worst number of opcodes
    /// it takes to form the disprove script.
    pub fn complexity_index(&self) -> usize {
        let mut resultant_complexity = 0;

        for i in 0..self.len() {
            // Calculating sizes of the shards and states.
            // Namely, since z[i] = f[i](z[i-1]), we need to calculate
            // the size (|z[i]| + |z[i-1]|) * STACK_SIZE_INDEX + |f[i]|
            resultant_complexity = resultant_complexity.max({
                let shard_size = self.shards[i].len();
                let current_state_size = self.intermediate_states[i].size();
                let previous_state_size = if i > 0 {
                    self.intermediate_states[i - 1].size()
                } else {
                    0
                };

                shard_size + (current_state_size + previous_state_size) * STACK_SIZE_INDEX
            });
        }

        resultant_complexity
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
    fn default_split(input: Script, split_type: SplitType) -> SplitResult {
        default_split(input, Self::script(), split_type)
    }

    /// Splits the script into smaller parts with the specified chunk size
    fn split(input: Script, split_type: SplitType, chunk_size: usize) -> SplitResult {
        naive_split(input, Self::script(), split_type, chunk_size)
    }

    /// Splits the script into smaller parts with the fuzzy split
    fn fuzzy_split(input: Script, split_type: SplitType) -> SplitResult {
        fuzzy_split(input, Self::script(), split_type)
    }
}
