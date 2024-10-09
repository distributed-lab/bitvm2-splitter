use bitvec::{order::Lsb0, slice::BitSlice};
use std::vec::Vec;

/// Secret key is array of $N$ chunks by $D$ bits, where the whole number
/// of bits is equal to $v$.
#[derive(Clone, Debug)]
pub struct SecretKey<const N: usize>(Vec<[u8; N]>);

impl<const N: usize> SecretKey<N> {
    pub fn new(chunks: Vec<[u8; N]>) -> Self {
        Self(chunks)
    }

    #[cfg(feature = "rand")]
    pub fn from_seed<Seed, Rng>(seed: Seed, chunks_num: usize) -> Self
    where
        Seed: Sized + Default + AsMut<[u8]>,
        Rng: rand::SeedableRng<Seed = Seed> + rand::Rng,
    {
        let mut rng = Rng::from_seed(seed);

        let mut chunks = Vec::new();

        for _ in 0..chunks_num {
            chunks.push(rng.sample(rand::distributions::Standard));
        }

        Self(chunks)
    }

    pub fn public_key<Hash>(&self) -> PublicKey<N>
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N]>,
    {
        PublicKey(
            self.0
                .iter()
                .map(|chunk| {
                    let mut chunk = *chunk;
                    for _ in 0..D {
                        chunk =
                            <Hash as bitcoin::hashes::Hash>::hash(chunk.as_slice()).to_byte_array();
                    }
                    chunk
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn sign<Hash>(&self, msg: &Message) -> Signature<N>
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N]>,
    {
        let hash_offsets = msg.to_offsets();

        let hashes = self
            .0
            .iter()
            .zip(hash_offsets)
            .map(|(chunk, times)| {
                let mut chunk = *chunk;
                for _ in 0..times {
                    chunk = <Hash as bitcoin::hashes::Hash>::hash(chunk.as_slice()).to_byte_array();
                }
                chunk
            })
            .collect::<Vec<_>>();

        Signature(hashes)
    }
}

/// Public key is hashed $D$ times each of the $N$ chunks of the
/// [`SecretKey`].
#[derive(Clone, Debug)]
pub struct PublicKey<const N: usize>(Vec<[u8; N]>);

impl<const N: usize> PublicKey<N> {
    pub fn verify<Hash>(&self, msg: &Message, sig: &Signature<N>) -> bool
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N]>,
    {
        let offsets = msg.to_offsets();

        for ((pubkey_chunk, offset), sig_chunk) in
            self.0.iter().zip(offsets.into_iter()).zip(sig.0.iter())
        {
            let mut sig_chunk = *sig_chunk;
            for _ in 0..(D - offset) {
                sig_chunk =
                    <Hash as bitcoin::hashes::Hash>::hash(sig_chunk.as_slice()).to_byte_array();
            }

            if *pubkey_chunk != sig_chunk {
                return false;
            }
        }

        true
    }
}

pub const D: usize = 3;
pub const BASE: usize = (D + 1).ilog2() as usize;

/// Representation of $I_d^n$ - the vector of length $n$ with bit
/// arrays of lentg $d$.
///
/// # Inner representation
///
/// Inner representation for now is `Vec<u8>`, which means, that each
/// "digit" is 8 bits max.
#[derive(Clone, Debug)]
pub struct Message(Vec<u8>);

impl Message {
    pub fn from_bytes(msg: &[u8]) -> Self {
        let mut result = Vec::with_capacity(msg.len() * 8 / D);
        let bits = BitSlice::<_, Lsb0>::from_slice(msg);

        // TODO: this is very unoptimized, so I would consider
        // reimplementing it in future.
        for chunk in bits.chunks(BASE) {
            let mut bitbuf = 0u8;
            for (idx, bit) in chunk.iter().enumerate() {
                bitbuf |= (*bit.as_ref() as u8) << idx;
            }
            result.push(bitbuf);
        }

        Self(result)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn to_offsets(&self) -> Vec<usize> {
        self.0.iter().map(|chunk| *chunk as usize).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Signature<const N: usize>(Vec<[u8; N]>);

#[cfg(test)]
mod tests {

    #[cfg(feature = "rand")]
    mod with_rand {
        use quickcheck::{Arbitrary, Gen};
        use quickcheck_macros::quickcheck;

        use super::super::*;

        use bitcoin::hashes::hash160::Hash as Hash160;
        use bitcoin::hashes::ripemd160::Hash as Ripemd160;
        use bitcoin::hashes::Hash;

        use rand::rngs::SmallRng;

        #[test]
        fn test_with_rand_public_key_with_ripemd_160() {
            const N: usize = Ripemd160::LEN;
            const MESSAGE: &[u8] = b"Hello, world!";

            let message = Message::from_bytes(MESSAGE);

            let n = message.len();

            let secret_key = SecretKey::from_seed::<_, SmallRng>([1u8; 32], n);
            let public_key: PublicKey<N> = secret_key.public_key::<Ripemd160>();

            let signature = secret_key.sign::<Ripemd160>(&message);

            assert!(public_key.verify::<Ripemd160>(&message, &signature));
        }

        #[derive(Clone, Debug)]
        struct TestInput {
            seed: [u8; 32],
            msg: String,
        }

        impl Arbitrary for TestInput {
            fn arbitrary(g: &mut Gen) -> Self {
                TestInput {
                    seed: [(); 32].map(|_| u8::arbitrary(g)),
                    msg: String::arbitrary(g),
                }
            }
        }

        #[quickcheck]
        fn any_msg_with_any_seed_works(TestInput { seed, msg }: TestInput) -> bool {
            const N: usize = Hash160::LEN;

            let message = Message::from_bytes(msg.as_bytes());

            let n = message.len();

            let secret_key = SecretKey::from_seed::<_, SmallRng>(seed, n);
            let public_key: PublicKey<N> = secret_key.public_key::<Hash160>();

            let signature = secret_key.sign::<Hash160>(&message);

            public_key.verify::<Hash160>(&message, &signature)
        }
    }
}
