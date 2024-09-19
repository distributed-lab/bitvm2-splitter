//! This module contains the test script
//! for performing the multiplication of two large integers
//! (exceeding standard Bitcoin 31-bit integers)

use crate::{
    bitvm::hash::{sha256::sha256, utils::push_bytes_hex},
    split::script::{IOPair, SplitableScript},
    treepp::*,
};

use rand::RngCore;
use sha2::{Digest, Sha256};

/// Script that performs the addition of two 254-bit numbers
pub struct SHA256Script<const INPUT_SIZE: usize>;

/// Output size is 32 bytes (256 bits)
const OUTPUT_SIZE: usize = 32;

impl<const INPUT_SIZE: usize> SplitableScript<INPUT_SIZE, { OUTPUT_SIZE }>
    for SHA256Script<INPUT_SIZE>
{
    fn script() -> Script {
        sha256(INPUT_SIZE)
    }

    fn generate_valid_io_pair() -> IOPair<INPUT_SIZE, { OUTPUT_SIZE }> {
        // Generate a random array of bytes
        let mut data = [0; INPUT_SIZE];
        rand::thread_rng().fill_bytes(&mut data);
        let data_hex = hex::encode(data);

        // Creating a SHA-256 hasher and find digest of the data
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();

        IOPair {
            input: script! {
                { push_bytes_hex(data_hex.as_str()) }
            },
            output: script! {
                { push_bytes_hex(hex::encode(result).as_str()) }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::stack_to_script;

    use super::*;

    #[test]
    fn test_sha256_verify() {
        const TEST_BYTES_NUM: usize = 80;
        assert!(SHA256Script::<TEST_BYTES_NUM>::verify_random());
    }

    #[test]
    fn test_naive_split_correctness() {
        // Choosing the number of bytes for the test
        const TEST_BYTES_NUM: usize = 80;
        type SHA256ScriptType = SHA256Script<TEST_BYTES_NUM>;

        // Generating a random valid input for the script and the script itself
        let IOPair { input, output } = SHA256ScriptType::generate_valid_io_pair();
        assert!(
            SHA256ScriptType::verify(input.clone(), output.clone()),
            "input/output is not correct"
        );

        // Splitting the script into shards
        let split_result = SHA256ScriptType::split(input.clone());

        // Now, we are going to concatenate all the shards and verify that the script is also correct
        let verification_script = script! {
            { input }
            for shard in split_result.shards {
                { shard }
            }
            { output }

            // Now, we need to verify that the output is correct.
            for i in (0..SHA256ScriptType::OUTPUT_SIZE).rev() {
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
        // Choosing the number of bytes for the test
        const TEST_BYTES_NUM: usize = 80;
        type SHA256ScriptType = SHA256Script<TEST_BYTES_NUM>;

        // First, we generate the pair of input and output scripts
        let IOPair { input, output } = SHA256ScriptType::generate_valid_io_pair();

        // Splitting the script into shards
        let split_result = SHA256ScriptType::split(input);

        // Checking the last state (which must be equal to the result of the multiplication)
        let last_state = split_result.must_last_state();

        // Altstack must be empty
        assert!(last_state.altstack.is_empty(), "altstack is not empty!");

        println!(
            "Last state: {:?}",
            stack_to_script(&last_state.stack).to_asm_string()
        );
        println!("Output: {:?}", output.to_asm_string());

        // The element of the mainstack must be equal to the actual output
        let verification_script = script! {
            { stack_to_script(&last_state.stack) }
            { output }

            // Now, we need to verify that the output is correct.
            // We have 32 bytes in the output, so we need to compare each byte
            for i in (0..SHA256ScriptType::OUTPUT_SIZE).rev() {
                { i+1 } OP_ROLL
                OP_EQUALVERIFY
            }

            // Marking the verification as successful
            OP_TRUE
        };

        let result = execute_script(verification_script);
        assert!(result.success, "verification has failed");

        // Now, we debug the total size of the states
        let total_size = split_result.total_states_size();
        println!("Total size of the states: {} bytes", total_size);
    }
}
