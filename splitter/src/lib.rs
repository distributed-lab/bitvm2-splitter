#[allow(dead_code)]
// Re-export what is needed to write treepp scripts
pub mod treepp {
    pub use crate::debug::execute_script;
    pub use bitcoin_script::{define_pushable, script};

    define_pushable!();
    pub use bitcoin::ScriptBuf as Script;
}

pub(crate) mod debug;
pub mod split;
pub(crate) mod test_scripts;

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::treepp::*;

    /// Tests that checks that environment was set up correctly by running a 2+3=5 script.
    #[test]
    fn test_healthy_check_1() {
        let script = script! {
            2 3 OP_ADD 5 OP_EQUAL
        };

        let exec_result = execute_script(script);
        assert!(exec_result.success);

        println!("Environment is set up correctly!");
    }

    /// Tests that checks that environment was set up correctly by running a 2+3=5 script.
    #[test]
    fn test_healthy_check_2() {
        let script_1 = script! {
            13123 1235 OP_ADD 4234 OP_ADD
        };
        println!("Script 1 stack: {:?}", script_1.to_asm_string());

        let exec_result_1 = execute_script(script_1);

        let script_2 = script! {
            { exec_result_1.main_stack.get(0) }
            3 OP_ADD 18595 OP_EQUAL
        };

        println!("Script 2 stack: {:?}", script_2.to_asm_string());

        let exec_result_2 = execute_script(script_2);
        assert!(exec_result_2.success);
    }
}
