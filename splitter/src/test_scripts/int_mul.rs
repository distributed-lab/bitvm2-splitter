//! This module contains the test script
//! for performing the multiplication of two large integers
//! (exceeding standard Bitcoin 31-bit integers)

use crate::{
    split::script::{IOPair, SplitableScript},
    treepp::*,
};
use bitcoin_window_mul::{
    bigint::{U254Windowed, U508},
    traits::integer::{NonNativeInteger, NonNativeLimbInteger},
};

use num_bigint::{BigUint, RandomBits};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Script that performs the addition of two 254-bit numbers
pub struct U254MulScript;

/// Input size is double the number of limbs of U254 since we are multiplying two numbers
const INPUT_SIZE: usize = 2 * U254Windowed::N_LIMBS;
/// Output size is the number of limbs of U508
const OUTPUT_SIZE: usize = U508::N_LIMBS;

impl SplitableScript<{ INPUT_SIZE }, { OUTPUT_SIZE }> for U254MulScript {
    fn script() -> Script {
        U254Windowed::OP_WIDENINGMUL::<U508>()
    }

    fn generate_valid_io_pair() -> IOPair<{ INPUT_SIZE }, { OUTPUT_SIZE }> {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        // Generate two random 254-bit numbers and calculate their sum
        let num_1: BigUint = prng.sample(RandomBits::new(254));
        let num_2: BigUint = prng.sample(RandomBits::new(254));
        let product: BigUint = num_1.clone() * num_2.clone();

        IOPair {
            input: script! {
                { U254Windowed::OP_PUSH_U32LESLICE(&num_1.to_u32_digits()) }
                { U254Windowed::OP_PUSH_U32LESLICE(&num_2.to_u32_digits()) }
            },
            output: U508::OP_PUSH_U32LESLICE(&product.to_u32_digits()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::stack_to_script;
    use bitcoin_window_mul::traits::comparable::Comparable;

    #[test]
    fn test_verify() {
        assert!(U254MulScript::verify_random());
    }

    #[test]
    fn test_naive_split_correctness() {
        // Generating a random valid input for the script and the script itself
        let IOPair { input, output } = U254MulScript::generate_valid_io_pair();
        assert!(
            U254MulScript::verify(input.clone(), output.clone()),
            "input/output is not correct"
        );

        // Splitting the script into shards
        let split_result = U254MulScript::split(input.clone());

        // Now, we are going to concatenate all the shards and verify that the script is also correct
        let verification_script = script! {
            { input }
            for shard in split_result.shards {
                { shard }
            }
            { output }

            // Now, we need to verify that the output is correct.
            for i in (0..U254MulScript::OUTPUT_SIZE).rev() {
                // { <a_1> <a_2> ... <a_n> <b_1> <b_2> ... <b_n> } <- we need to push element <a_n> to the top of the stack
                { i+1 } OP_ROLL
                OP_EQUALVERIFY
            }

            OP_TRUE
        };

        let result = execute_script(verification_script);
        assert!(result.success, "Verification has failed");
    }

    #[test]
    fn test_naive_split() {
        // First, we generate the pair of input and output scripts
        let IOPair { input, output } = U254MulScript::generate_valid_io_pair();

        // Splitting the script into shards
        let split_result = U254MulScript::split(input);

        // Checking the last state (which must be equal to the result of the multiplication)
        let last_state = split_result.must_last_state();

        // Altstack must be empty
        assert!(last_state.altstack.is_empty(), "altstack is not empty!");

        // The element of the mainstack must be equal to the actual output
        let verification_script = script! {
            { stack_to_script(&last_state.stack) }
            { output }
            { U508::OP_EQUAL(0, 1) }
        };

        let result = execute_script(verification_script);
        assert!(result.success, "verification has failed");

        // Now, we debug the total size of the states
        let total_size = split_result.total_states_size();
        println!("Total size of the states: {} bytes", total_size);
    }
}
