use crate::bitvm::bigint::BigIntImpl;
// utils for push fields into stack
use crate::bitvm::bn254::fq::bigint_to_u32_limbs;
use ark_ff::BigInt;
use num_bigint::BigUint;

use crate::bitvm::bn254::{fp254impl::Fp254Impl, fq::Fq};

use bitcoin_utils::treepp::*;

pub fn fq_push(element: ark_bn254::Fq) -> Script {
    script! {
        { Fq::push_u32_le(&BigUint::from(element).to_u32_digits()) }
    }
}

pub fn fq_push_not_montgomery(element: ark_bn254::Fq) -> Script {
    script! {
        { Fq::push_u32_le_not_montgomery(&BigUint::from(element).to_u32_digits()) }
    }
}

pub enum Hint {
    Fq(ark_bn254::Fq),
    BigIntegerTmulLC1(num_bigint::BigInt),
    BigIntegerTmulLC2(num_bigint::BigInt),
}

impl Hint {
    pub fn push(&self) -> Script {
        const K1: (u32, u32) = Fq::bigint_tmul_lc_1();
        const K2: (u32, u32) = Fq::bigint_tmul_lc_2();
        pub type T1 = BigIntImpl<{ K1.0 }, { K1.1 }>;
        pub type T2 = BigIntImpl<{ K2.0 }, { K2.1 }>;
        match self {
            Hint::Fq(fq) => script! {
                { fq_push_not_montgomery(*fq) }
            },
            Hint::BigIntegerTmulLC1(a) => script! {
                { T1::push_u32_le(&bigint_to_u32_limbs(a.clone(), T1::N_BITS)) }
            },
            Hint::BigIntegerTmulLC2(a) => script! {
                { T2::push_u32_le(&bigint_to_u32_limbs(a.clone(), T2::N_BITS)) }
            },
        }
    }
}

pub fn fq_to_bits(fq: BigInt<4>, limb_size: usize) -> Vec<u32> {
    let mut bits: Vec<bool> = ark_ff::BitIteratorBE::new(fq.as_ref()).skip(2).collect();
    bits.reverse();

    bits.chunks(limb_size)
        .map(|chunk| {
            let mut factor = 1;
            let res = chunk.iter().fold(0, |acc, &x| {
                let r = acc + if x { factor } else { 0 };
                factor *= 2;
                r
            });
            res
        })
        .collect()
}
