use crate::MASK_TARGET_BITS;

pub(crate) fn hash_murmur_style(v: [i32; 3], s1: u32, s2: u32, s3: u32) -> u32 {
    let x = v[0] as u32;
    let y = v[1] as u32;
    let z = v[2] as u32;

    let mut h =
        x.wrapping_mul(0xcc9e2d51) ^ y.wrapping_mul(0x1b873593) ^ z.wrapping_mul(0xe6546b64);

    h ^= h >> s1;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> s2;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> s3;

    h & MASK_TARGET_BITS
}

pub(crate) fn hash_xxhash_style(v: [i32; 3], r1: u32, r2: u32, s1: u32, s2: u32) -> u32 {
    let x = v[0] as u32;
    let y = v[1] as u32;
    let z = v[2] as u32;

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

pub(crate) fn hash_splitmix_style(v: [i32; 3], s1: u32, s2: u32, s3: u32) -> u32 {
    let x = v[0] as u64;
    let y = v[1] as u64;
    let z = v[2] as u64;

    let mut h = (x << 32) | (y << 16) | z;
    h = h.wrapping_add(0x9e3779b97f4a7c15);
    h = (h ^ (h >> s1)).wrapping_mul(0xbf58476d1ce4e5b9);
    h = (h ^ (h >> s2)).wrapping_mul(0x94d049bb133111eb);
    h ^= h >> s3;

    (h as u32) & MASK_TARGET_BITS
}

pub(crate) fn hash_pcg_style(v: [i32; 3], s1: u32, s2: u32, sh1: u32, sh2: u32) -> u32 {
    let x = v[0] as u64;
    let y = v[1] as u64;
    let z = v[2] as u64;

    let state = (x << sh1) ^ (y << sh2) ^ z;
    let state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);

    let xorshifted = (((state >> s1) ^ state) >> s2) as u32;
    let rot = (state >> 59) as u32;

    (xorshifted.rotate_right(rot)) & MASK_TARGET_BITS
}

pub(crate) fn hash_wang_style(v: [i32; 3], sh1: u32, sh2: u32, s1: u32, s2: u32, s3: u32) -> u32 {
    let x = v[0] as u32;
    let y = v[1] as u32;
    let z = v[2] as u32;

    let mut h = x ^ (y << sh1) ^ (z << sh2);
    h = (h ^ 61) ^ (h >> s1);
    h = h.wrapping_add(h << 3);
    h ^= h >> s2;
    h = h.wrapping_mul(0x27d4eb2d);
    h ^= h >> s3;

    h & MASK_TARGET_BITS
}

pub(crate) fn hash_fnv_style(v: [i32; 3], s1: u32, s2: u32) -> u32 {
    let x = v[0] as u32;
    let y = v[1] as u32;
    let z = v[2] as u32;

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