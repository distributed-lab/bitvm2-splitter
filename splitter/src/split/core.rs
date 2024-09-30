//! Module containing the logic of splitting the script into smaller parts

use bitcoin::script::{Instruction, Instructions, PushBytes};

use crate::treepp::*;

/// Maximum size of the script in bytes
pub(super) const MAX_SCRIPT_SIZE: usize = 30000;

pub struct SplitResult {
    /// Scripts that constitute the input script
    pub scripts: Vec<Script>, 
    /// Scripts that contain intermediate results
    pub intermediate_results: Vec<Script>,
}

/// Splits the given script into smaller parts
fn split_into_chunks(instructions: Instructions) -> Vec<Script> {
    let intructions: Vec<Instruction> = instructions
        .into_iter()
        .map(|x| x.expect("intructions are corrupted"))
        .collect();

    println!("Instructions: {:?}", intructions.len());
    
    intructions.chunks(MAX_SCRIPT_SIZE).map(|chunk| {
        println!("Chunk: {:?}", chunk.len());

        let num_opcodes = chunk.iter().filter(|x| matches!(x, Instruction::Op(_))).count();
        let num_pushes = chunk.iter().filter(|x| matches!(x, Instruction::PushBytes(_))).count();

        println!("Opcodes: {}, Pushes: {}", num_opcodes, num_pushes);

        script! {
            for instruction in chunk {
                if let Instruction::Op(op) = instruction {
                    { op.to_u8() }
                }
                else if let Instruction::PushBytes(bytes) = instruction {
                    { PushBytes::as_bytes(*bytes).to_vec() }
                }
            }
        }
    }).collect()
}

pub(super) fn naive_split(script: Script) -> SplitResult {
    let scripts = split_into_chunks(script.instructions());
    
    SplitResult {
        scripts,
        intermediate_results: vec![],
    }
}
