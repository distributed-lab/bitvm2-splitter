use bitcoin_scriptexec::Stack;

use crate::treepp::*;

pub(crate) fn stack_to_script(stack: &Stack) -> Script {
    script! {
        for element in stack.iter_str() {
            { element.to_vec() }
        }
    }
}
