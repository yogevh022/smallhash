mod hash;
mod helpers;

use hash_small::u32x3_to_18_bits;
use crate::hash::{hash_fnv_style, hash_murmur_style, hash_pcg_style, hash_xxhash_style};
use crate::helpers::hash_thread;

const TARGET_BITS: u32 = 18;
const MASK_TARGET_BITS: u32 = (1 << TARGET_BITS) - 1;

fn main() {
    let bounds = 512;

    let murmur_handle = hash_thread("Murmur", bounds, |rng, input| {
        hash_murmur_style(input, 2, 2, 1)
    });
    let xxhash_handle = hash_thread("XXHash", bounds, |rng, input| {
        u32x3_to_18_bits(input)
    });
    let pcg_handle = hash_thread("PCG", bounds, |rng, input| {
        hash_pcg_style(input, 1, 5, 7, 14)
    });
    let fnv_handle = hash_thread("FNV", bounds, |rng, input| {
        hash_fnv_style(input, 2, 1)
    });

    let murmur = murmur_handle.join().unwrap();
    let xxhash = xxhash_handle.join().unwrap();
    let pcg = pcg_handle.join().unwrap();
    let fnv = fnv_handle.join().unwrap();

    let mut hashes = vec![murmur, xxhash, pcg, fnv];
    hashes.sort_by(|a, b| a.cv.partial_cmp(&b.cv).unwrap());

    let total_hashes = (bounds * 2).pow(3) as usize;
    let mean = total_hashes / (1 << TARGET_BITS) as usize;

    println!(
        "bounds: {}..{}, total hashes: {}, mean: {}\n",
        -bounds, bounds, total_hashes, mean
    );
    for hash in hashes {
        println!("{:?}", hash);
    }
}
