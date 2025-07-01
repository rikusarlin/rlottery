use rand_xoshiro::rand_core::RngCore;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro512StarStar;

pub struct Rng(Xoshiro512StarStar);

impl Rng {
    pub fn new(seed: [u8; 64]) -> Self {
        Rng(Xoshiro512StarStar::from_seed(rand_xoshiro::Seed512(seed)))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
}
