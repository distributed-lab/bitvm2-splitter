use bitcoin_scriptexec::Stack;

use crate::treepp::*;

/// Converts a stack to a script that pushes all elements of the stack
pub(crate) fn stack_to_script(stack: &Stack) -> Script {
    script! {
        for element in stack.iter_str() {
            { element.to_vec() }
        }
    }
}
