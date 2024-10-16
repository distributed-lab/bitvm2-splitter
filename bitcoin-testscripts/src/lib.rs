#[allow(dead_code)]
// Re-export what is needed to write treepp scripts
pub mod treepp {
    pub use crate::debug::{execute_script, run};
    pub use bitcoin_script::{define_pushable, script};

    define_pushable!();
    pub use bitcoin::ScriptBuf as Script;
}

pub mod utils;

pub(crate) mod bitvm;
pub(crate) mod debug;
pub(crate) mod pseudo;

pub mod int_mul_karatsuba;
pub mod int_mul_windowed;
pub mod int_add;
pub mod sha256;

#[cfg(test)]
pub mod tests;

