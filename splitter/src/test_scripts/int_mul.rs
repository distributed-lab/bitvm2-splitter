//! This module contains the test script
//! for performing the multiplication of two large integers
//! (exceeding standard Bitcoin 31-bit integers)

use crate::{
    split::script::{IOPair, SplitableScript},
    treepp::*,
};
use bitcoin_window_mul::{
    bigint::{U254Windowed, U508},
    traits::integer::{NonNativeInteger, NonNativeLimbInteger},
};

use num_bigint::{BigUint, RandomBits};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Script that performs the addition of two 254-bit numbers
pub struct U254MulScript;

/// Input size is double the number of limbs of U254 since we are multiplying two numbers
const INPUT_SIZE: usize = 2 * U254Windowed::N_LIMBS;
/// Output size is the number of limbs of U508
const OUTPUT_SIZE: usize = U508::N_LIMBS;

impl SplitableScript<{ INPUT_SIZE }, { OUTPUT_SIZE }> for U254MulScript {
    fn script() -> Script {
        U254Windowed::OP_WIDENINGMUL::<U508>()
    }

    fn generate_valid_io_pair() -> IOPair<{ INPUT_SIZE }, { OUTPUT_SIZE }> {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        // Generate two random 254-bit numbers and calculate their sum
        let num_1: BigUint = prng.sample(RandomBits::new(254));
        let num_2: BigUint = prng.sample(RandomBits::new(254));
        let product: BigUint = num_1.clone() * num_2.clone();

        IOPair {
            input: script! {
                { U254Windowed::OP_PUSH_U32LESLICE(&num_1.to_u32_digits()) }
                { U254Windowed::OP_PUSH_U32LESLICE(&num_2.to_u32_digits()) }
            },
            output: U508::OP_PUSH_U32LESLICE(&product.to_u32_digits()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify() {
        assert!(U254MulScript::verify_random());
    }

    #[test]
    fn test_split() {
        // First, we generate the pair of input and output scripts
        let IOPair { input, output } = U254MulScript::generate_valid_io_pair();

        let split_result = U254MulScript::split(input);
        for shard in split_result.shards {
            println!("Shard is of length {}", shard.len());
        }

        println!("Intermediate results:");
        for intermediate_result in split_result.intermediate_states {
            println!(
                "Intermediate results: {};{}",
                intermediate_result.stack.len(),
                intermediate_result.altstack.len()
            );
        }
    }
}
