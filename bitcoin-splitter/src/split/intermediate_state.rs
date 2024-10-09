//! This module contains the [`IntermediateState`] struct, which is used to store the intermediate
//! state of the stack and altstack during the execution of a script split into the shards (subprograms).

use core::fmt;

use crate::{treepp::*, utils::stack_to_script};
use bitcoin_scriptexec::Stack;

/// Structure that represents the intermediate state.
/// It contains the stack and altstack after the execution of the
/// corresponding shard.
pub struct IntermediateState {
    pub stack: Stack,
    pub altstack: Stack,
}

impl fmt::Debug for IntermediateState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Stack: {:?}", self.stack)?;
        writeln!(f, "Altstack: {:?}", self.altstack)
    }
}

impl IntermediateState {
    /// Executes the script with the given input and returns the intermediate result,
    /// that is, the stack and altstack after the execution
    pub fn from_input_script(input: &Script, script: &Script) -> Self {
        let script = script! {
            { input.clone() }
            { script.clone() }
        };

        let result = execute_script(script);

        Self {
            stack: result.main_stack.clone(),
            altstack: result.alt_stack.clone(),
        }
    }

    /// Based on the previous intermediate result, executes the script with the stacks
    /// and altstacks of the previous result and returns the new intermediate result
    pub fn from_intermediate_result(result: &Self, script: &Script) -> Self {
        let Self { stack, altstack } = result;

        let insert_result_script = script! {
            // Checks for length of the stack and altstack
            // are used to avoid panic when referencing the first element
            // of the stack or altstack (by using get(0) method)
            if !stack.is_empty() {
                { stack_to_script(stack) }
            }

            if !altstack.is_empty() {
                { stack_to_script(altstack) }

                for i in (0..altstack.len()).rev() {
                    { i } OP_ROLL
                    OP_TOALTSTACK
                }
            }
        };

        Self::from_input_script(&insert_result_script, script)
    }

    /// Converts the stack to a vector of u32 values
    pub fn interpret_as_u32_array(&self) -> Vec<u8> {
        self.stack.clone().serialize_to_bytes()
    }

    /// Returns the size of the stack and altstack
    pub fn size(&self) -> usize {
        self.stack.len() + self.altstack.len()
    }
}
