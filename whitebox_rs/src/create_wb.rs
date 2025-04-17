use rand::Rng;
use rand_distr::Normal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::ops::Neg;

#[derive(Clone, Serialize, Deserialize)]
struct NtruVector {
    vector: Vec<i64>,
    degree: usize,
    modulus: i64,
    ntt: bool,
}

impl NtruVector {
    fn new(degree: usize, modulus: i64, ntt: bool) -> Self {
        NtruVector {
            vector: vec![0; degree],
            degree,
            modulus,
            ntt,
        }
    }

    fn add(&self, other: &NtruVector) -> NtruVector {
        let mut res = NtruVector::new(self.degree, self.modulus, self.ntt);
        for i in 0..self.degree {
            res.vector[i] = (self.vector[i] + other.vector[i]).rem_euclid(self.modulus);
        }
        res
    }

    fn mul(&self, other: &NtruVector) -> NtruVector {
        let mut res = NtruVector::new(self.degree, self.modulus, self.ntt);
        if self.ntt {
            for i in 0..self.degree {
                res.vector[i] = (self.vector[i] * other.vector[i]) % self.modulus;
            }
        } else {
            for i in 0..self.degree {
                for j in 0..self.degree {
                    let d = (i + j) % self.degree;
                    let sign = if i + j < self.degree { 1 } else { -1 };
                    res.vector[d] = (res.vector[d] + sign * (self.vector[i] * other.vector[j])).rem_euclid(self.modulus);
                }
            }
        }
        res
    }

    fn goto_ntt(&mut self, root: i64) {
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
    }

    fn goback_ntt(&mut self, unroot: i64, ninv: i64) {
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
    }
}

impl Neg for NtruVector {
    type Output = NtruVector;
    fn neg(self) -> Self::Output {
        let mut res = self.clone();
        for i in 0..res.degree {
            res.vector[i] = (-res.vector[i]).rem_euclid(res.modulus);
        }
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
    base.iter().map(|&b| x.rem_euclid(b)).collect()
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

fn key_gen(degree: usize, q: i64) -> (NtruVector, NtruVector, NtruVector) {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut sk = NtruVector::new(degree, q, false);
    let mut pka = NtruVector::new(degree, q, false);
    let mut pkb = NtruVector::new(degree, q, false);
    for i in 0..degree {
        let sample_sk: f64 = rng.sample(normal);
        sk.vector[i] = sample_sk.round() as i64;
        pka.vector[i] = rng.gen_range(0..q);
        let sample_pkb: f64 = rng.sample(normal);
        pkb.vector[i] = 2 * sample_pkb.round() as i64;
    }
    pkb = -(pkb.add(&pka.mul(&sk)));
    (pka, pkb, sk)
}

fn encrypt(m: &[i64], pka: &NtruVector, pkb: &NtruVector, degree: usize, modulus: i64) -> (NtruVector, NtruVector) {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut u = NtruVector::new(degree, modulus, false);
    let mut e1 = NtruVector::new(degree, modulus, false);
    let mut e2 = NtruVector::new(degree, modulus, false);
    for i in 0..degree {
        let sample_u: f64 = rng.sample(normal);
        u.vector[i] = sample_u.round() as i64;
        let sample_e1: f64 = rng.sample(normal);
        e1.vector[i] = 2 * sample_e1.round() as i64;
        let sample_e2: f64 = rng.sample(normal);
        e2.vector[i] = 2 * sample_e2.round() as i64;
    }
    let mut tmp = NtruVector::new(degree, modulus, false);
    for i in 0..degree {
        tmp.vector[i] = m[i];
    }
    let a1 = pka.mul(&u).add(&e1);
    let a2 = pkb.mul(&u).add(&e2).add(&tmp);
    (a1, a2)
}

fn print_progress(m: usize, n: usize, step: usize) {
    if m % step == 0 {
        println!("{:.1}%", m as f64 * 100.0 / n as f64);
    }
}

fn prepare_first_box_mm3(sk: &NtruVector, a1_r: &NtruVector, a2_r: &NtruVector, a1_ma: &NtruVector, a2_ma: &NtruVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
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

fn prepare_second_box_mm3(sk: &NtruVector, a1_r: &NtruVector, a2_r: &NtruVector, a1_ma: &NtruVector, a2_ma: &NtruVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], beta_p: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
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

fn prepare_first_box_mm2(sk: &NtruVector, a1_o: &NtruVector, a2_o: &NtruVector, a1_z: &NtruVector, a2_z: &NtruVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
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

fn prepare_second_box_mm2(sk: &NtruVector, a1_o: &NtruVector, a2_o: &NtruVector, a1_z: &NtruVector, a2_z: &NtruVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], beta_p: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
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

fn prepare_first_box_mm(sk: &mut NtruVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
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

fn prepare_second_box_mm(sk: &mut NtruVector, root: i64, _unroot: i64, _ninv: i64, beta: &[i64], beta_p: &[i64], k: usize) -> HashMap<String, Vec<Vec<i64>>> {
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

fn write_data(degree: usize, modulus: i64, beta: &[i64], beta_p: &[i64], k: usize, chal: u8) {
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

    let private_data = serde_json::json!({
        "sk": sk.vector,
        "a1_o": a1_o.vector,
        "a2_o": a2_o.vector,
        "a1_z": a1_z.vector,
        "a2_z": a2_z.vector,
    });
    let mut file = File::create("private_data.json").unwrap();
    file.write_all(serde_json::to_string(&private_data).unwrap().as_bytes()).unwrap();

    let pub_enc_data = serde_json::json!({
        "degree": degree,
        "modulus": modulus,
        "pka": pka.vector,
        "pkb": pkb.vector,
    });
    let mut file = File::create("pub_enc_data.json").unwrap();
    file.write_all(serde_json::to_string(&pub_enc_data).unwrap().as_bytes()).unwrap();

    let mut wb_dec_data = serde_json::json!({
        "root": root,
        "unroot": unroot,
        "ninv": ninv,
        "beta": beta,
        "beta_p": beta_p,
        "k": k,
        "mask": mask,
        "rotate": rot,
        "chal": chal,
    });

    match chal {
        0 => {
            println!("prepare_first_box_MM");
            let fb = prepare_first_box_mm(&mut sk.clone(), root, unroot, ninv, beta, k);
            println!("prepare_second_box_MM");
            let sb = prepare_second_box_mm(&mut sk.clone(), root, unroot, ninv, beta, beta_p, k);
            let obj = wb_dec_data.as_object_mut().unwrap();
            for (key, value) in fb {
                obj.insert(key, serde_json::to_value(value).unwrap());
            }
            for (key, value) in sb {
                obj.insert(key, serde_json::to_value(value).unwrap());
            }
        }
        1 => {
            println!("prepare_first_box_MM2");
            let fb = prepare_first_box_mm2(&sk, &a1_o, &a2_o, &a1_z, &a2_z, root, unroot, ninv, beta, k);
            println!("prepare_second_box_MM2");
            let sb = prepare_second_box_mm2(&sk, &a1_o, &a2_o, &a1_z, &a2_z, root, unroot, ninv, beta, beta_p, k);
            let obj = wb_dec_data.as_object_mut().unwrap();
            for (key, value) in fb {
                obj.insert(key, serde_json::to_value(value).unwrap());
            }
            for (key, value) in sb {
                obj.insert(key, serde_json::to_value(value).unwrap());
            }
        }
        2 => {
            println!("prepare_first_box_MM3");
            let fb = prepare_first_box_mm3(&sk, &a1_rot, &a2_rot, &a1_ma, &a2_ma, root, unroot, ninv, beta, k);
            println!("prepare_second_box_MM3");
            let sb = prepare_second_box_mm3(&sk, &a1_rot, &a2_rot, &a1_ma, &a2_ma, root, unroot, ninv, beta, beta_p, k);
            let obj = wb_dec_data.as_object_mut().unwrap();
            for (key, value) in fb {
                obj.insert(key, serde_json::to_value(value).unwrap());
            }
            for (key, value) in sb {
                obj.insert(key, serde_json::to_value(value).unwrap());
            }
        }
        _ => panic!("Invalid challenge level"),
    }

    let mut file = File::create("wb_dec_data.json").unwrap();
    file.write_all(serde_json::to_string(&wb_dec_data).unwrap().as_bytes()).unwrap();
}

pub fn create_wb() {
    let degree = 512;
    let modulus = 1231873;
    // beta = 3094416
    let beta = vec![13, 16, 19, 27, 29];
    // beta_p = 3333275
    let beta_p = vec![11, 17, 23, 25, 31];
    let k = 5;
    let chal = 2;
    write_data(degree, modulus, &beta, &beta_p, k, chal);
}
