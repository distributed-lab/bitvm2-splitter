use bitcoin::hashes::HashEngine;
use bitvec::{order::Lsb0, slice::BitSlice};
use std::vec::Vec;

use bitvm2_splitter::treepp::*;

/// Fixed value of $d$ specified in original doc.
///
/// This value is used to set [`BASE`] of digits the algorithm splits
/// message by.
pub const D: usize = 3;

pub const BASE: usize = (D + 1).ilog2() as usize;

/// Secret key is array of $N$ chunks by $D$ bits, where the whole number
/// of bits is equal to $v$.
#[derive(Clone, Debug)]
pub struct SecretKey<const N: usize>(Vec<[u8; N]>);

impl<const N: usize> SecretKey<N> {
    /// Construct new [`SecretKey`] from given secret parts.
    pub fn new(chunks: Vec<[u8; N]>) -> Self {
        Self(chunks)
    }

    #[cfg(feature = "rand")]
    /// Construct new [`SecretKey`] from seed, by generating required
    /// number of parts (chunks).
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

    /// Return public key derived from secret one.
    pub fn public_key<Hash, Eng>(&self) -> PublicKey<N>
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N], Engine = Eng>,
        Eng: HashEngine<MidState = [u8; N]>,
    {
        let hash_chunks = self.hashed_d_times_chunks::<Hash>();

        PublicKey::from_hashes::<Hash, Eng>(hash_chunks)
    }

    /// Return chunked public key derived from secret one.
    pub fn chunked_public_key<Hash, Eng>(&self) -> ChunkedPublicKey<N>
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N], Engine = Eng>,
        Eng: HashEngine<MidState = [u8; N]>,
    {
        let hash_chunks = self.hashed_d_times_chunks::<Hash>().into_iter().collect();

        ChunkedPublicKey::new(hash_chunks)
    }

    fn hashed_d_times_chunks<Hash>(&self) -> impl IntoIterator<Item = [u8; N]> + '_
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N]>,
    {
        self.0.iter().map(|chunk| {
            let mut chunk = *chunk;
            for _ in 0..D {
                chunk = <Hash as bitcoin::hashes::Hash>::hash(chunk.as_slice()).to_byte_array();
            }
            chunk
        })
    }

    /// Generate [`Signature`] from [`Message`].
    pub fn sign<Hash>(&self, msg: &Message) -> Signature<N>
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N]>,
    {
        let hashes = self
            .0
            .iter()
            .zip(msg.0.iter())
            .map(|(chunk, hash_times)| {
                let mut chunk = *chunk;
                for _ in 0..*hash_times {
                    chunk = <Hash as bitcoin::hashes::Hash>::hash(chunk.as_slice()).to_byte_array();
                }
                chunk
            })
            .collect::<Vec<_>>();

        Signature(hashes)
    }
}

/// Public key is a hashed $D$ times each of the $n$ parts of the
/// [`SecretKey`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkedPublicKey<const N: usize>(Vec<[u8; N]>);

impl<const N: usize> ChunkedPublicKey<N> {
    pub fn new(chunks: Vec<[u8; N]>) -> Self {
        Self(chunks)
    }

    /// Construct [`PublicKey`] from [`ChunkedPublicKey`]
    pub fn into_public_key<Hash, Eng>(self) -> PublicKey<N>
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N], Engine = Eng>,
        Eng: HashEngine<MidState = [u8; N]>,
    {
        PublicKey::from_hashes::<Hash, Eng>(self.0)
    }

    /// Verify signature for given message.    
    pub fn verify<Hash, Eng>(&self, msg: &Message, sig: &Signature<N>) -> bool
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N], Engine = Eng>,
        Eng: HashEngine<MidState = [u8; N]>,
    {
        for ((offset, sig_chunk), pubkey_chunk) in msg.0.iter().zip(sig.0.iter()).zip(self.0.iter())
        {
            let mut sig_chunk = *sig_chunk;
            for _ in 0..(D - *offset as usize) {
                sig_chunk =
                    <Hash as bitcoin::hashes::Hash>::hash(sig_chunk.as_slice()).to_byte_array();
            }
            if sig_chunk != *pubkey_chunk {
                return false;
            }
        }

        true
    }
}

/// The hash of concatenated chunks of [`ChunkedPublicKey`]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicKey<const N: usize>([u8; N]);

impl<const N: usize> PublicKey<N> {
    /// Construct [`PublicKey`] from iterator of hashes, by concatinating
    /// and hashing all sub-hashes.
    pub fn from_hashes<Hash, Eng>(chunks: impl IntoIterator<Item = [u8; N]>) -> Self
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N], Engine = Eng>,
        Eng: HashEngine<MidState = [u8; N]>,
    {
        let mut hasher = Hash::engine();

        for chunk in chunks {
            hasher.input(&chunk);
        }

        Self(hasher.midstate())
    }

    /// Verify signature for given message.    
    pub fn verify<Hash, Eng>(&self, msg: &Message, sig: &Signature<N>) -> bool
    where
        Hash: bitcoin::hashes::Hash<Bytes = [u8; N], Engine = Eng>,
        Eng: HashEngine<MidState = [u8; N]>,
    {
        // or $\hat{y}$
        let pubkey_chunks = msg.0.iter().zip(sig.0.iter()).map(|(offset, sig_chunk)| {
            let mut sig_chunk = *sig_chunk;
            for _ in 0..(D - *offset as usize) {
                sig_chunk =
                    <Hash as bitcoin::hashes::Hash>::hash(sig_chunk.as_slice()).to_byte_array();
            }
            sig_chunk
        });

        *self == (Self::from_hashes::<Hash, Eng>(pubkey_chunks))
    }
}

/// Representation of $I_d^n$ - the vector of length $n$ with bit
/// arrays of length $d$.
///
/// # Inner representation
///
/// Inner representation for now is `Vec<u8>`, which means, that each
/// "digit" is 8 bits max.
#[derive(Clone, Debug)]
pub struct Message(Vec<u8>);

impl Message {
    /// Construct the $I_d^n$ repsentation of `msg`.
    ///
    /// Due to the winternitz paper, message here is splitted into `n0` and
    /// `n1` `d+1`-base digits.
    pub fn from_bytes(msg: &[u8]) -> Self {
        if msg.is_empty() {
            return Self(Vec::new());
        }

        let mut result = Vec::with_capacity(msg.len() * 8 / D);
        let bits = BitSlice::<_, Lsb0>::from_slice(msg);

        let v = msg.len();
        // the same as v/log_2(D+1) with rounding to positive infinity.
        let n0 = v.div_ceil(BASE);

        // TODO: this is very unoptimized, so I would consider
        // reimplementing it in future.
        for chunk in bits.chunks(BASE).take(n0) {
            let mut bitbuf = 0u8;
            for (idx, bit) in chunk.iter().enumerate() {
                bitbuf |= (*bit.as_ref() as u8) << idx;
            }
            result.push(bitbuf);
        }

        let n1 = ((D * n0).ilog(D + 1) + 1) as usize;

        let checksum = ((D * n0) as u128) - result.iter().map(|v| *v as u128).sum::<u128>();

        let checksum_bytes = checksum.to_be_bytes();
        let bits = BitSlice::<_, Lsb0>::from_slice(&checksum_bytes);
        // TODO: this is very unoptimized, so I would consider
        // reimplementing it in future.
        for chunk in bits.chunks(BASE).take(n1) {
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
}

/// Winternitz signature. The array of intermidiate hashes of secret key.
#[derive(Clone, Debug)]
pub struct Signature<const N: usize>(Vec<[u8; N]>);

pub fn checksig_verify_script<const N: usize>(
    public_key: &ChunkedPublicKey<N>,
    n: usize,
    n0: usize,
    n1: usize,
) -> Script {
    script! {
        //
        // Verify the hash chain for each digit
        //

        // Repeat this for every of the n many digits
        for digit_index in 0..n {
            // Verify that the digit is in the range [0, d]
            // See https://github.com/BitVM/BitVM/issues/35
            { D }
            OP_MIN

            // Push two copies of the digit onto the altstack
            OP_DUP
            OP_TOALTSTACK
            OP_TOALTSTACK

            // Hash the input hash d times and put every result on the stack
            for _ in 0..D {
                OP_DUP OP_HASH160
            }

            // Verify the signature for this digit
            OP_FROMALTSTACK
            OP_PICK
            { public_key.0[n - 1 - digit_index].to_vec() }
            OP_EQUALVERIFY

            // Drop the d+1 stack items
            for _ in 0..(D+1)/2 {
                OP_2DROP
            }
        }

        //
        // Verify the Checksum
        //

        // 1. Compute the checksum of the message's digits
        OP_FROMALTSTACK OP_DUP OP_NEGATE
        for _ in 1..n0 {
            OP_FROMALTSTACK OP_TUCK OP_SUB
        }
        { D * n0 }
        OP_ADD


        // 2. Sum up the signed checksum's digits
        OP_FROMALTSTACK
        for _ in 0..n1 - 1 {
            for _ in 0..BASE {
                OP_DUP OP_ADD
            }
            OP_FROMALTSTACK
            OP_ADD
        }

        // 3. Ensure both checksums are equal
        OP_EQUALVERIFY


        // Convert the message's digits to bytes
        for i in 0..n0 / 2 {
            OP_SWAP
            for _ in 0..BASE {
                OP_DUP OP_ADD
            }
            OP_ADD
            // Push all bytes to the altstack, except for the last byte
            if i != (n0/2) - 1 {
                OP_TOALTSTACK
            }
        }
        // Read the bytes from the altstack
        for _ in 0..n0 / 2 - 1{
            OP_FROMALTSTACK
        }

    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "rand")]
    mod with_rand {
        use quickcheck::{Arbitrary, Gen};
        use quickcheck_macros::quickcheck;

        use super::super::*;

        use bitcoin::hashes::ripemd160::Hash as Ripemd160;
        use bitcoin::hashes::Hash;

        use rand::rngs::SmallRng;

        #[test]
        fn test_public_key_with_ripemd_160() {
            const N: usize = Ripemd160::LEN;
            const MESSAGE: &[u8] = b"Hello, world!";

            let message = Message::from_bytes(MESSAGE);

            let n = message.len();

            let secret_key = SecretKey::from_seed::<_, SmallRng>([1u8; 32], n);
            let public_key: PublicKey<N> = secret_key.public_key::<Ripemd160, _>();

            let signature = secret_key.sign::<Ripemd160>(&message);

            assert!(public_key.verify::<Ripemd160, _>(&message, &signature));
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
        fn test_any_msg_with_any_seed_works(TestInput { seed, msg }: TestInput) -> bool {
            const N: usize = Ripemd160::LEN;

            let message = Message::from_bytes(msg.as_bytes());

            let n = message.len();

            let secret_key = SecretKey::from_seed::<_, SmallRng>(seed, n);
            let public_key: PublicKey<N> = secret_key.public_key::<Ripemd160, _>();

            let signature = secret_key.sign::<Ripemd160>(&message);

            public_key.verify::<Ripemd160, _>(&message, &signature)
        }

        #[quickcheck]
        fn test_chunked_any_msg_with_any_seed_works(TestInput { seed, msg }: TestInput) -> bool {
            const N: usize = Ripemd160::LEN;

            let message = Message::from_bytes(msg.as_bytes());

            let n = message.len();

            let secret_key = SecretKey::from_seed::<_, SmallRng>(seed, n);
            let public_key: ChunkedPublicKey<N> = secret_key.chunked_public_key::<Ripemd160, _>();

            let signature = secret_key.sign::<Ripemd160>(&message);

            public_key.verify::<Ripemd160, _>(&message, &signature)
        }
    }
}
