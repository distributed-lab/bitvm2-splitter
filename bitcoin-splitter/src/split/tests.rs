use crate::{
    split::core::SplitType,
    treepp::*,
    utils::stack_to_script,
};

use super::{core::split_into_shards, intermediate_state::IntermediateState};

/// Tests whether splitting the script into subprograms (shards)
/// works properly for the most basic script
#[test]
fn test_split_basic() {
    const CHUNK_SIZE: usize = 3;

    // Adding a toy script to test the splitting
    let test_script = script! {
        {13123} {1235} OP_ADD {4234} OP_ADD {3} OP_ADD {18595} OP_EQUAL
    };
    println!("Initial script: {}", test_script.to_asm_string());
    println!("Initial script size: {}", test_script.len());

    // Verifying its correctness
    let result = execute_script(test_script.clone());
    assert!(result.success, "Test script failed");

    // Splitting the script into shards
    let shards = split_into_shards(&test_script, CHUNK_SIZE, SplitType::ByInstructions);

    // Debugging the shards
    for (i, shard) in shards.clone().into_iter().enumerate() {
        println!("Shard {}: {}", i, shard.to_asm_string());
        println!("Shard {} size: {}", i, shard.len());
    }

    // Asserting that the shards are correct
    let shard_1_script = script! {
        { shards[0].clone() }
        { 14358 }
        OP_EQUAL
    };
    let shard_1_result = execute_script(shard_1_script);
    assert!(shard_1_result.success, "Shard 1 failed");

    let shard_2_script = script! {
        { 1 }
        { shards[1].clone() }
        OP_DROP
        { 4235 }
        OP_EQUAL
    };
    let shard_2_result = execute_script(shard_2_script);
    assert!(shard_2_result.success, "Shard 2 failed");

    let shard_3_result = execute_script(shards[2].clone());
    assert!(
        !shard_3_result.success,
        "Shard 3 for somer reason succeeded"
    );

    // Now, we are going to concatenate all the shards and verify that the script is also correct
    let verification_script = script! {
        for shard in shards {
            { shard }
        }
    };
    let result = execute_script(verification_script);
    assert!(result.success, "Verification has failed");
}

#[test]
fn test_from_input_script_mainstack_only() {
    // Adding input and verification scripts
    let input_script = script! {
        { 13123 } { 1235 }
    };
    let main_script = script! {
        OP_ADD 1 OP_ADD 3 OP_ADD
    };

    // Creating an intermediate state
    let IntermediateState { stack, altstack } =
        IntermediateState::from_input_script(&input_script, &main_script);

    // Asserting that the altstack is empty
    assert!(altstack.is_empty(), "Altstack is not empty");

    // Now, checking that our stack is simply a number 14362
    let verify_script = script! {
        { stack_to_script(&stack) }
        { 14362 }
        OP_EQUAL
    };
    let result = execute_script(verify_script);
    assert!(result.success, "Verification failed");
}

#[test]
fn test_state_from_input_script_mainstack_and_altstack_1() {
    // Adding input and verification scripts
    let input_script = script! {
        {13123} {1235}
    };
    let main_script = script! {
        OP_ADD 1 OP_ADD 3 OP_ADD 5 OP_TOALTSTACK
    };

    // Creating an intermediate state
    let IntermediateState { stack, altstack } =
        IntermediateState::from_input_script(&input_script, &main_script);

    // Now, checking that our stack is simply a number 14362
    let verify_mainstack_script = script! {
        { stack_to_script(&stack) }
        {14362}
        OP_EQUAL
    };
    let result = execute_script(verify_mainstack_script);
    assert!(result.success, "mainstack verification failed");

    // Asserting that the altstack is correct
    let verify_altstack_script = script! {
        { stack_to_script(&altstack) }
        { 5 }
        OP_EQUAL
    };
    let result = execute_script(verify_altstack_script);
    assert!(result.success, "altstack verification failed");
}

#[test]
fn test_state_from_input_script_mainstack_and_altstack_2() {
    // Adding input and verification scripts
    let input_script = script! {
        { 13123 } { 1235 }
    };
    let main_script = script! {
        OP_ADD OP_1 OP_ADD OP_3 OP_ADD
        { 5 }  OP_TOALTSTACK
        { 100 } OP_TOALTSTACK
        { 20050 } OP_TOALTSTACK
    };

    // Creating an intermediate state
    let IntermediateState { stack, altstack } =
        IntermediateState::from_input_script(&input_script, &main_script);

    // Now, checking that our stack is simply a number 14362
    let verify_mainstack_script = script! {
        { stack_to_script(&stack) }
        { 14362 }
        OP_EQUAL
    };
    let result = execute_script(verify_mainstack_script);
    assert!(result.success, "mainstack verification failed");

    // Asserting that the altstack is correct
    let verify_altstack_script = script! {
        { stack_to_script(&altstack) }
        { 20050 } OP_EQUALVERIFY
        { 100 } OP_EQUALVERIFY
        { 5 } OP_EQUAL
    };
    let result = execute_script(verify_altstack_script);
    assert!(result.success, "altstack verification failed");
}

#[test]
fn test_if_split() {
    const CHUNK_SIZE: usize = 3;

    // Adding input and verification scripts
    let input_script = script! {
        { 5 } { 10 }
    };
    let main_script = script! {
        10 OP_EQUAL OP_IF
            { 20 }
            OP_ADD
        OP_ELSE
            { 50 } OP_ADD
        OP_ENDIF

        { 20 }
        OP_ADD
    };

    // Now, splitting the main_script:
    let shards = split_into_shards(&main_script, CHUNK_SIZE, SplitType::ByInstructions);

    // Creating the first intermediate state
    let z1 = IntermediateState::from_input_script(&input_script, &shards[0]);

    // Asserting that both the stack and altstack are correct
    let verify_main_stack_script = script! {
        { stack_to_script(&z1.stack) }
        { 25 } OP_EQUAL
    };

    let result = execute_script(verify_main_stack_script);
    assert!(result.success, "z1 mainstack verification failed");
}

#[test]
fn test_state_from_state() {
    const CHUNK_SIZE: usize = 3;

    // Adding input and verification scripts
    let input_script = script! {
        { 10 } { 20 }
    };
    let main_script = script! {
        OP_1 OP_TOALTSTACK OP_1 OP_TOALTSTACK OP_0 OP_TOALTSTACK
        OP_FROMALTSTACK OP_FROMALTSTACK OP_FROMALTSTACK
        OP_ADD OP_ADD OP_ADD
    };

    // Now, splitting the main_script:
    let shards = split_into_shards(&main_script, CHUNK_SIZE, SplitType::ByInstructions);

    // Creating the first intermediate state
    let z1 = IntermediateState::from_input_script(&input_script, &shards[0]);

    // Asserting that both the stack and altstack are correct
    let verify_main_stack_script = script! {
        { stack_to_script(&z1.stack) }
        OP_1 OP_EQUALVERIFY
        { 20 } OP_EQUALVERIFY
        { 10 } OP_EQUAL
    };

    let result = execute_script(verify_main_stack_script);
    assert!(result.success, "z1 mainstack verification failed");

    let verify_alt_stack_script = script! {
        { stack_to_script(&z1.altstack) }
        OP_1 OP_EQUAL
    };
    let result = execute_script(verify_alt_stack_script);
    assert!(result.success, "z1 altstack verification failed");

    // Now, getting the second state
    let z2 = IntermediateState::from_intermediate_result(&z1, &shards[1]);

    // Asserting that both the stack and altstack are correct
    let verify_main_stack_script = script! {
        { stack_to_script(&z2.stack) }
        { 20 } OP_EQUALVERIFY
        { 10 } OP_EQUAL
    };
    let result = execute_script(verify_main_stack_script);
    assert!(result.success, "z2 mainstack verification failed");

    let verify_alt_stack_script = script! {
        { stack_to_script(&z2.altstack) }
        OP_0 OP_EQUALVERIFY
        OP_1 OP_EQUALVERIFY
        OP_1 OP_EQUAL
    };
    let result = execute_script(verify_alt_stack_script);
    assert!(result.success, "z2 altstack verification failed");

    // Now, getting the third state
    let z3 = IntermediateState::from_intermediate_result(&z2, &shards[2]);

    // Asserting that both the stack and altstack are correct
    let verify_main_stack_script = script! {
        { stack_to_script(&z3.stack) }
        OP_1 OP_EQUALVERIFY
        OP_1 OP_EQUALVERIFY
        OP_0 OP_EQUALVERIFY
        { 20 } OP_EQUALVERIFY
        { 10 } OP_EQUAL
    };
    let result = execute_script(verify_main_stack_script);
    assert!(result.success, "z3 mainstack verification failed");

    assert!(
        z3.altstack.is_empty(),
        "z3 altstack should be empty at this point"
    );
}
