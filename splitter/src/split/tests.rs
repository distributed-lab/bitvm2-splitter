use crate::treepp::*;

use super::core::split_into_shards;

/// Tests whether splitting the script into subprograms (shards)
/// works properly
#[test]
fn test_split() {
    const CHUNK_SIZE: usize = 3;

    // Adding a toy script to test the splitting
    let test_script = script! {
        {13123} {1235} OP_ADD {4234} OP_ADD {3} OP_ADD {18595} OP_EQUAL
    };
    println!("Initial script: {}", test_script.to_asm_string());

    // Verifying its correctness 
    let result = execute_script(test_script.clone());
    assert!(result.success, "Test script failed");

    let shards = split_into_shards(&test_script, CHUNK_SIZE);

    // Asserting that we have three shards and each shard has at most three elements in the stack
    assert_eq!(
        shards.len(),
        3,
        "Shards number is incorrect"
    );
    for i in 0..3 {
        println!("Shard {}: {}", i, shards[i].to_asm_string());
    }

    // Asserting that the shards are correct
    let shard_1_script = script! {
        { shards[0].clone() }
        { 14358 }
        OP_EQUAL
    };
    let shard_1_result = execute_script(shard_1_script);
    assert!(shard_1_result.success, "Shard 1 failed");
}
