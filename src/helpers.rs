use crate::TARGET_BITS;
use std::fmt::Debug;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use rand::rngs::ThreadRng;
use crate::hash::hash_xxhash_style;

#[derive(Default)]
pub(crate) struct HashStats {
    pub name: Option<&'static str>,
    pub duration: Option<Duration>,
    pub unique: usize,
    pub variance: f32,
    pub std_dev: f32,
    pub cv: f32,
}

impl HashStats {
    pub(crate) fn new(hashes: &[u32], bounds: i32) -> Self {
        let bounds2 = bounds * 2;
        let combinations = bounds2 * bounds2 * bounds2;
        let unique = hashes.iter().filter(|&&c| c > 0).count();
        let mean = combinations as f32 / (hashes.len() - 1) as f32;
        let variance: f32 = hashes
            .iter()
            .map(|&c| {
                let diff = c as f32 - mean;
                diff * diff
            })
            .sum::<f32>()
            / hashes.len() as f32;
        let std_dev = variance.sqrt();
        let cv = std_dev / mean;

        Self {
            name: None,
            duration: None,
            unique,
            variance,
            std_dev,
            cv,
        }
    }
}

impl Debug for HashStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: unique: {}, dev: {:.3}, cv: {:.6}, time: {:?}",
            self.name.unwrap_or("unknown"),
            self.unique,
            self.std_dev,
            self.cv,
            self.duration.unwrap_or(Duration::from_secs(0)),
        )
    }
}

#[derive(Debug, Default)]
pub(crate) struct HashConsts {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
}

pub(crate) fn ivec_range<F>(bounds: i32, mut f: F) -> HashStats
where
    F: FnMut([i32; 3]) -> u32,
{
    let mut hashes = [0u32; (1 << TARGET_BITS) + 1];
    for x in -bounds..bounds {
        for y in -bounds..bounds {
            for z in -bounds..bounds {
                let hash = f([x, y, z]);
                hashes[hash as usize] += 1;
            }
        }
    }
    HashStats::new(&hashes, bounds)
}

pub(crate) fn permute5<F>(
    a_max: u32,
    b_max: u32,
    c_max: u32,
    d_max: u32,
    e_max: u32,
    mut f: F,
) -> (HashConsts, HashStats)
where
    F: FnMut(u32, u32, u32, u32, u32) -> HashStats,
{
    let mut best = HashStats {
        cv: f32::MAX,
        ..Default::default()
    };
    let mut best_const = HashConsts {
        ..Default::default()
    };
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
        println!("a: {}/{}", a, a_max);
    }
    (best_const, best)
}

pub(crate) fn hash_thread<F>(name: &'static str, bounds: i32, mut f: F) -> JoinHandle<HashStats>
where
    F: FnMut(&mut ThreadRng, [i32; 3]) -> u32 + Send + 'static,
{
    thread::spawn(move || {
        println!("{} thread started.", name);
        let mut rng = rand::rng();
        let start = Instant::now();
        let mut hash_stats = ivec_range(bounds, |input| f(&mut rng, input));
        hash_stats.name = Some(name);
        hash_stats.duration = Some(start.elapsed());
        hash_stats
    })
}
