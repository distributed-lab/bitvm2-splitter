pub mod split;

#[cfg(test)]
mod tests {
    use bitcoin_utils::treepp::*;

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
