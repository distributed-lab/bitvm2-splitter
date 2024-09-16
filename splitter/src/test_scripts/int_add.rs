//! This module contains the test script
//! for performing the addition of two large integers (exceeding standard Bitcoin 31-bit integers)

use crate::{
    split::script::{IOPair, SplitableScript},
    treepp::*,
};
use bitcoin_window_mul::{
    bigint::U254,
    traits::{
        arithmeticable::Arithmeticable,
        integer::{NonNativeInteger, NonNativeLimbInteger},
    },
};

use core::ops::{Rem, Shl};
use num_bigint::{BigUint, RandomBits};
use num_traits::One;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Script that performs the addition of two 254-bit numbers
pub struct U254AddScript;

/// Input size is double the number of limbs of U254 since we are adding two numbers
const INPUT_SIZE: usize = 2 * U254::N_LIMBS;
/// Output size is the number of limbs of U254
const OUTPUT_SIZE: usize = U254::N_LIMBS;

impl SplitableScript<{ INPUT_SIZE }, { OUTPUT_SIZE }> for U254AddScript {
    fn script() -> Script {
        U254::OP_ADD(1, 0)
    }

    fn generate_valid_io_pair() -> IOPair<{ INPUT_SIZE }, { OUTPUT_SIZE }> {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let num_1: BigUint = prng.sample(RandomBits::new(254));
        let num_2: BigUint = prng.sample(RandomBits::new(254));
        let sum: BigUint = (num_1.clone() + num_2.clone()).rem(BigUint::one().shl(254));

        IOPair {
            input: script! {
                { U254::OP_PUSH_U32LESLICE(&num_1.to_u32_digits()) }
                { U254::OP_PUSH_U32LESLICE(&num_2.to_u32_digits()) }
            },
            output: U254::OP_PUSH_U32LESLICE(&sum.to_u32_digits()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify() {
        assert!(U254AddScript::verify_random());
    }
}
