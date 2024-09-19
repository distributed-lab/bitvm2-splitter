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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{bitvm::sha256::sha256, utils::stack_to_script};
//     use bitcoin_window_mul::traits::comparable::Comparable;

//     #[test]
//     fn test_sha256() {
//         println!("sha256(32): {} bytes", sha256(32).len());
//         println!("sha256(80): {} bytes", sha256(80).len());
//         println!(
//             "sha256 chunk: {} bytes",
//             sha256_transform(8 + 16 + 64 + 1, 8 + 16).len()
//         );
//         let hex_in = "6162636462636465636465666465666765666768666768696768696a68696a6b696a6b6c6a6b6c6d6b6c6d6e6c6d6e6f6d6e6f706e6f70716f7071727071727371727374727374757374757674757677";
//         let mut hasher = Sha256::new();
//         let data = hex::decode(hex_in).unwrap();
//         hasher.update(&data);
//         let mut result = hasher.finalize();
//         hasher = Sha256::new();
//         hasher.update(result);
//         result = hasher.finalize();
//         let res = hex::encode(result);
//         let script = script! {
//             {push_bytes_hex(hex_in)}
//             {sha256(hex_in.len()/2)}
//             {sha256(32)}
//             {push_bytes_hex(res.as_str())}
//             for _ in 0..32 {
//                 OP_TOALTSTACK
//             }

//             for i in 1..32 {
//                 {i}
//                 OP_ROLL
//             }

//             for _ in 0..32 {
//                 OP_FROMALTSTACK
//                 OP_EQUALVERIFY
//             }
//             OP_TRUE
//         };
//         let res = execute_script(script);
//         assert!(res.success);
//     }
// }
