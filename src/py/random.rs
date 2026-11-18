//! CPython's `random.Random`, bit for bit, for integer seeds.
//!
//! PyBot picks wording with `random.Random(seed).choice(pool)`, where the seed is a sum of
//! character codes, and with the module-level `random.choice` after `random.seed(n)`. The same
//! seed has to give the same line here, so this is CPython's Mersenne Twister with its seeding
//! (`init_by_array` over the seed's 32-bit words) and its `_randbelow` (rejection sampling on
//! `getrandbits`), not a generic RNG.

use std::cell::RefCell;

const N: usize = 624;
const M: usize = 397;

#[derive(Clone, Debug)]
pub struct Random {
    mt: [u32; N],
    index: usize,
}

impl Random {
    /// `random.Random(seed)` for an integer seed (the sign is ignored, as in CPython).
    pub fn new(seed: i128) -> Random {
        let mut r = Random { mt: [0; N], index: N + 1 };
        r.seed(seed);
        r
    }

    /// A generator in the state CPython's `getstate()[1]` describes: 624 words, then the position.
    pub fn from_python_state(state: &[u64]) -> Random {
        let mut r = Random { mt: [0; N], index: N };
        for (i, w) in state.iter().take(N).enumerate() {
            r.mt[i] = *w as u32;
        }
        r.index = state.get(N).copied().unwrap_or(N as u64) as usize;
        r
    }

    /// `getstate()[1]`, the inverse of [`Random::from_python_state`].
    pub fn python_state(&self) -> Vec<u64> {
        let mut v: Vec<u64> = self.mt.iter().map(|w| *w as u64).collect();
        v.push(self.index as u64);
        v
    }

    /// `rng.seed(n)`.
    pub fn seed(&mut self, seed: i128) {
        let mut a = seed.unsigned_abs();
        let mut key = Vec::new();
        if a == 0 {
            key.push(0u32);
        }
        while a > 0 {
            key.push((a & 0xffff_ffff) as u32);
            a >>= 32;
        }
        self.init_by_array(&key);
    }

    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..N {
            let prev = self.mt[i - 1];
            self.mt[i] = 1812433253u32.wrapping_mul(prev ^ (prev >> 30)).wrapping_add(i as u32);
        }
        self.index = N;
    }

    fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19650218);
        let (mut i, mut j) = (1usize, 0usize);
        let len = key.len();
        let mut k = N.max(len);
        while k > 0 {
            let prev = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1664525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            if j >= len {
                j = 0;
            }
            k -= 1;
        }
        k = N - 1;
        while k > 0 {
            let prev = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1566083941)).wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            k -= 1;
        }
        self.mt[0] = 0x8000_0000;
        self.index = N;
    }

    fn genrand_u32(&mut self) -> u32 {
        const UPPER: u32 = 0x8000_0000;
        const LOWER: u32 = 0x7fff_ffff;
        const MATRIX_A: u32 = 0x9908_b0df;
        if self.index >= N {
            for kk in 0..N {
                let y = (self.mt[kk] & UPPER) | (self.mt[(kk + 1) % N] & LOWER);
                let mut v = self.mt[(kk + M) % N] ^ (y >> 1);
                if y & 1 != 0 {
                    v ^= MATRIX_A;
                }
                self.mt[kk] = v;
            }
            self.index = 0;
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    /// `rng.random()`: a float in [0, 1) with 53 random bits.
    pub fn random(&mut self) -> f64 {
        let a = (self.genrand_u32() >> 5) as f64;
        let b = (self.genrand_u32() >> 6) as f64;
        (a * 67108864.0 + b) * (1.0 / 9007199254740992.0)
    }

    /// `rng.getrandbits(k)` for k <= 64.
    pub fn getrandbits(&mut self, k: u32) -> u64 {
        assert!(k <= 64);
        if k == 0 {
            return 0;
        }
        if k <= 32 {
            return (self.genrand_u32() >> (32 - k)) as u64;
        }
        // CPython fills 32-bit words from the least significant end
        let lo = self.genrand_u32() as u64;
        let hi = (self.genrand_u32() >> (64 - k)) as u64;
        lo | (hi << 32)
    }

    /// `rng._randbelow(n)`: a uniform integer in [0, n).
    pub fn randbelow(&mut self, n: usize) -> usize {
        assert!(n > 0, "randbelow(0)");
        let k = usize::BITS - n.leading_zeros();
        loop {
            let r = self.getrandbits(k) as usize;
            if r < n {
                return r;
            }
        }
    }

    /// `rng.randrange(n)`.
    pub fn randrange(&mut self, n: usize) -> usize {
        self.randbelow(n)
    }

    /// `rng.randint(a, b)`.
    pub fn randint(&mut self, a: i64, b: i64) -> i64 {
        a + self.randbelow((b - a + 1) as usize) as i64
    }

    /// `rng.choice(seq)`. Panics on an empty sequence, as CPython raises.
    pub fn choice<'a, T>(&mut self, seq: &'a [T]) -> &'a T {
        &seq[self.randbelow(seq.len())]
    }

    /// `rng.shuffle(x)`.
    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        for i in (1..x.len()).rev() {
            let j = self.randbelow(i + 1);
            x.swap(i, j);
        }
    }

    /// `rng.sample(population, k)`, with CPython 3.12's choice between a pool and a set.
    pub fn sample<T: Clone>(&mut self, population: &[T], k: usize) -> Vec<T> {
        let n = population.len();
        assert!(k <= n, "sample larger than population");
        let mut setsize = 21usize;
        if k > 5 {
            // 4 ** ceil(log(k * 3, 4))
            let e = ((k * 3) as f64).ln() / 4f64.ln();
            setsize += 4usize.pow(e.ceil() as u32);
        }
        let mut result = Vec::with_capacity(k);
        if n <= setsize {
            let mut pool: Vec<T> = population.to_vec();
            for i in 0..k {
                let j = self.randbelow(n - i);
                result.push(pool[j].clone());
                pool[j] = pool[n - i - 1].clone();
            }
        } else {
            let mut selected = std::collections::HashSet::new();
            for _ in 0..k {
                let mut j = self.randbelow(n);
                while selected.contains(&j) {
                    j = self.randbelow(n);
                }
                selected.insert(j);
                result.push(population[j].clone());
            }
        }
        result
    }
}

thread_local! {
    /// The module-level generator behind `random.choice` and `random.seed`.
    static GLOBAL: RefCell<Random> = RefCell::new(Random::new(0));
}

/// `random.seed(n)`.
pub fn seed(n: i128) {
    GLOBAL.with(|g| g.borrow_mut().seed(n));
}

/// `random.choice(seq)` on the module-level generator.
pub fn choice<T: Clone>(seq: &[T]) -> T {
    GLOBAL.with(|g| g.borrow_mut().choice(seq).clone())
}

/// `random.randrange(n)` on the module-level generator.
pub fn randrange(n: usize) -> usize {
    GLOBAL.with(|g| g.borrow_mut().randbelow(n))
}

/// `random.getstate()`.
pub fn getstate() -> Random {
    GLOBAL.with(|g| g.borrow().clone())
}

/// `random.setstate(state)`.
pub fn setstate(state: Random) {
    GLOBAL.with(|g| *g.borrow_mut() = state);
}

/// The seed most of PyBot's pickers use: `sum(ord(c) for c in text)`.
pub fn ord_sum(text: &str) -> i128 {
    text.chars().map(|c| c as i128).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values from CPython 3.12 (see harness/random_check.py).
    #[test]
    fn matches_cpython() {
        let mut r = Random::new(0);
        assert_eq!(r.random(), 0.8444218515250481);
        let mut r = Random::new(12345);
        let v: Vec<usize> = (0..5).map(|_| r.randbelow(10)).collect();
        assert_eq!(v, vec![6, 0, 4, 5, 3]);
    }
}
