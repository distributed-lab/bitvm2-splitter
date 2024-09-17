//! Module containing the logic of splitting the script into smaller parts

use crate::{split::script::IntermediateResult, treepp::*};
use super::script::SplitResult;

/// Maximum size of the script in bytes
pub(super) const MAX_SCRIPT_SIZE: usize = 30000;

/// Splits the given script into smaller parts
fn split_into_shards(script: Script) -> Vec<Script> {
    script
        .as_bytes()
        .chunks(MAX_SCRIPT_SIZE)
        .map(|chunk| {
            script! {
                { chunk.to_vec() }
            }
        })
        .collect()
}

pub(super) fn naive_split(input: Script, script: Script) -> SplitResult {
    // First, we split the script into smaller parts
    let shards = split_into_shards(script);
    let mut intermediate_results: Vec<IntermediateResult> = vec![];

    // Then, we do the following steps:
    // 1. We execute the first script with the input
    // 2. We take the stack and write it to the intermediate results
    // 3. We execute the second script with the saved intermediate results
    // 4. Take the stask, save to the intermediate results
    // 5. Repeat until the last script
    let first_shard_script = script! {
        { input }
        { shards[0].clone() }
    };
    let result = execute_script(first_shard_script);
    intermediate_results.push(IntermediateResult{
        stack: result.main_stack.clone(),
        altstack: result.alt_stack.clone(),
    });
    println!("Intermediate lengths: {:?} + {:?}", result.main_stack.len(), result.alt_stack.len());

    for shard in shards.clone().into_iter().skip(1) {
        // Executing a piece of the script with the current input.
        // NOTE #1: unwrap is safe to use here since intermediate_results is of length 1.
        // NOTE #2: we need to feed in both the stack and the altstack

        let current_input = intermediate_results.last().unwrap();
        let IntermediateResult{stack, altstack} = current_input;

        println!("Stack lengths: {:?} + {:?}", stack.len(), altstack.len());

        let temporary_script = script! {
            // Pushing main stack
            if stack.len() > 0 {
                { stack.get(0) }
            }

            // Pushing alt stack
            if altstack.len() > 0 {
                { altstack.get(0) }
                for _ in 0..altstack.len() {
                    OP_TOALTSTACK
                }
            }

            // Pushing the current shard
            { shard.clone() }
        };

        let result = execute_script(temporary_script);
        intermediate_results.push(IntermediateResult{
            stack: result.main_stack.clone(),
            altstack: result.alt_stack.clone(),
        });

        println!("Stack lengths: {:?} + {:?}", result.main_stack.len(), result.alt_stack.len());
    }

    assert_eq!(
        intermediate_results.len(),
        shards.len(),
        "Intermediate results should be the same as the number of scripts"
    );

    SplitResult {
        shards,
        intermediate_results,
    }
}
