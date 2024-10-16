//! Module containing the logic of splitting the script into smaller parts

use std::panic;

use bitcoin::{
    opcodes::all::{OP_ENDIF, OP_IF, OP_NOTIF},
    script::Instruction,
};
use bitcoin_utils::treepp::*;
use indicatif::ProgressBar;

use super::script::SplitResult;
use crate::split::intermediate_state::IntermediateState;

/// Optimal size of the script in bytes
///
/// Note that it is not possible to cut the script into ideal
/// pieces of the same size, so we need to find the optimal size
/// that will be used to split the script and the algorithm
/// will try to keep the size of the script as close to the optimal
/// as possible.
pub(super) const DEFAULT_SCRIPT_SIZE: usize = 7000;

/// Maximum scriptsize in bytes that is allowed by the Bitcoin network
pub(super) const MAX_SCRIPT_SIZE: usize = 50000;

/// When dealing with BitVM2 Disprove transaction,
/// there are two factors that we need to consider:
/// 1. The chunk (shard) script size.
/// 2. The intermediate state size.
///
/// The total size of the disprove script is (approximately):
/// `shard_size_i + (z_i_size + z_(i-1)_size) * STACK_SIZE_INDEX`
///
/// The `STACK_SIZE_INDEX` is the factor of how much intermediate states
/// are contributing to the total size of the script.
pub(super) const STACK_SIZE_INDEX: usize = 1000;

/// Type of the split that we are going to use
///
/// - [`SplitType::ByInstructions`]- splits the script by the number of instructions
/// - [`SplitType::ByBytes`] - splits the script by the number of bytes
#[derive(Debug, Clone, Copy, Default)]
pub enum SplitType {
    #[default]
    ByInstructions,
    ByBytes,
}

/// Splits the given script into smaller parts. Tries to keep each chunk size
/// to the optimal size `chunk_size` as close as possible.
pub fn split_into_shards(script: &Script, chunk_size: usize, split_type: SplitType) -> Vec<Script> {
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
                OP_IF | OP_NOTIF => {
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
            if_count = 0;
            endif_count = 0;
        }

        // Checking that the total size has not exceeded the maximum size
        assert!(
            current_shard_size <= MAX_SCRIPT_SIZE,
            "Script size has exceeded the maximum size"
        );
    }

    shards
}

/// Fuzzy split of the script into smaller parts by searching for the optimal size
/// by checking various script sizes
pub fn fuzzy_split(input: Script, script: Script, split_type: SplitType) -> SplitResult {
    // Define the limits
    const MIN_CHUNK_SIZE: usize = 100;
    const MAX_CHUNK_SIZE: usize = MAX_SCRIPT_SIZE;
    const STEP_SIZE: usize = 20;

    // Defining the final result
    let mut resultant_split = SplitResult::new(vec![], vec![]);
    let mut resultant_complexity = usize::MAX;

    // Now, displaying the progress bar
    let total_progress = (MAX_CHUNK_SIZE - MIN_CHUNK_SIZE) / STEP_SIZE;
    let bar = ProgressBar::new(total_progress as u64);

    // Trying to find the optimal size of the chunk by checking
    // each size from MIN_CHUNK_SIZE to MAX_CHUNK_SIZE
    for chunk_size in (MIN_CHUNK_SIZE..MAX_CHUNK_SIZE).step_by(STEP_SIZE) {
        // Incrementing the progress bar
        bar.inc(1);
        // We are using panic::catch_unwind to catch any panics that might occur
        // during the splitting process. If a panic occurs, we just skip the current
        // chunk size and continue with the next one.
        let current_split_result = panic::catch_unwind(|| {
            naive_split(input.clone(), script.clone(), split_type, chunk_size)
        });
        if let Ok(split_result) = current_split_result {
            let current_complexity = split_result.complexity_index();

            if current_complexity < resultant_complexity {
                resultant_complexity = current_complexity;
                resultant_split = split_result;
            }
        }
    }

    bar.finish();
    resultant_split
}

/// Default split of the script into smaller parts with the hard-coded optimal size
pub fn default_split(input: Script, script: Script, split_type: SplitType) -> SplitResult {
    naive_split(input, script, split_type, DEFAULT_SCRIPT_SIZE)
}

/// Naive split of the script into smaller parts. It works as follows:
/// 1. We split the script into smaller parts
/// 2. We execute each shard with the input
/// 3. Save intermediate results
/// 4. Return all the shards and intermediate results in the form of [`SplitResult`]
pub fn naive_split(
    input: Script,
    script: Script,
    split_type: SplitType,
    chunk_size: usize,
) -> SplitResult {
    // First, we split the script into smaller parts
    let shards = split_into_shards(&script, chunk_size, split_type);
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
