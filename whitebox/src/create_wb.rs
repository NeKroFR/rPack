use crate::lattice::{NTRUVector, PubEncData, WhiteData};
use rand::Rng;
use rand_distr::Normal;
use std::collections::HashMap;
use std::ops::Neg;

impl NTRUVector {
    pub fn goto_ntt(&mut self, root: i64) {
        if self.ntt {
            println!("This vector is already ntt");
            return;
        }
        let n = self.degree;
        self.ntt = true;
        let levels = (n as f64).log2() as usize;
        let mut powtable = Vec::new();
        let mut temp = 1;
        for i in 0..n {
            self.vector[i] = (self.vector[i] * temp).rem_euclid(self.modulus);
            if i % 2 == 0 {
                powtable.push(temp);
            }
            temp = (temp * root).rem_euclid(self.modulus);
        }

        for i in 0..n {
            let j = bit_reverse(i, levels);
            if j > i {
                self.vector.swap(i, j);
            }
        }

        let mut size = 2;
        while size <= n {
            let halfsize = size / 2;
            let tablestep = n / size;
            for i in (0..n).step_by(size) {
                let mut k = 0;
                for j in i..i + halfsize {
                    let l = j + halfsize;
                    let left = self.vector[j];
                    let right = (self.vector[l] * powtable[k]).rem_euclid(self.modulus);
                    self.vector[j] = (left + right).rem_euclid(self.modulus);
                    self.vector[l] = (left - right).rem_euclid(self.modulus);
                    k += tablestep;
                }
            }
            size *= 2;
        }
        self.update_checksum();
    }

    pub fn goback_ntt(&mut self, unroot: i64, ninv: i64) {
        if !self.ntt {
            println!("This vector is not ntt");
            return;
        }
        self.ntt = false;
        let n = self.degree;
        let levels = (n as f64).log2() as usize;
        let mut powtable = Vec::new();
        let mut powtable2 = Vec::new();
        let mut temp = 1;
        for i in 0..n {
            if i % 2 == 0 {
                powtable.push(temp);
            }
            powtable2.push(temp);
            temp = (temp * unroot).rem_euclid(self.modulus);
        }

        for i in 0..n {
            let j = bit_reverse(i, levels);
            if j > i {
                self.vector.swap(i, j);
            }
        }

        let mut size = 2;
        while size <= n {
            let halfsize = size / 2;
            let tablestep = n / size;
            for i in (0..n).step_by(size) {
                let mut k = 0;
                for j in i..i + halfsize {
                    let l = j + halfsize;
                    let left = self.vector[j];
                    let right = (self.vector[l] * powtable[k]).rem_euclid(self.modulus);
                    self.vector[j] = (left + right).rem_euclid(self.modulus);
                    self.vector[l] = (left - right).rem_euclid(self.modulus);
                    k += tablestep;
                }
            }
            size *= 2;
        }

        for i in 0..n {
            self.vector[i] = (self.vector[i] * ninv * powtable2[i]).rem_euclid(self.modulus);
        }
        self.update_checksum();
    }
}

impl Neg for NTRUVector {
    type Output = NTRUVector;
    fn neg(self) -> Self::Output {
        let mut res = self.clone();
        for i in 0..res.degree {
            res.vector[i] = (-res.vector[i]).rem_euclid(res.modulus);
        }
        res.update_checksum();
        res
    }
}

fn bit_reverse(x: usize, bits: usize) -> usize {
    let mut y = 0;
    let mut x = x;
    for _ in 0..bits {
        y = (y << 1) | (x & 1);
        x >>= 1;
    }
    y
}

fn goto_crt(x: i64, base: &[i64]) -> Vec<i64> {
    let crt_values = base.iter().map(|&b| x.rem_euclid(b)).collect();    
    crt_values
}

fn xgcd(b: i64, n: i64) -> (i64, i64, i64) {
    let mut x0 = 1;
    let mut x1 = 0;
    let mut y0 = 0;
    let mut y1 = 1;
    let mut b = b;
    let mut n = n;
    while n != 0 {
        let q = b / n;
        let temp = n;
        n = b % n;
        b = temp;
        let temp = x1;
        x1 = x0 - q * x1;
        x0 = temp;
        let temp = y1;
        y1 = y0 - q * y1;
        y0 = temp;
    }
    (b, x0, y0)
}

fn key_gen(degree: usize, q: i64) -> (NTRUVector, NTRUVector, NTRUVector) {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut sk = NTRUVector::new(degree, q, false);
    let mut pka = NTRUVector::new(degree, q, false);
    let mut pkb = NTRUVector::new(degree, q, false);
    for i in 0..degree {
        let sample_sk: f64 = rng.sample(normal);
        sk.vector[i] = sample_sk.round() as i64;
        pka.vector[i] = rng.gen_range(0..q);
        let sample_pkb: f64 = rng.sample(normal);
        pkb.vector[i] = 2 * sample_pkb.round() as i64;
    }
    
    sk.update_checksum();
    pka.update_checksum();
    pkb.update_checksum();
    
    pkb = -(pkb.add(&pka.mul(&sk)));
    (pka, pkb, sk)
}

fn encrypt(m: &[i64], pka: &NTRUVector, pkb: &NTRUVector, degree: usize, modulus: i64) -> (NTRUVector, NTRUVector) {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut u = NTRUVector::new(degree, modulus, false);
    let mut e1 = NTRUVector::new(degree, modulus, false);
    let mut e2 = NTRUVector::new(degree, modulus, false);
    for i in 0..degree {
        let sample_u: f64 = rng.sample(normal);
        u.vector[i] = sample_u.round() as i64;
        let sample_e1: f64 = rng.sample(normal);
        e1.vector[i] = 2 * sample_e1.round() as i64;
        let sample_e2: f64 = rng.sample(normal);
        e2.vector[i] = 2 * sample_e2.round() as i64;
    }
    
    u.update_checksum();
    e1.update_checksum();
    e2.update_checksum();
    
    let mut tmp = NTRUVector::new(degree, modulus, false);
    for i in 0..degree {
        tmp.vector[i] = m[i];
    }
    tmp.update_checksum();
    
    let a1 = pka.mul(&u).add(&e1);
    let a2 = pkb.mul(&u).add(&e2).add(&tmp);
    (a1, a2)
}

fn print_progress(m: usize, n: usize, step: usize) {
    if m % step == 0 {
        println!("{:.1}%", m as f64 * 100.0 / n as f64);
    }
}

fn prepare_first_box_mm3(sk: &NTRUVector, a1_r: &NTRUVector, a2_r: &NTRUVector, a1_ma: &NTRUVector, a2_ma: &NTRUVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
    let mut rot = a2_r.add(&a1_r.mul(sk));
    let mut mask = a2_ma.add(&a1_ma.mul(sk)).neg();
    let mut tmp_sk = sk.mul(&rot);
    let mut tmp_sz = sk.mul(&mask);
    rot.goto_ntt(root);
    mask.goto_ntt(root);
    tmp_sk.goto_ntt(root);
    tmp_sz.goto_ntt(root);
    let m: i64 = beta.iter().product();
    let n = tmp_sk.modulus;
    let (_, n_inv, _) = xgcd(n, m);
    let n_inv_m = goto_crt(n_inv, beta);
    let mut fb = HashMap::new();
    for dim in 0..tmp_sk.degree {
        print_progress(dim, tmp_sk.degree, 64);
        let key = format!("fb_dim_{}", dim);
        let mut table = vec![vec![0; 32]; 32];
        let s = goto_crt(tmp_sk.vector[dim], beta);
        let _sz = goto_crt(tmp_sz.vector[dim], beta);
        let r = goto_crt(rot.vector[dim], beta);
        let mask_crt = goto_crt(mask.vector[dim], beta);
        for j in 0..32 {
            let a = goto_crt(j as i64, beta);
            for l in 0..32 {
                let b = goto_crt(l as i64, beta);
                let mut val = 0;
                for i in 0..k {
                    let part = (a[i] * s[i] + b[i] * r[i] + r[i] * mask_crt[i]).rem_euclid(beta[i]);
                    let neg_n_inv_m = (-n_inv_m[i]).rem_euclid(beta[i]);
                    val += (part * neg_n_inv_m).rem_euclid(beta[i]) << (5 * i);
                }
                table[j][l] = val;
            }
        }
        fb.insert(key, table);
    }
    print_progress(tmp_sk.degree, tmp_sk.degree, 64);
    fb
}

fn prepare_second_box_mm3(sk: &NTRUVector, a1_r: &NTRUVector, a2_r: &NTRUVector, a1_ma: &NTRUVector, a2_ma: &NTRUVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], beta_p: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
    let mut rot = a2_r.add(&a1_r.mul(sk));
    let mut mask = a2_ma.add(&a1_ma.mul(sk)).neg();
    let mut tmp_sk = sk.mul(&rot);
    let mut tmp_sz = sk.mul(&mask);
    rot.goto_ntt(root);
    mask.goto_ntt(root);
    tmp_sk.goto_ntt(root);
    tmp_sz.goto_ntt(root);
    let m: i64 = beta.iter().product();
    let m_p: i64 = beta_p.iter().product();
    let (_, m_inv, _) = xgcd(m, m_p);
    let m_inv_m_p = goto_crt(m_inv, beta_p);
    let mut sb = HashMap::new();
    for dim in 0..tmp_sk.degree {
        print_progress(dim, tmp_sk.degree, 64);
        let key = format!("sb_dim_{}", dim);
        let mut table = vec![vec![0; 32]; 32];
        let s = goto_crt(tmp_sk.vector[dim], beta_p);
        let _sz = goto_crt(tmp_sz.vector[dim], beta_p);
        let r = goto_crt(rot.vector[dim], beta_p);
        let mask_crt = goto_crt(mask.vector[dim], beta_p);
        for j in 0..32 {
            let a = goto_crt(j as i64, beta_p);
            for l in 0..32 {
                let b = goto_crt(l as i64, beta_p);
                let mut val = 0;
                for i in 0..k {
                    let part = (a[i] * s[i] + b[i] * r[i] + r[i] * mask_crt[i]).rem_euclid(beta_p[i]);
                    val += (part * m_inv_m_p[i]).rem_euclid(beta_p[i]) << (5 * i);
                }
                table[j][l] = val;
            }
        }
        sb.insert(key, table);
    }
    print_progress(tmp_sk.degree, tmp_sk.degree, 64);
    sb
}

fn prepare_first_box_mm2(sk: &NTRUVector, a1_o: &NTRUVector, a2_o: &NTRUVector, a1_z: &NTRUVector, a2_z: &NTRUVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
    let mut one = a2_o.add(&a1_o.mul(sk));
    let mut zero = a2_z.add(&a1_z.mul(sk)).neg();
    let mut tmp_sk = sk.mul(&one);
    let mut tmp_sz = sk.mul(&zero);
    one.goto_ntt(root);
    zero.goto_ntt(root);
    tmp_sk.goto_ntt(root);
    tmp_sz.goto_ntt(root);
    let m: i64 = beta.iter().product();
    let n = tmp_sk.modulus;
    let (_, n_inv, _) = xgcd(n, m);
    let n_inv_m = goto_crt(n_inv, beta);
    let mut fb = HashMap::new();
    for dim in 0..tmp_sk.degree {
        print_progress(dim, tmp_sk.degree, 64);
        let key = format!("fb_dim_{}", dim);
        let mut table = vec![vec![0; 32]; 32];
        let s = goto_crt(tmp_sk.vector[dim], beta);
        let _sz = goto_crt(tmp_sz.vector[dim], beta);
        let o = goto_crt(one.vector[dim], beta);
        let z = goto_crt(zero.vector[dim], beta);
        for j in 0..32 {
            let a = goto_crt(j as i64, beta);
            for l in 0..32 {
                let b = goto_crt(l as i64, beta);
                let mut val = 0;
                for i in 0..k {
                    let part = (a[i] * s[i] + b[i] * o[i] + a[i] * _sz[i] + b[i] * z[i]).rem_euclid(beta[i]);
                    let neg_n_inv_m = (-n_inv_m[i]).rem_euclid(beta[i]);
                    val += (part * neg_n_inv_m).rem_euclid(beta[i]) << (5 * i);
                }
                table[j][l] = val;
            }
        }
        fb.insert(key, table);
    }
    print_progress(tmp_sk.degree, tmp_sk.degree, 64);
    fb
}

fn prepare_second_box_mm2(sk: &NTRUVector, a1_o: &NTRUVector, a2_o: &NTRUVector, a1_z: &NTRUVector, a2_z: &NTRUVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], beta_p: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
    let mut one = a2_o.add(&a1_o.mul(sk));
    let mut zero = a2_z.add(&a1_z.mul(sk)).neg();
    let mut tmp_sk = sk.mul(&one);
    let mut tmp_sz = sk.mul(&zero);
    one.goto_ntt(root);
    zero.goto_ntt(root);
    tmp_sk.goto_ntt(root);
    tmp_sz.goto_ntt(root);
    let m: i64 = beta.iter().product();
    let m_p: i64 = beta_p.iter().product();
    let (_, m_inv, _) = xgcd(m, m_p);
    let m_inv_m_p = goto_crt(m_inv, beta_p);
    let mut sb = HashMap::new();
    for dim in 0..tmp_sk.degree {
        print_progress(dim, tmp_sk.degree, 64);
        let key = format!("sb_dim_{}", dim);
        let mut table = vec![vec![0; 32]; 32];
        let s = goto_crt(tmp_sk.vector[dim], beta_p);
        let _sz = goto_crt(tmp_sz.vector[dim], beta_p);
        let o = goto_crt(one.vector[dim], beta_p);
        let z = goto_crt(zero.vector[dim], beta_p);
        for j in 0..32 {
            let a = goto_crt(j as i64, beta_p);
            for l in 0..32 {
                let b = goto_crt(l as i64, beta_p);
                let mut val = 0;
                for i in 0..k {
                    let part = (a[i] * s[i] + b[i] * o[i] + a[i] * _sz[i] + b[i] * z[i]).rem_euclid(beta_p[i]);
                    val += (part * m_inv_m_p[i]).rem_euclid(beta_p[i]) << (5 * i);
                }
                table[j][l] = val;
            }
        }
        sb.insert(key, table);
    }
    print_progress(tmp_sk.degree, tmp_sk.degree, 64);
    sb
}

fn prepare_first_box_mm(sk: &mut NTRUVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
    sk.goto_ntt(root);
    let m: i64 = beta.iter().product();
    let n = sk.modulus;
    let (_, n_inv, _) = xgcd(n, m);
    let n_inv_m = goto_crt(n_inv, beta);
    let mut fb = HashMap::new();
    for dim in 0..sk.degree {
        print_progress(dim, sk.degree, 64);
        let key = format!("fb_dim_{}", dim);
        let mut table = vec![vec![0; 32]; 32];
        let s = goto_crt(sk.vector[dim], beta);
        for j in 0..32 {
            let a = goto_crt(j as i64, beta);
            for l in 0..32 {
                let b = goto_crt(l as i64, beta);
                let mut val = 0;
                for i in 0..k {
                    let part = (a[i] * s[i] + b[i]).rem_euclid(beta[i]);
                    let neg_n_inv_m = (-n_inv_m[i]).rem_euclid(beta[i]);
                    val += (part * neg_n_inv_m).rem_euclid(beta[i]) << (5 * i);
                }
                table[j][l] = val;
            }
        }
        fb.insert(key, table);
    }
    print_progress(sk.degree, sk.degree, 64);
    sk.goback_ntt(_unroot, _ninv);
    fb
}

fn prepare_second_box_mm(sk: &mut NTRUVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], beta_p: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
    sk.goto_ntt(root);
    let m: i64 = beta.iter().product();
    let m_p: i64 = beta_p.iter().product();
    let (_, m_inv, _) = xgcd(m, m_p);
    let m_inv_m_p = goto_crt(m_inv, beta_p);
    let mut sb = HashMap::new();
    for dim in 0..sk.degree {
        print_progress(dim, sk.degree, 64);
        let key = format!("sb_dim_{}", dim);
        let mut table = vec![vec![0; 32]; 32];
        let s = goto_crt(sk.vector[dim], beta_p);
        for j in 0..32 {
            let a = goto_crt(j as i64, beta_p);
            for l in 0..32 {
                let b = goto_crt(l as i64, beta_p);
                let mut val = 0;
                for i in 0..k {
                    let part = (a[i] * s[i] + b[i]).rem_euclid(beta_p[i]);
                    val += (part * m_inv_m_p[i]).rem_euclid(beta_p[i]) << (5 * i);
                }
                table[j][l] = val;
            }
        }
        sb.insert(key, table);
    }
    print_progress(sk.degree, sk.degree, 64);
    sk.goback_ntt(_unroot, _ninv);
    sb
}

fn pow(base: i64, exp: i64, modulus: i64) -> i64 {
    let mut result = 1;
    let mut base = base % modulus;
    let mut exp = exp;
    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % modulus;
        }
        base = (base * base) % modulus;
        exp /= 2;
    }
    result
}

fn unique_prime_factors(mut n: i64) -> Vec<i64> {
    let mut result = Vec::new();
    let mut i = 2;
    while i * i <= n {
        if n % i == 0 {
            result.push(i);
            while n % i == 0 {
                n /= i;
            }
        }
        i += 1;
    }
    if n > 1 {
        result.push(n);
    }
    result
}

fn is_generator(val: i64, totient: i64, modulus: i64) -> bool {
    let pf = unique_prime_factors(totient);
    pow(val, totient, modulus) == 1 && pf.iter().all(|&p| pow(val, totient / p, modulus) != 1)
}

fn find_generator(totient: i64, modulus: i64) -> i64 {
    for i in 1..modulus {
        if is_generator(i, totient, modulus) {
            return i;
        }
    }
    panic!("No generator exists");
}

fn find_primitive_root(degree: usize, totient: i64, modulus: i64) -> i64 {
    let gen = find_generator(totient, modulus);
    pow(gen, totient / degree as i64, modulus)
}

pub fn generate_whitebox_data(degree: usize, modulus: i64, beta: &[i64], beta_p: &[i64], k: usize, chal: u8) -> (PubEncData, WhiteData) {
    let (pka, pkb, sk) = key_gen(degree, modulus);
    let mut rng = rand::thread_rng();

    let mut one = vec![0; degree];
    one[0] = 1;
    let (a1_o, a2_o) = encrypt(&one, &pka, &pkb, degree, modulus);
    let zero = vec![0; degree];
    let (a1_z, a2_z) = encrypt(&zero, &pka, &pkb, degree, modulus);

    let mut rotate = vec![0; degree];
    let rot = rng.gen_range(0..degree);
    rotate[rot] = 1;
    let (a1_rot, a2_rot) = encrypt(&rotate, &pka, &pkb, degree, modulus);

    let mut mask = vec![0; degree];
    for i in 0..degree {
        mask[i] = rng.gen_range(0..2);
    }
    let (a1_ma, a2_ma) = encrypt(&mask, &pka, &pkb, degree, modulus);

    let root = find_primitive_root(2 * degree, modulus - 1, modulus);
    let unroot = xgcd(root, modulus).1;
    let ninv = xgcd(degree as i64, modulus).1;

    let (fb, sb) = match chal {
        0 => {
            println!("prepare_first_box_MM");
            let fb = prepare_first_box_mm(&mut sk.clone(), root, unroot, ninv, beta, k);
            println!("prepare_second_box_MM");
            let sb = prepare_second_box_mm(&mut sk.clone(), root, unroot, ninv, beta, beta_p, k);
            (fb, sb)
        }
        1 => {
            println!("prepare_first_box_MM2");
            let fb = prepare_first_box_mm2(&sk, &a1_o, &a2_o, &a1_z, &a2_z, root, unroot, ninv, beta, k);
            println!("prepare_second_box_MM2");
            let sb = prepare_second_box_mm2(&sk, &a1_o, &a2_o, &a1_z, &a2_z, root, unroot, ninv, beta, beta_p, k);
            (fb, sb)
        }
        2 => {
            println!("prepare_first_box_MM3");
            let fb = prepare_first_box_mm3(&sk, &a1_rot, &a2_rot, &a1_ma, &a2_ma, root, unroot, ninv, beta, k);
            println!("prepare_second_box_MM3");
            let sb = prepare_second_box_mm3(&sk, &a1_rot, &a2_rot, &a1_ma, &a2_ma, root, unroot, ninv, beta, beta_p, k);
            (fb, sb)
        }
        _ => panic!("Invalid challenge level"),
    };

    let beta_checksum = checksum::compute_crt_checksum(beta);
    let beta_p_checksum = checksum::compute_crt_checksum(beta_p);
    let mask_checksum = checksum::compute_crt_checksum(&mask);
    
    let mut all_data = Vec::new();
    all_data.extend_from_slice(&[root, unroot, ninv]);
    all_data.extend_from_slice(&[k as i64, rot as i64, chal as i64]);
    let data_checksum = checksum::compute_crt_checksum(&all_data);
    
    let pub_data = format!("{}:{}", degree, modulus).into_bytes();
    let pub_data_checksum = checksum::compute_blake3(&pub_data);
    
    let pub_enc_data = PubEncData {
        degree,
        modulus,
        pka: pka.clone(),
        pkb: pkb.clone(),
        data_checksum: pub_data_checksum,
    };

    let white_data = WhiteData {
        root,
        unroot,
        ninv,
        beta: beta.to_vec(),
        beta_p: beta_p.to_vec(),
        k,
        mask,
        rotate: rot,
        chal,
        fb,
        sb,
        beta_checksum,
        beta_p_checksum,
        mask_checksum,
        data_checksum,
    };

    (pub_enc_data, white_data)
}

pub fn create_whitebox() -> (PubEncData, WhiteData) {
    let degree = 512;
    let modulus = 1231873;
    let beta = vec![13, 16, 19, 27, 29];
    let beta_p = vec![11, 17, 23, 25, 31];
    let k = 5;
    let chal = 2;
    let (mut pub_enc_data, mut white_data) = generate_whitebox_data(degree, modulus, &beta, &beta_p, k, chal);
    
    let beta_checksum = checksum::compute_crt_checksum(&beta);
    let beta_p_checksum = checksum::compute_crt_checksum(&beta_p);
    let mask_checksum = checksum::compute_crt_checksum(&white_data.mask);
    
    let mut all_data = Vec::new();
    all_data.extend_from_slice(&[white_data.root, white_data.unroot, white_data.ninv]);
    all_data.extend_from_slice(&[white_data.k as i64, white_data.rotate as i64, white_data.chal as i64]);
    let data_checksum = checksum::compute_crt_checksum(&all_data);
    
    let pub_data = format!("{}:{}",
        pub_enc_data.degree,
        pub_enc_data.modulus
    ).into_bytes();
    
    let pub_data_checksum = checksum::compute_blake3(&pub_data);
    
    white_data.beta_checksum = beta_checksum;
    white_data.beta_p_checksum = beta_p_checksum;
    white_data.mask_checksum = mask_checksum;
    white_data.data_checksum = data_checksum;
    
    pub_enc_data.data_checksum = pub_data_checksum;
    
    (pub_enc_data, white_data)
}
