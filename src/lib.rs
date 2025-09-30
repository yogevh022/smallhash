
#[inline]
pub fn u32x3_to_18_bits(v: [i32; 3]) -> u32 {
    const MASK_LOW_18: u32 = (1 << 18) - 1;

    let mut h = (v[0] as u32).wrapping_add(0x9e3779b1);
    h ^= (v[1] as u32).wrapping_mul(0x85ebca77).rotate_left(2);
    h = h.wrapping_mul(0x27d4eb2f);
    h ^= (v[2] as u32).wrapping_mul(0x85ebca77).rotate_left(13);
    h = h.wrapping_mul(0x27d4eb2f);

    h ^= h >> 1;
    h = h.wrapping_mul(0x85ebca77);
    h ^= h >> 1;

    h & MASK_LOW_18
}