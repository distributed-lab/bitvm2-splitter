use bitcoin_splitter::split::{
    core::{split_into_shards, SplitType},
    script::{IOPair, SplitableScript},
};
use bitcoin_utils::treepp::*;

use crate::int_mul_windowed::U254MulScript;

/// Tests whether splitting the script into subprograms (shards)
/// works properly for the most advanced script (two big integers multipication)
#[test]
fn test_split_mul() {
    const CHUNK_SIZE: usize = 100;

    // Generating a random valid input for the script and the script itself
    let test_script = U254MulScript::script();
    let IOPair { input, output } = U254MulScript::generate_valid_io_pair();
    assert!(
        U254MulScript::verify(input.clone(), output.clone()),
        "input/output is not correct"
    );

    // Splitting the script into shards
    let shards = split_into_shards(&test_script, CHUNK_SIZE, SplitType::ByInstructions);

    // Now, we are going to concatenate all the shards and verify that the script is also correct
    let verification_script = script! {
        { input }
        for shard in shards {
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
