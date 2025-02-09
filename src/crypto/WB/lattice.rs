use num_integer::{ExtendedGcd, Integer};
use rand::{thread_rng, Rng};
use rand_distr::Normal;
use std::ops::{Add, Sub, Mul, Neg};

/// A wrapper for the modulus.
#[derive(Debug, Clone, Copy)]
pub struct Modulus {
    pub value: i64,
}

/// A lattice (or polynomial) vector in ℤ/(q, Xⁿ + 1).
/// The field `ntt` tells whether the vector is in “NTT domain” (true) or not.
#[derive(Debug, Clone)]
pub struct LatticeVector {
    pub vector: Vec<i64>,
    pub degree: usize,
    pub modulus: Modulus,
    pub ntt: bool,
}

impl LatticeVector {
    /// Create a new vector from a coefficient vector.
    pub fn new(vector: Vec<i64>, modulus: Modulus, ntt: bool) -> Self {
        let degree = vector.len();
        LatticeVector {
            vector,
            degree,
            modulus,
            ntt,
        }
    }

    /// Forward NTT transform.
    /// Assumes that `degree` is a power of 2 and that `root` is a primitive n‑th root
    /// of unity modulo `modulus.value`.
    pub fn goto_ntt(&mut self, root: i64) {
        if self.ntt {
            println!("This vector is already in NTT domain.");
            return;
        }
        let n = self.degree;
        let mod_val = self.modulus.value;
        self.ntt = true;
        let levels = (n as f64).log2().round() as u32;

        // Precompute twiddle factors and pre-multiply coefficients.
        let mut powtable = Vec::new();
        let mut temp = 1;
        for i in 0..n {
            self.vector[i] = (self.vector[i] * temp).rem_euclid(mod_val);
            if i % 2 == 0 {
                powtable.push(temp);
            }
            temp = (temp * root).rem_euclid(mod_val);
        }

        // Bit reversal permutation.
        for i in 0..n {
            let j = reverse_bits(i, levels as usize);
            if j > i {
                self.vector.swap(i, j);
            }
        }

        // Cooley-Tukey NTT.
        let mut size = 2;
        while size <= n {
            let halfsize = size / 2;
            let tablestep = n / size;
            for i in (0..n).step_by(size) {
                let mut k = 0;
                for j in i..(i + halfsize) {
                    let l = j + halfsize;
                    let left = self.vector[j];
                    let right = (self.vector[l] * powtable[k]).rem_euclid(mod_val);
                    self.vector[j] = (left + right).rem_euclid(mod_val);
                    self.vector[l] = (left - right).rem_euclid(mod_val);
                    k += tablestep;
                }
            }
            size *= 2;
        }
    }

    /// Inverse NTT transform.
    /// `unroot` should be the modular inverse of the root used in the forward transform,
    /// and `ninv` the modular inverse of n modulo modulus.
    pub fn goback_ntt(&mut self, unroot: i64, ninv: i64) {
        if !self.ntt {
            println!("This vector is not in NTT domain.");
            return;
        }
        let n = self.degree;
        let mod_val = self.modulus.value;
        let mut res = self.vector.clone();
        let levels = (n as f64).log2().round() as u32;

        let mut powtable = Vec::new();
        let mut powtable2 = Vec::new();
        let mut temp = 1;
        for i in 0..n {
            if i % 2 == 0 {
                powtable.push(temp);
            }
            powtable2.push(temp);
            temp = (temp * unroot).rem_euclid(mod_val);
        }

        // Bit reversal permutation.
        for i in 0..n {
            let j = reverse_bits(i, levels as usize);
            if j > i {
                res.swap(i, j);
            }
        }

        // Cooley-Tukey transform.
        let mut size = 2;
        while size <= n {
            let halfsize = size / 2;
            let tablestep = n / size;
            for i in (0..n).step_by(size) {
                let mut k = 0;
                for j in i..(i + halfsize) {
                    let l = j + halfsize;
                    let left = res[j];
                    let right = (res[l] * powtable[k]).rem_euclid(mod_val);
                    res[j] = (left + right).rem_euclid(mod_val);
                    res[l] = (left - right).rem_euclid(mod_val);
                    k += tablestep;
                }
            }
            size *= 2;
        }

        // Final adjustment.
        for i in 0..n {
            self.vector[i] = ((res[i] * ninv) % mod_val * powtable2[i]) % mod_val;
        }
        self.ntt = false;
    }
}

/// Reverse the lower `bits` bits of `x`.
fn reverse_bits(mut x: usize, bits: usize) -> usize {
    let mut y = 0;
    for _ in 0..bits {
        y = (y << 1) | (x & 1);
        x >>= 1;
    }
    y
}

//
// Arithmetic operators on LatticeVector
//

impl Add for LatticeVector {
    type Output = LatticeVector;

    fn add(self, other: LatticeVector) -> LatticeVector {
        assert_eq!(self.degree, other.degree, "Degrees must match for addition");
        let mod_val = self.modulus.value;
        let vec = self
            .vector
            .into_iter()
            .zip(other.vector.into_iter())
            .map(|(a, b)| (a + b).rem_euclid(mod_val))
            .collect();
        LatticeVector::new(vec, self.modulus, false)
    }
}

impl Sub for LatticeVector {
    type Output = LatticeVector;

    fn sub(self, other: LatticeVector) -> LatticeVector {
        assert_eq!(self.degree, other.degree, "Degrees must match for subtraction");
        let mod_val = self.modulus.value;
        let vec = self
            .vector
            .into_iter()
            .zip(other.vector.into_iter())
            .map(|(a, b)| (a - b).rem_euclid(mod_val))
            .collect();
        LatticeVector::new(vec, self.modulus, false)
    }
}

impl Mul for LatticeVector {
    type Output = LatticeVector;

    fn mul(self, other: LatticeVector) -> LatticeVector {
        assert_eq!(self.degree, other.degree, "Degrees must match for multiplication");
        let n = self.degree;
        let mod_val = self.modulus.value;
        let mut res = vec![0i64; n];
        if self.ntt {
            // Coefficient‑wise multiplication.
            for i in 0..n {
                res[i] = (self.vector[i] * other.vector[i]).rem_euclid(mod_val);
            }
        } else {
            // Convolution‑style multiplication.
            for i in 0..n {
                for j in 0..n {
                    let mut d = i + j;
                    if d < n {
                        res[d] = (res[d] + self.vector[i] * other.vector[j]).rem_euclid(mod_val);
                    } else {
                        d = d % n;
                        res[d] = (res[d] - self.vector[i] * other.vector[j]).rem_euclid(mod_val);
                    }
                }
            }
        }
        LatticeVector::new(res, self.modulus, self.ntt)
    }
}

impl Neg for LatticeVector {
    type Output = LatticeVector;

    fn neg(self) -> LatticeVector {
        let mod_val = self.modulus.value;
        let vec = self.vector.into_iter().map(|a| (-a).rem_euclid(mod_val)).collect();
        LatticeVector::new(vec, self.modulus, self.ntt)
    }
}

/// Extended Euclidean algorithm.
/// Returns an ExtendedGcd structure using the `extended_gcd` method.
pub fn xgcd(a: i64, b: i64) -> ExtendedGcd<i64> {
    a.extended_gcd(&b)
}

/// Basic CRT transform. Given x and a slice of moduli `beta`, returns
/// a vector of remainders (one per modulus).
pub fn goto_crt(x: i64, beta: &[i64]) -> Vec<i64> {
    beta.iter().map(|&b| x.rem_euclid(b)).collect()
}

/// Generate public and secret keys.
/// Returns (pka, pkb, sk) as lattice vectors.
/// Gaussian values are sampled using `rand_distr::Normal`.
pub fn key_gen(degree: usize, modulus: Modulus) -> (LatticeVector, LatticeVector, LatticeVector) {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 2.0).unwrap();

    let mut sk_vec = vec![0i64; degree];
    let mut pka_vec = vec![0i64; degree];
    let mut pkb_vec = vec![0i64; degree];

    for i in 0..degree {
        sk_vec[i] = (rng.sample(normal) as f64).round() as i64;
        pka_vec[i] = (rng.sample(normal) as f64).round() as i64;
        pkb_vec[i] = 2 * (rng.sample(normal) as f64).round() as i64;
    }

    let sk = LatticeVector::new(sk_vec, modulus, false);
    let pka = LatticeVector::new(pka_vec, modulus, false);
    let pkb = LatticeVector::new(pkb_vec, modulus, false);

    (pka, pkb, sk)
}

/// Encrypt a plaintext (as a LatticeVector) using public keys `pka` and `pkb`.
/// Returns a pair (a1, a2) as ciphertext vectors.
pub fn encrypt(
    plaintext: &LatticeVector,
    pka: &LatticeVector,
    pkb: &LatticeVector,
) -> (LatticeVector, LatticeVector) {
    let degree = plaintext.degree;
    let modulus = plaintext.modulus;
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 2.0).unwrap();

    let mut u_vec = vec![0i64; degree];
    let mut e1_vec = vec![0i64; degree];
    let mut e2_vec = vec![0i64; degree];

    for i in 0..degree {
        u_vec[i] = (rng.sample(normal) as f64).round() as i64;
        e1_vec[i] = 2 * (rng.sample(normal) as f64).round() as i64;
        e2_vec[i] = 2 * (rng.sample(normal) as f64).round() as i64;
    }

    let u = LatticeVector::new(u_vec, modulus, false);
    let e1 = LatticeVector::new(e1_vec, modulus, false);
    let e2 = LatticeVector::new(e2_vec, modulus, false);

    let a1 = pka.clone() * u.clone() + e1;
    let a2 = pkb.clone() * u + e2 + plaintext.clone();
    (a1, a2)
}
