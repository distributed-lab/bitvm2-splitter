//! Module containing the structure for scripts that we are going to use

use pushable::Pushable;

use crate::treepp::*;

/// Structure that represents a pair of input and output scripts. Typically, the prover
/// wants to prove `script(input) == output`
pub struct IOPair<I, O: Pushable, const INPUT_SIZE: usize, const OUTPUT_SIZE: usize> {
    pub input: [I; INPUT_SIZE],
    pub output: [O; OUTPUT_SIZE],
}

/// Trait that any script that can be split should implement
pub trait SplitableScript<I, O: Pushable, const INPUT_SIZE: usize, const OUTPUT_SIZE: usize> {
    /// Returns the main logic (f) of the script
    fn script() -> Script;

    /// Generates a random valid input for the script
    fn generate_valid_io_pair() -> IOPair<I, O, INPUT_SIZE, OUTPUT_SIZE>;

    /// Verifies that the input is valid for the script
    fn verify(input: [I; INPUT_SIZE], output: [O; OUTPUT_SIZE]) -> bool {
        let output_length = output.len();
        println!("Output length: {}", output_length);

        let script = script! {
            for i in 0..input.len() {
                { input[i] }
            }

            { Self::script() }
            for i in 0..input.len() {
                { output[i] }
            }

            // Now, we need to verify that the output is correct.
            // Since the output is not necessarily a single element, we check
            // elements one by one
            for i in (0..output_length).rev() {
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
}
