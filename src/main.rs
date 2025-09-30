use glam::IVec3;
use std::thread;

const TARGET_BITS: u32 = 18;
const MASK_TARGET_BITS: u32 = (1 << TARGET_BITS) - 1;

// 1. MurmurHash3-inspired finalizer
pub fn hash_murmur_style(v: IVec3, s1: u32, s2: u32, s3: u32) -> u32 {
    let x = v.x as u32;
    let y = v.y as u32;
    let z = v.z as u32;

    let mut h = x.wrapping_mul(0xcc9e2d51)
        ^ y.wrapping_mul(0x1b873593)
        ^ z.wrapping_mul(0xe6546b64);

    h ^= h >> s1;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> s2;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> s3;

    h & MASK_TARGET_BITS
}

// 2. xxHash-inspired mixer
pub fn hash_xxhash_style(v: IVec3, r1: u32, r2: u32, s1: u32, s2: u32) -> u32 {
    let x = v.x as u32;
    let y = v.y as u32;
    let z = v.z as u32;

    let mut h = x.wrapping_add(0x9e3779b1);
    h ^= y.wrapping_mul(0x85ebca77).rotate_left(r1);
    h = h.wrapping_mul(0x27d4eb2f);
    h ^= z.wrapping_mul(0x85ebca77).rotate_left(r2);
    h = h.wrapping_mul(0x27d4eb2f);

    h ^= h >> s1;
    h = h.wrapping_mul(0x85ebca77);
    h ^= h >> s2;

    h & MASK_TARGET_BITS
}

// 3. SplitMix-inspired
pub fn hash_splitmix_style(v: IVec3, s1: u32, s2: u32, s3: u32) -> u32 {
    let x = v.x as u64;
    let y = v.y as u64;
    let z = v.z as u64;

    let mut h = (x << 32) | (y << 16) | z;
    h = h.wrapping_add(0x9e3779b97f4a7c15);
    h = (h ^ (h >> s1)).wrapping_mul(0xbf58476d1ce4e5b9);
    h = (h ^ (h >> s2)).wrapping_mul(0x94d049bb133111eb);
    h ^= h >> s3;

    (h as u32) & MASK_TARGET_BITS
}

// 4. PCG-inspired
pub fn hash_pcg_style(v: IVec3, s1: u32, s2: u32, sh1: u32, sh2: u32) -> u32 {
    let x = v.x as u64;
    let y = v.y as u64;
    let z = v.z as u64;

    let state = (x << sh1) ^ (y << sh2) ^ z;
    let state = state.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);

    let xorshifted = (((state >> s1) ^ state) >> s2) as u32;
    let rot = (state >> 59) as u32;

    (xorshifted.rotate_right(rot)) & MASK_TARGET_BITS
}

// 5. Wang hash variant
pub fn hash_wang_style(v: IVec3, sh1: u32, sh2: u32, s1: u32, s2: u32, s3: u32) -> u32 {
    let x = v.x as u32;
    let y = v.y as u32;
    let z = v.z as u32;

    let mut h = x ^ (y << sh1) ^ (z << sh2);
    h = (h ^ 61) ^ (h >> s1);
    h = h.wrapping_add(h << 3);
    h ^= h >> s2;
    h = h.wrapping_mul(0x27d4eb2d);
    h ^= h >> s3;

    h & MASK_TARGET_BITS
}

// 6. FNV-inspired with mixing
pub fn hash_fnv_style(v: IVec3, s1: u32, s2: u32) -> u32 {
    let x = v.x as u32;
    let y = v.y as u32;
    let z = v.z as u32;

    const FNV_PRIME: u32 = 0x01000193;
    const FNV_OFFSET: u32 = 0x811c9dc5;

    let mut h = FNV_OFFSET;
    h = (h ^ x).wrapping_mul(FNV_PRIME);
    h = (h ^ y).wrapping_mul(FNV_PRIME);
    h = (h ^ z).wrapping_mul(FNV_PRIME);

    h ^= h >> s1;
    h = h.wrapping_mul(0xc2b2ae3d);
    h ^= h >> s2;

    h & MASK_TARGET_BITS
}

#[derive(Debug, Default)]
struct HashStats {
    mean: f32,
    variance: f32,
    std_dev: f32,
    cv: f32,
}

#[derive(Debug, Default)]
struct HashConsts {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
}

fn ivec_range<F>(bounds: i32, mut f: F) -> HashStats
where
    F: FnMut(IVec3) -> u32,
{
    let mut hashes = [0u32; (1 << TARGET_BITS) + 1];
    for x in -bounds..bounds {
        for y in -bounds..bounds {
            for z in -bounds..bounds {
                let ivec = IVec3::new(x, y, z);
                let hash = f(ivec);
                hashes[hash as usize] += 1;
            }
        }
    }
    let bounds2 = bounds * 2;
    let combinations = bounds2 * bounds2 * bounds2;
    let mean = combinations as f32 / (hashes.len() - 1) as f32;
    let variance: f32 = hashes.iter()
        .map(|&c| {
            let diff = c as f32 - mean;
            diff * diff
        })
        .sum::<f32>() / hashes.len() as f32;
    let std_dev = variance.sqrt();
    let cv = std_dev / mean;

    HashStats {
        mean,
        variance,
        std_dev,
        cv,
    }
}

fn permute5<F>(a_max: u32, b_max: u32, c_max: u32, d_max: u32, e_max: u32, mut f: F) -> (HashConsts, HashStats)
where
    F: FnMut(u32, u32, u32, u32, u32) -> HashStats,
{
    let mut best = HashStats { cv: f32::MAX, ..Default::default() };
    let mut best_const = HashConsts { ..Default::default() };
    for a in 0..=a_max {
        for b in 0..=b_max {
            for c in 0..=c_max {
                for d in 0..=d_max {
                    for e in 0..=e_max {
                        let hash = f(a, b, c, d, e);
                        if hash.cv < best.cv {
                            best = hash;
                            best_const = HashConsts { a, b, c, d, e };
                        }
                    }
                }
            }
        }
    }
    (best_const, best)
}

fn main() {
    let const_max = 16;
    let bounds = 32;

    let murmur_handle = thread::spawn(move || {
        permute5(const_max, const_max, const_max, 0, 0, |a, b, c, d, e| {
            ivec_range(bounds, |ivec| hash_murmur_style(ivec, a, b, c))
        })
    });

    let xxhash_handle = thread::spawn(move || {
        permute5(const_max, const_max, const_max, const_max, 0, |a, b, c, d, e| {
            ivec_range(bounds, |ivec| hash_xxhash_style(ivec, a, b, c, d))
        })
    });

    let splitmix_handle = thread::spawn(move || {
        permute5(const_max, const_max, const_max, 0, 0, |a, b, c, d, e| {
            ivec_range(bounds, |ivec| hash_splitmix_style(ivec, a, b, c))
        })
    });

    let pcg_handle = thread::spawn(move || {
        permute5(const_max, const_max, const_max, const_max, 0, |a, b, c, d, e| {
            ivec_range(bounds, |ivec| hash_pcg_style(ivec, a, b, c, d))
        })
    });

    let wang_handle = thread::spawn(move || {
        permute5(const_max, const_max, const_max, const_max, const_max, |a, b, c, d, e| {
            ivec_range(bounds, |ivec| hash_wang_style(ivec, a, b, c, d, e))
        })
    });

    let fnv_handle = thread::spawn(move || {
        permute5(const_max, const_max, 0, 0, 0, |a, b, c, d, e| {
            ivec_range(bounds, |ivec| hash_fnv_style(ivec, a, b))
        })
    });

    println!("spawned threads.");

    let murmur = murmur_handle.join().unwrap();
    let xxhash = xxhash_handle.join().unwrap();
    let splitmix = splitmix_handle.join().unwrap();
    let pcg = pcg_handle.join().unwrap();
    let wang = wang_handle.join().unwrap();
    let fnv = fnv_handle.join().unwrap();

    dbg!(murmur);
    dbg!(xxhash);
    dbg!(splitmix);
    dbg!(pcg);
    dbg!(wang);
    dbg!(fnv);
}
