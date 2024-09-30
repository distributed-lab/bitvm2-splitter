//! This module contains the test script
//! for performing the addition of two large integers (exceeding standard Bitcoin 31-bit integers)

use crate::{
    split::script::{IOPair, SplitableScript},
    treepp::*,
};
use bitcoin_window_mul::{
    bigint::{U254Windowed, U254},
    traits::{arithmeticable::Arithmeticable, integer::NonNativeInteger},
};

use core::ops::{Rem, Shl};
use num_bigint::{BigUint, RandomBits};
use num_traits::One;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Script that performs the addition of two 254-bit numbers
pub struct U254AddScript;

impl SplitableScript for U254AddScript {
    fn script() -> Script {
        U254::OP_ADD(1, 0)
    }

    fn generate_valid_io_pair() -> IOPair {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a: BigUint = prng.sample(RandomBits::new(254));
        let b: BigUint = prng.sample(RandomBits::new(254));
        let c: BigUint = (a.clone() + b.clone()).rem(BigUint::one().shl(254));

        println!("a: {}", a);
        println!("b: {}", b);
        println!("c: {}", c);

        IOPair {
            input: script! {
                { U254::OP_PUSH_U32LESLICE(&a.to_u32_digits()) }
                { U254::OP_PUSH_U32LESLICE(&b.to_u32_digits()) }
            },
            output: U254::OP_PUSH_U32LESLICE(&c.to_u32_digits()),
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
