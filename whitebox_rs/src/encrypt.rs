use crate::lattice::{Array1i64, NTRUVector};
use rand::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PubEncData {
    pub degree: usize,
    pub modulus: i64,
    pub pka: Vec<i64>,
    pub pkb: Vec<i64>,
}

// takes message bits and encrypts
pub fn encrypt_func(m_bits: &Array1i64, pka: &NTRUVector, pkb: &NTRUVector, degree: usize, modulus: i64) -> (NTRUVector, NTRUVector) {
    let mut rng = rand::thread_rng();
    let mut u = NTRUVector::new(degree, modulus, false);
    let mut e1 = NTRUVector::new(degree, modulus, false);
    let mut e2 = NTRUVector::new(degree, modulus, false);

    for i in 0..degree {
        u.vector[i] = rng.gen_range(-2..=2); // Simulate gaussian
        e1.vector[i] = 2 * rng.gen_range(-2..=2); // Simulate 2 * gaussian
        e2.vector[i] = 2 * rng.gen_range(-2..=2); // Simulate 2 * gaussian
    }

    let mut m_ntru = NTRUVector::new(degree, modulus, false);
    m_ntru.vector.assign(m_bits);

    let tmp_a1 = pka.mul(&u);
    let a1 = tmp_a1.add(&e1);

    let tmp_a2_1 = pkb.mul(&u);
    let tmp_a2_2 = tmp_a2_1.add(&e2);
    let a2 = tmp_a2_2.add(&m_ntru);

    (a1, a2)
}
