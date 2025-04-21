use crate::lattice::{Array1i64, NTRUVector};
use rand::prelude::*;
use rand_distr::Normal;

pub fn encrypt_func(m_bits: &Array1i64, pka: &NTRUVector, pkb: &NTRUVector, degree: usize, modulus: i64) -> (NTRUVector, NTRUVector) {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut u = NTRUVector::new(degree, modulus, false);
    let mut e1 = NTRUVector::new(degree, modulus, false);
    let mut e2 = NTRUVector::new(degree, modulus, false);

    for i in 0..degree {
        u.vector[i] = normal.sample(&mut rng) as i64;
        e1.vector[i] = 2 * (normal.sample(&mut rng) as i64);
        e2.vector[i] = 2 * (normal.sample(&mut rng) as i64);
    }
    
    // Update checksums after modifying the vectors
    u.update_checksum();
    e1.update_checksum();
    e2.update_checksum();

    let mut m_ntru = NTRUVector::new(degree, modulus, false);
    m_ntru.vector.assign(m_bits);
    m_ntru.update_checksum();

    let tmp_a1 = pka.mul(&u);
    let a1 = tmp_a1.add(&e1);

    let tmp_a2_1 = pkb.mul(&u);
    let tmp_a2_2 = tmp_a2_1.add(&e2);
    let a2 = tmp_a2_2.add(&m_ntru);

    (a1, a2)
}
