//! Module containing the logic of splitting the script into smaller parts

use bitcoin::{
    opcodes::all::{OP_ENDIF, OP_IF},
    script::Instruction,
};

use super::script::SplitResult;
use crate::{split::intermediate_state::IntermediateState, treepp::*};

/// Optimal size of the script in bytes
///
/// Note that it is not possible to cut the script into ideal
/// pieces of the same size, so we need to find the optimal size
/// that will be used to split the script and the algorithm
/// will try to keep the size of the script as close to the optimal
/// as possible.
pub(super) const OPTIMAL_SCRIPT_SIZE: usize = 20000;

/// Maximum scriptsize in bytes that is allowed by the Bitcoin network
pub(super) const MAX_SCRIPT_SIZE: usize = 50000;

/// Type of the split that we are going to use
///
/// - [`SplitType::ByInstructions`]- splits the script by the number of instructions
/// - [`SplitType::ByBytes`] - splits the script by the number of bytes
pub enum SplitType {
    ByInstructions,
    ByBytes,
}

// TODO: Currently, the chunk size splits the script into the parts of the same size IN TERMS OF INSTRUCTIONS, not bytes.
/// Splits the given script into smaller parts
pub(super) fn split_into_shards(
    script: &Script,
    chunk_size: usize,
    split_type: SplitType,
) -> Vec<Script> {
    let instructions: Vec<Instruction> = script
        .instructions()
        .map(|instruction| instruction.expect("script is most likely corrupted"))
        .collect();

    // Now, we are going to collect the chunks
    let mut shards: Vec<Script> = vec![Script::new()];

    // Now, one of the biggest problems that can occur is that OP_IF and OP_ENDIF are not in the same shard.
    // For that reason, we count both of them and if the number of OP_IFs is not equal to the number of OP_ENDIFs,
    // we need to continue splitting the script until we have the same number of both.
    let mut if_count = 0;
    let mut endif_count = 0;

    for (instruction_id, instruction) in instructions.clone().into_iter().enumerate() {
        // Pushing the instruction to the current shard.
        let current_shard = shards.last_mut().expect("shards should not be empty");
        current_shard.push_instruction(instruction);

        let current_shard_size = match split_type {
            SplitType::ByInstructions => instruction_id % chunk_size + 1,
            SplitType::ByBytes => current_shard.len(),
        };

        // Checking if the current instruction is OP_IF or OP_ENDIF
        if let Instruction::Op(op) = instruction {
            match op {
                OP_IF => {
                    if_count += 1;
                }
                OP_ENDIF => {
                    endif_count += 1;
                }
                _ => {}
            }
        }

        // If the current shard is too big AND number of ifs and endifs
        // is the same, we need to create a new one
        if current_shard_size >= chunk_size && if_count == endif_count {
            shards.push(Script::new());
        }

        // Checking that the total size has not exceeded the maximum size
        assert!(
            current_shard_size <= MAX_SCRIPT_SIZE,
            "Script size has exceeded the maximum size"
        );
    }

    shards
}

pub(super) fn naive_split(input: Script, script: Script, split_type: SplitType) -> SplitResult {
    // First, we split the script into smaller parts
    let shards = split_into_shards(&script, OPTIMAL_SCRIPT_SIZE, split_type);
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
