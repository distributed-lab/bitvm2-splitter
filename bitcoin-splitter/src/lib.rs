#[allow(dead_code)]
// Re-export what is needed to write treepp scripts
pub mod treepp {
    pub use crate::debug::{execute_script, run};
    pub use bitcoin_script::{define_pushable, script};

    define_pushable!();
    pub use bitcoin::ScriptBuf as Script;
}

pub mod split;

pub(crate) mod bitvm;
pub(crate) mod debug;
pub(crate) mod pseudo;
pub(crate) mod test_scripts;
pub(crate) mod utils;

#[cfg(test)]
mod tests {
    use super::treepp::*;

    /// Tests that checks that environment was set up correctly by running a 2+3=5 script.
    #[test]
    fn test_healthy_check() {
        let script = script! {
            2 3 OP_ADD 5 OP_EQUAL
        };

        let exec_result = execute_script(script);
        assert!(exec_result.success);

        println!("Environment is set up correctly!");
    }
}
