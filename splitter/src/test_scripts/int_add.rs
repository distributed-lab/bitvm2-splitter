//! This module contains the test script
//! for performing the addition of two large integers (exceeding standard Bitcoin 31-bit integers)

use crate::{
    split::script::{IOPair, SplitableScript},
    treepp::*,
};
use bitcoin_window_mul::{
    bigint::U254,
    traits::{arithmeticable::Arithmeticable, integer::NonNativeLimbInteger},
};

use core::ops::{Rem, Shl};
use num_bigint::{BigUint, RandomBits};
use num_traits::One;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Script that performs the addition of two 254-bit numbers
pub struct U254AddScript;

type IOType = u32;
const INPUT_SIZE: usize = 2 * U254::N_LIMBS;
const OUTPUT_SIZE: usize = U254::N_LIMBS;

impl SplitableScript<IOType, IOType, { INPUT_SIZE }, { OUTPUT_SIZE }> for U254AddScript {
    fn script() -> Script {
        U254::OP_ADD(1, 0)
    }

    fn generate_valid_io_pair() -> IOPair<IOType, IOType, { INPUT_SIZE }, { OUTPUT_SIZE }> {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        let a: BigUint = prng.sample(RandomBits::new(254));
        let b: BigUint = prng.sample(RandomBits::new(254));
        let c: BigUint = (a.clone() + b.clone()).rem(BigUint::one().shl(254));

        // Input is simply a and b limbs concatenated
        let mut input = a.to_u32_digits().clone();
        input.append(&mut b.to_u32_digits());

        // Output is limbs of c
        let output = c.to_u32_digits();

        IOPair {
            input: input.try_into().expect("Input is not of the correct size"),
            output: output
                .try_into()
                .expect("Output is not of the correct size"),
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
