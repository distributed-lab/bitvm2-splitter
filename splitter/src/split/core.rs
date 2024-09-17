//! Module containing the logic of splitting the script into smaller parts

use bitcoin::script::Instruction;

use super::script::SplitResult;
use crate::{split::intermediate_state::IntermediateState, treepp::*};

/// Maximum size of the script in bytes
pub(super) const MAX_SCRIPT_SIZE: usize = 30000;

/// Splits the given script into smaller parts
pub(super) fn split_into_shards(script: &Script, chunk_size: usize) -> Vec<Script> {
    let instructions: Vec<Instruction> = script
        .instructions()
        .map(|instruction| {
            let instruction = instruction.expect("script is most likely corrupted");
            instruction
        }).collect();
    
    instructions
        .chunks(chunk_size)
        .map(|chunk| {
            for instruction in chunk {
                println!("{:?}", instruction);
            }

            let mut shard = Script::new();
            for instruction in chunk {
                shard.push_instruction(instruction.clone());
            }

            shard
        })
        .collect()
}

pub(super) fn naive_split(input: Script, script: Script) -> SplitResult {
    // First, we split the script into smaller parts
    let shards = split_into_shards(&script, MAX_SCRIPT_SIZE);
    let mut intermediate_states: Vec<IntermediateState> = vec![];

    // Then, we do the following steps:
    // 1. We execute the first script with the input
    // 2. We take the stack and write it to the intermediate results
    // 3. We execute the second script with the saved intermediate results
    // 4. Take the stask, save to the intermediate results
    // 5. Repeat until the last script

    intermediate_states.push(IntermediateState::from_input_script(&input, &shards[0]));

    for shard in shards.clone().into_iter().skip(1) {
        // Executing a piece of the script with the current input.
        // NOTE #1: unwrap is safe to use here since intermediate_states is of length 1.
        // NOTE #2: we need to feed in both the stack and the altstack

        intermediate_states.push(IntermediateState::from_intermediate_result(
            intermediate_states.last().unwrap(),
            &shard,
        ));
    }

    assert_eq!(
        intermediate_states.len(),
        shards.len(),
        "Intermediate results should be the same as the number of scripts"
    );
    SplitResult {
        shards,
        intermediate_states,
    }
}
