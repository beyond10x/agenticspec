//! A small deterministic generator (splitmix64) and the draws the simulation needs.

#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut r = Self(seed ^ 0x9E37_79B9_7F4A_7C15);
        r.next_u64();
        r
    }

    /// An independent stream for `index` under `seed`.
    pub fn stream(seed: u64, index: u64) -> Self {
        let mut mix = Self(
            seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        );
        Self::new(mix.next_u64())
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    pub fn normal(&mut self) -> f64 {
        let u1 = 1.0 - self.unit();
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    pub fn exponential(&mut self, rate: f64) -> f64 {
        if rate <= 0.0 {
            return f64::INFINITY;
        }
        -(1.0 - self.unit()).ln() / rate
    }

    pub fn poisson(&mut self, mean: f64) -> u64 {
        if mean <= 0.0 {
            return 0;
        }
        if mean > 30.0 {
            return (mean + mean.sqrt() * self.normal()).round().max(0.0) as u64;
        }
        let limit = (-mean).exp();
        let mut k = 0;
        let mut p = self.unit();
        while p > limit {
            k += 1;
            p *= self.unit();
        }
        k
    }
}
