use numpy::ndarray::Array1;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub type Array1i64 = Array1<i64>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NTRUVector {
    pub vector: Array1i64,
    pub degree: usize,
    pub modulus: i64,
    pub ntt: bool,
}

impl NTRUVector {
    pub fn new(degree: usize, modulus: i64, ntt: bool) -> Self {
        NTRUVector {
            vector: Array1::zeros(degree),
            degree,
            modulus,
            ntt,
        }
    }

    pub fn add(&self, other: &NTRUVector) -> Self {
        let mut res = NTRUVector::new(self.degree, self.modulus, self.ntt);
        res.vector = self.vector.iter().zip(other.vector.iter())
            .map(|(s, o)| (s + (o % self.modulus)).rem_euclid(self.modulus))
            .collect();
        res
    }

    pub fn mul(&self, other: &NTRUVector) -> Self {
        let mut res = NTRUVector::new(self.degree, self.modulus, self.ntt);
        if self.ntt {
            res.vector = self.vector.iter().zip(other.vector.iter())
                .map(|(s, o)| (s * o).rem_euclid(self.modulus))
                .collect();
        } else {
            for i in 0..self.degree {
                for j in 0..self.degree {
                    let d = i + j;
                    if d < self.degree {
                        res.vector[d] = (res.vector[d] + self.vector[i] * other.vector[j]).rem_euclid(self.modulus);
                    } else {
                        let d_mod = d % self.degree;
                        res.vector[d_mod] = (res.vector[d_mod] - self.vector[i] * other.vector[j]).rem_euclid(self.modulus);
                    }
                }
            }
        }
        res
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubEncData {
    pub degree: usize,
    pub modulus: i64,
    pub pka: NTRUVector,
    pub pkb: NTRUVector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteData {
    pub root: i64,
    pub unroot: i64,
    pub ninv: i64,
    pub beta: Vec<i64>,
    pub beta_p: Vec<i64>,
    pub k: usize,
    pub mask: Vec<i64>,
    pub rotate: usize,
    pub chal: u8,
    pub fb: HashMap<String, Vec<Vec<i64>>>,
    pub sb: HashMap<String, Vec<Vec<i64>>>,
}
