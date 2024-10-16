use bitcoin_scriptexec::Stack;
use treepp::*;

pub mod comparison;
pub mod debug;
pub mod pseudo;

#[allow(dead_code)]
// Re-export what is needed to write treepp scripts
pub mod treepp {
    pub use crate::debug::{execute_script, run};
    pub use bitcoin_script::{define_pushable, script};

    define_pushable!();
    pub use bitcoin::ScriptBuf as Script;
}

/// Converts a stack to a script that pushes all elements of the stack
pub fn stack_to_script(stack: &Stack) -> Script {
    script! {
        for element in stack.iter_str() {
            { element.to_vec() }
        }
    }
}
