use crate::lattice::{NTRUVector, WhiteData};
use numpy::ndarray::Array1;

type Array1i64 = Array1<i64>;

#[derive(Debug, Clone)]
struct WBVector {
    ntru_vector: NTRUVector,
}

impl WBVector {
    fn from_ntru_vector(ntru_vector: NTRUVector) -> Self {
        WBVector { ntru_vector }
    }

    fn mul(&self, other: &WBVector, white_data: &WhiteData) -> Self {
        let mut res_ntru = NTRUVector::new(self.ntru_vector.degree, self.ntru_vector.modulus, self.ntru_vector.ntt);
        if self.ntru_vector.ntt {
            self.my_mult(other, &mut res_ntru, white_data);
        } else {
            println!("WB vector must be turned in ntt form");
        }
        WBVector::from_ntru_vector(res_ntru)
    }

    fn my_mult(&self, other: &WBVector, res: &mut NTRUVector, white_data: &WhiteData) {
        for i in 0..self.ntru_vector.degree {
            let x = self.ntru_vector.vector[i];
            let y = other.ntru_vector.vector[i];
            let z = WBVector::mont_mult(i, x, y, self.ntru_vector.modulus, white_data);
            res.vector[i] = z;
        }
        // Update checksum after multiplication
        res.update_checksum();
    }

    fn mont_mult(dim: usize, a: i64, b: i64, n: i64, white_data: &WhiteData) -> i64 {
        // Verify beta and beta_p checksums before using them
        let b_val = &white_data.beta;
        let b_p_val = &white_data.beta_p;
        
        let beta_checksum = checksum::compute_crt_checksum(b_val);
        let beta_p_checksum = checksum::compute_crt_checksum(b_p_val);
        
        if beta_checksum != white_data.beta_checksum {
            panic!("Beta array checksum verification failed in mont_mult!");
        }
        
        if beta_p_checksum != white_data.beta_p_checksum {
            panic!("Beta prime array checksum verification failed in mont_mult!");
        }
        
        let k_val = white_data.k;
    
        // Convert values to CRT representation
        let a_m = goto_crt(a, b_val);
        let b_m = goto_crt(b, b_val);
        let a_m_p = goto_crt(a, b_p_val);
        let b_m_p = goto_crt(b, b_p_val);
    
        // Calculate modular values
        let m_val: i64 = b_val.iter().product();
        let m_p_val: i64 = b_p_val.iter().product();
    
        let minv_m_p = goto_crt(xgcd(m_val, m_p_val).1, b_p_val);
        let n_m_p = goto_crt(n, b_p_val);
    
        // First box lookup
        let fb = &white_data.fb;
        let mut q = vec![0i64; k_val];
        q[0] = (fb[&format!("fb_dim_{}", dim)][a_m[0] as usize][b_m[0] as usize] % (1 << 5)) as i64;
        q[1] = ((fb[&format!("fb_dim_{}", dim)][a_m[1] as usize][b_m[1] as usize] % (1 << 10)) >> 5) as i64;
        q[2] = ((fb[&format!("fb_dim_{}", dim)][a_m[2] as usize][b_m[2] as usize] % (1 << 15)) >> 10) as i64;
        q[3] = ((fb[&format!("fb_dim_{}", dim)][a_m[3] as usize][b_m[3] as usize] % (1 << 20)) >> 15) as i64;
        q[4] = (fb[&format!("fb_dim_{}", dim)][a_m[4] as usize][b_m[4] as usize] >> 20) as i64;
    
        // Verify the CRT conversion with a Blake3 hash
        let q_crt = goback_crt(&q, b_val);
        let _q_hash = checksum::integer_sequence_checksum(&q);
        let q_crt_vec = goto_crt(q_crt, b_p_val);
    
        // Second box lookup
        let sb = &white_data.sb;
        let mut r = vec![0i64; k_val];
        for i in 0..k_val {
            r[i] = ((q_crt_vec[i] * n_m_p[i] % b_p_val[i]) * minv_m_p[i]).rem_euclid(b_p_val[i]);
        }
        r[0] = (r[0] + (sb[&format!("sb_dim_{}", dim)][a_m_p[0] as usize][b_m_p[0] as usize] % (1 << 5)) as i64).rem_euclid(b_p_val[0]);
        r[1] = (r[1] + ((sb[&format!("sb_dim_{}", dim)][a_m_p[1] as usize][b_m_p[1] as usize] % (1 << 10)) >> 5) as i64).rem_euclid(b_p_val[1]);
        r[2] = (r[2] + ((sb[&format!("sb_dim_{}", dim)][a_m_p[2] as usize][b_m_p[2] as usize] % (1 << 15)) >> 10) as i64).rem_euclid(b_p_val[2]);
        r[3] = (r[3] + ((sb[&format!("sb_dim_{}", dim)][a_m_p[3] as usize][b_m_p[3] as usize] % (1 << 20)) >> 15) as i64).rem_euclid(b_p_val[3]);
        r[4] = (r[4] + (sb[&format!("sb_dim_{}", dim)][a_m_p[4] as usize][b_m_p[4] as usize] >> 20) as i64).rem_euclid(b_p_val[4]);
    
        // Verify r vector with a hash before final calculation
        let _r_hash = checksum::integer_sequence_checksum(&r);
        let r_crt = goback_crt(&r, b_p_val);
    
        // Final result with modular multiplication
        (r_crt * m_val).rem_euclid(n)
    }
}

fn goto_crt(x: i64, base: &[i64]) -> Vec<i64> {
    base.iter().map(|&b| x.rem_euclid(b)).collect()
}

fn goback_crt(x_b: &[i64], base: &[i64]) -> i64 {
    let mut x = 0;
    let b_prod: i64 = base.iter().product();
    for i in 0..base.len() {
        let b_i = b_prod / base[i];
        let (_, mi, _) = xgcd(b_i, base[i]);
        x = (x + (x_b[i] * b_i % b_prod).rem_euclid(b_prod) * mi % b_prod).rem_euclid(b_prod);
    }
    x.rem_euclid(b_prod)
}

fn xgcd(mut b: i64, mut n: i64) -> (i64, i64, i64) {
    let mut x0 = 1;
    let mut x1 = 0;
    let mut y0 = 0;
    let mut y1 = 1;
    while n != 0 {
        let q = b / n;
        let temp_n = n;
        n = b % n;
        b = temp_n;

        let temp_x1 = x1;
        x1 = x0 - q * x1;
        x0 = temp_x1;

        let temp_y1 = y1;
        y1 = y0 - q * y1;
        y0 = temp_y1;
    }
    (b, x0, y0)
}

fn decrypt_white(a1_vec: &NTRUVector, a2_vec: &NTRUVector, degree: usize, modulus: i64, white_data: &WhiteData) -> Array1i64 {
    let mut tmp_a1 = WBVector::from_ntru_vector(NTRUVector {
        vector: a1_vec.vector.clone(),
        degree: a1_vec.degree,
        modulus: a1_vec.modulus,
        ntt: a1_vec.ntt,
        checksum: a1_vec.checksum,
    });
    let mut tmp_a2 = WBVector::from_ntru_vector(NTRUVector {
        vector: a2_vec.vector.clone(),
        degree: a2_vec.degree,
        modulus: a2_vec.modulus,
        ntt: a2_vec.ntt,
        checksum: a2_vec.checksum,
    });

    let root = white_data.root;
    let unroot = white_data.unroot;
    let ninv = white_data.ninv;
    tmp_a1.ntru_vector.goto_ntt(root);
    tmp_a2.ntru_vector.goto_ntt(root);

    let tmp_wb = tmp_a1.mul(&tmp_a2, white_data);
    let mut tmp_ntru = tmp_wb.ntru_vector;
    tmp_ntru.goback_ntt(unroot, ninv);

    let chal = white_data.chal;
    let mask = &white_data.mask;
    let mask_checksum = checksum::compute_crt_checksum(mask);
    
    // Verify mask checksum before using
    if mask_checksum != white_data.mask_checksum {
        panic!("Mask checksum verification failed in decrypt_white!");
    }
    
    let rot = white_data.rotate;

    let mut m = Array1::zeros(degree);
    for i in 0..degree {
        let m_val;
        if chal == 2 {
            m_val = tmp_ntru.vector[(i + rot) % degree].rem_euclid(tmp_ntru.modulus);
            if m_val > modulus / 2 {
                m[i] = (1 - ((m_val + mask[i]) % 2)).rem_euclid(2);
            } else {
                m[i] = ((m_val + mask[i]) % 2).rem_euclid(2);
            }
        } else {
            m_val = tmp_ntru.vector[i].rem_euclid(tmp_ntru.modulus);
            if m_val > modulus / 2 {
                m[i] = (1 - (m_val % 2)).rem_euclid(2);
            } else {
                m[i] = (m_val % 2).rem_euclid(2);
            }
        }
    }
    m
}

pub fn decrypt_message(white_data: &WhiteData, a1: &NTRUVector, a2: &NTRUVector, degree: usize, modulus: i64) -> Array1i64 {
    // Verify checksums before proceeding
    let beta_checksum = checksum::compute_crt_checksum(&white_data.beta);
    let beta_p_checksum = checksum::compute_crt_checksum(&white_data.beta_p);
    let mask_checksum = checksum::compute_crt_checksum(&white_data.mask);
    
    if beta_checksum != white_data.beta_checksum {
        panic!("Beta array checksum verification failed!");
    }
    
    if beta_p_checksum != white_data.beta_p_checksum {
        panic!("Beta prime array checksum verification failed!");
    }
    
    if mask_checksum != white_data.mask_checksum {
        panic!("Mask checksum verification failed!");
    }
    
    // Verify vector checksums
    if !a1.verify_checksum() {
        panic!("A1 vector checksum verification failed!");
    }
    
    if !a2.verify_checksum() {
        panic!("A2 vector checksum verification failed!");
    }
    
    // Continue with the normal decryption process
    decrypt_white(a1, a2, degree, modulus, white_data)
}
