use bitcoin::{hashes::Hash, ScriptBuf, TapLeafHash, Transaction};
use bitcoin_scriptexec::{Exec, ExecCtx, ExecError, ExecStats, Options, Stack, TxTemplate};
use core::fmt;

/// Information about the status of the script execution.
#[derive(Debug)]
pub struct ExecuteInfo {
    pub success: bool,
    pub error: Option<ExecError>,
    pub main_stack: Stack,
    pub alt_stack: Stack,
    pub stats: ExecStats,
}

impl fmt::Display for ExecuteInfo {
    /// Formats the `ExecuteInfo` struct for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.success {
            writeln!(f, "Script execution successful.")?;
        } else {
            writeln!(f, "Script execution failed!")?;
        }
        if let Some(ref error) = self.error {
            writeln!(f, "Error: {:?}", error)?;
        }

        writeln!(f, "Stats: {:?}", self.stats)?;
        Ok(())
    }
}

/// Executes the given script and returns the result of the execution
/// (success, error, stack, etc.)
pub fn execute_script(script: ScriptBuf) -> ExecuteInfo {
    let mut exec = Exec::new(
        ExecCtx::Tapscript,
        Options {
            require_minimal: false,
            ..Default::default()
        },
        TxTemplate {
            tx: Transaction {
                version: bitcoin::transaction::Version::TWO,
                lock_time: bitcoin::locktime::absolute::LockTime::ZERO,
                input: vec![],
                output: vec![],
            },
            prevouts: vec![],
            input_idx: 0,
            taproot_annex_scriptleaf: Some((TapLeafHash::all_zeros(), None)),
        },
        script,
        vec![],
    )
    .expect("error when creating the execution body");

    // Execute all the opcodes while possible
    loop {
        if exec.exec_next().is_err() {
            break;
        }
    }

    // Obtaining the result of the execution
    let result = exec.result().unwrap();

    ExecuteInfo {
        success: result.success,
        error: result.error.clone(),
        main_stack: exec.stack().clone(),
        alt_stack: exec.altstack().clone(),
        stats: exec.stats().clone(),
    }
}

/// Execute a script on stack without `MAX_STACK_SIZE` limit.
/// This function is only used for script test, not for production.
///
/// NOTE: Only for test purposes.
#[allow(dead_code)]
pub fn execute_script_no_stack_limit(script: bitcoin::ScriptBuf) -> ExecuteInfo {
    // Get the default options for the script exec.
    // NOTE: Do not enforce the stack limit.
    let opts = Options {
        enforce_stack_limit: false,
        ..Default::default()
    };

    let mut exec = Exec::new(
        ExecCtx::Tapscript,
        opts,
        TxTemplate {
            tx: Transaction {
                version: bitcoin::transaction::Version::TWO,
                lock_time: bitcoin::locktime::absolute::LockTime::ZERO,
                input: vec![],
                output: vec![],
            },
            prevouts: vec![],
            input_idx: 0,
            taproot_annex_scriptleaf: Some((TapLeafHash::all_zeros(), None)),
        },
        script,
        vec![],
    )
    .expect("error while creating the execution body");

    // Execute all the opcodes while possible
    loop {
        if exec.exec_next().is_err() {
            break;
        }
    }

    // Get the result of the execution
    let result = exec.result().unwrap();

    ExecuteInfo {
        success: result.success,
        error: result.error.clone(),
        main_stack: exec.stack().clone(),
        alt_stack: exec.altstack().clone(),
        stats: exec.stats().clone(),
    }
}

#[cfg(test)]
mod test {
    use super::execute_script_no_stack_limit;
    use crate::treepp::*;

    #[test]
    fn test_script_debug() {
        let script = script! {
            OP_TRUE
            DEBUG
            OP_TRUE
            OP_VERIFY
        };
        let exec_result = execute_script(script);
        assert!(!exec_result.success);
    }

    #[test]
    fn test_script_execute_no_stack_limit() {
        let script = script! {
            for _ in 0..1001 {
                OP_1
            }
            for _ in 0..1001 {
                OP_DROP
            }
            OP_1
        };

        let exec_result = execute_script_no_stack_limit(script);
        assert!(exec_result.success);
    }
}
