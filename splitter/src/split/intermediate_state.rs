//! This module contains the [`IntermediateState`] struct, which is used to store the intermediate
//! state of the stack and altstack during the execution of a script split into the shards (subprograms).

use crate::treepp::*;
use bitcoin_scriptexec::Stack;

pub struct IntermediateState {
    pub stack: Stack,
    pub altstack: Stack,
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
                { stack.get(0) }
            }

            if !altstack.is_empty() {
                { altstack.get(0) }

                for _ in 0..altstack.len() {
                    OP_TOALTSTACK
                }
            }
        };

        Self::from_input_script(&insert_result_script, script)
    }
}
