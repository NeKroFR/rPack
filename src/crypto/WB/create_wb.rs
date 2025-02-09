use crate::crypto::WB::lattice::{LatticeVector, Modulus, key_gen, encrypt, goto_crt, xgcd};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use rand::Rng;

// ---------- Helper Functions ----------

/// Print progress if m is a multiple of step.
fn print_progress(m: usize, n: usize, step: usize) {
    if m % step == 0 {
        println!("{:.1}%", m as f64 * 100.0 / n as f64);
    }
}

/// A variant of CRT that returns only the first `k` remainders.
fn goto_crt_k(x: i64, base: &[i64], k: usize) -> Vec<i64> {
    base.iter().take(k).map(|&b| x.rem_euclid(b)).collect()
}

/// Compute the product of all elements in a slice.
fn product(slice: &[i64]) -> i64 {
    slice.iter().product()
}

/// Modular exponentiation.
fn mod_pow(mut base: i64, mut exp: i64, modulus: i64) -> i64 {
    let mut result = 1;
    base = base.rem_euclid(modulus);
    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base).rem_euclid(modulus);
        }
        base = (base * base).rem_euclid(modulus);
        exp /= 2;
    }
    result
}

/// Returns an arbitrary generator of the multiplicative group modulo `m` given totient.
fn find_generator(totient: i64, m: i64) -> Option<i64> {
    for i in 1..m {
        if is_generator(i, totient, m) {
            return Some(i);
        }
    }
    None
}

/// Returns true if `val` is a generator modulo `m`.
fn is_generator(val: i64, totient: i64, m: i64) -> bool {
    if mod_pow(val, totient, m) != 1 {
        return false;
    }
    let factors = unique_prime_factors(totient);
    for &p in &factors {
        if mod_pow(val, totient / p, m) == 1 {
            return false;
        }
    }
    true
}

/// Returns an arbitrary primitive degree-th root of unity modulo `m`.
/// `totient` should be a multiple of `degree`.
fn find_primitive_root(degree: i64, totient: i64, m: i64) -> Option<i64> {
    if totient % degree != 0 {
        return None;
    }
    if let Some(gen) = find_generator(totient, m) {
        let exp = totient / degree;
        Some(mod_pow(gen, exp, m))
    } else {
        None
    }
}

/// Returns a sorted vector of unique prime factors of n.
fn unique_prime_factors(mut n: i64) -> Vec<i64> {
    let mut factors = Vec::new();
    let mut i = 2;
    while i * i <= n {
        if n % i == 0 {
            factors.push(i);
            while n % i == 0 {
                n /= i;
            }
        }
        i += 1;
    }
    if n > 1 {
        factors.push(n);
    }
    factors.sort();
    factors
}

// ---------- Prepare Lookup Table Functions ----------

/// prepare_first_box_MM3 (with “rotate/mask” encoding)
pub fn prepare_first_box_MM3(
    sk: &LatticeVector,
    a1_r: &LatticeVector,
    a2_r: &LatticeVector,
    a1_ma: &LatticeVector,
    a2_ma: &LatticeVector,
    root: i64,
    _unroot: i64,
    _ninv: i64,
    beta: &[i64],
    k: usize,
) -> HashMap<String, Vec<Vec<i64>>> {
    // Compute intermediate values.
    let rot = a2_r.clone() + a1_r.clone() * sk.clone();
    let mask = -(a2_ma.clone() + a1_ma.clone() * sk.clone());
    let tmp_sk = sk.clone() * rot.clone();
    let tmp_sz = sk.clone() * mask.clone();

    // Convert all to NTT domain.
    let mut rot_ntt = rot.clone();
    rot_ntt.goto_ntt(root);
    let mut mask_ntt = mask.clone();
    mask_ntt.goto_ntt(root);
    let mut tmp_sk_ntt = tmp_sk.clone();
    tmp_sk_ntt.goto_ntt(root);
    let mut tmp_sz_ntt = tmp_sz.clone();
    tmp_sz_ntt.goto_ntt(root);

    // Montgomery parameters.
    let M = product(beta);
    let N = tmp_sk_ntt.modulus.value;
    let egcd = xgcd(N, M);
    let Ninv = egcd.x; // modular inverse of N modulo M
    let Ninv_M = goto_crt_k(Ninv, beta, k);

    let mut fb = HashMap::new();
    for dim in 0..tmp_sk_ntt.degree {
        print_progress(dim, tmp_sk_ntt.degree, 64);
        let key = format!("fb_dim_{}", dim);
        let mut matrix = vec![vec![0i64; 32]; 32];
        let s = goto_crt_k(tmp_sk_ntt.vector[dim], beta, k);
        // We do not use sz further here but keep for symmetry.
        let _sz = goto_crt_k(tmp_sz_ntt.vector[dim], beta, k);
        let r = goto_crt_k(rot_ntt.vector[dim], beta, k);
        let m = goto_crt_k(mask_ntt.vector[dim], beta, k);
        for j in 0..32 {
            let a = goto_crt_k(j as i64, beta, k);
            for l in 0..32 {
                let b = goto_crt_k(l as i64, beta, k);
                let mut value = 0;
                for i in 0..k {
                    let term = (a[i] * s[i] + b[i] * r[i] + r[i] * m[i])
                        .rem_euclid(beta[i]);
                    let term = (-Ninv_M[i] * term).rem_euclid(beta[i]);
                    value += term << (5 * i);
                }
                matrix[j][l] = value;
            }
        }
        fb.insert(key, matrix);
    }
    print_progress(tmp_sk_ntt.degree, tmp_sk_ntt.degree, 64);
    fb
}

/// prepare_second_box_MM3 (with “rotate/mask” encoding, using beta_p)
pub fn prepare_second_box_MM3(
    sk: &LatticeVector,
    a1_r: &LatticeVector,
    a2_r: &LatticeVector,
    a1_ma: &LatticeVector,
    a2_ma: &LatticeVector,
    root: i64,
    _unroot: i64,
    _ninv: i64,
    beta: &[i64],
    beta_p: &[i64],
    k: usize,
) -> HashMap<String, Vec<Vec<i64>>> {
    let rot = a2_r.clone() + a1_r.clone() * sk.clone();
    let mask = -(a2_ma.clone() + a1_ma.clone() * sk.clone());
    let tmp_sk = sk.clone() * rot.clone();
    let tmp_sz = sk.clone() * mask.clone();

    let mut rot_ntt = rot.clone();
    rot_ntt.goto_ntt(root);
    let mut mask_ntt = mask.clone();
    mask_ntt.goto_ntt(root);
    let mut tmp_sk_ntt = tmp_sk.clone();
    tmp_sk_ntt.goto_ntt(root);
    let mut tmp_sz_ntt = tmp_sz.clone();
    tmp_sz_ntt.goto_ntt(root);

    let M = product(beta);
    let M_p = product(beta_p);
    let egcd = xgcd(M, M_p);
    let Minv = egcd.x;
    let Minv_M_p = goto_crt_k(Minv, beta_p, k);

    let mut sb = HashMap::new();
    for dim in 0..tmp_sk_ntt.degree {
        print_progress(dim, tmp_sk_ntt.degree, 64);
        let key = format!("sb_dim_{}", dim);
        let mut matrix = vec![vec![0i64; 32]; 32];
        let s = goto_crt_k(tmp_sk_ntt.vector[dim], beta_p, k);
        let sz = goto_crt_k(tmp_sz_ntt.vector[dim], beta_p, k); // use tmp_sz_ntt here
        let o = goto_crt_k(rot_ntt.vector[dim], beta_p, k);
        let z = goto_crt_k(mask_ntt.vector[dim], beta_p, k);
        for j in 0..32 {
            let a = goto_crt_k(j as i64, beta_p, k);
            for l in 0..32 {
                let b = goto_crt_k(l as i64, beta_p, k);
                let mut value = 0;
                for i in 0..k {
                    // Note: replaced "sb" with "sz" here.
                    let term = (a[i] * s[i] + b[i] * o[i] + a[i] * sz[i] + b[i] * z[i])
                        .rem_euclid(beta_p[i]);
                    let term = (term * Minv_M_p[i]).rem_euclid(beta_p[i]);
                    value += term << (5 * i);
                }
                matrix[j][l] = value;
            }
        }
        sb.insert(key, matrix);
    }
    print_progress(tmp_sk_ntt.degree, tmp_sk_ntt.degree, 64);
    sb
}

/// prepare_first_box_MM2 (with “one/zero” encoding)
pub fn prepare_first_box_MM2(
    sk: &LatticeVector,
    a1_o: &LatticeVector,
    a2_o: &LatticeVector,
    a1_z: &LatticeVector,
    a2_z: &LatticeVector,
    root: i64,
    _unroot: i64,
    _ninv: i64,
    beta: &[i64],
    k: usize,
) -> HashMap<String, Vec<Vec<i64>>> {
    let one = a2_o.clone() + a1_o.clone() * sk.clone();
    let zero = -(a2_z.clone() + a1_z.clone() * sk.clone());
    let tmp_sk = sk.clone() * one.clone();
    let tmp_sz = sk.clone() * zero.clone();

    let mut one_ntt = one.clone();
    one_ntt.goto_ntt(root);
    let mut zero_ntt = zero.clone();
    zero_ntt.goto_ntt(root);
    let mut tmp_sk_ntt = tmp_sk.clone();
    tmp_sk_ntt.goto_ntt(root);
    let mut tmp_sz_ntt = tmp_sz.clone();
    tmp_sz_ntt.goto_ntt(root);

    let M = product(beta);
    let N = tmp_sk_ntt.modulus.value;
    let egcd = xgcd(N, M);
    let Ninv = egcd.x;
    let Ninv_M = goto_crt_k(Ninv, beta, k);

    let mut fb = HashMap::new();
    for dim in 0..tmp_sk_ntt.degree {
        print_progress(dim, tmp_sk_ntt.degree, 64);
        let key = format!("fb_dim_{}", dim);
        let mut matrix = vec![vec![0i64; 32]; 32];
        let s = goto_crt_k(tmp_sk_ntt.vector[dim], beta, k);
        let sz = goto_crt_k(tmp_sz_ntt.vector[dim], beta, k);
        let o = goto_crt_k(one_ntt.vector[dim], beta, k);
        let z = goto_crt_k(zero_ntt.vector[dim], beta, k);
        for j in 0..32 {
            let a = goto_crt_k(j as i64, beta, k);
            for l in 0..32 {
                let b = goto_crt_k(l as i64, beta, k);
                let mut value = 0;
                for i in 0..k {
                    let term = (a[i] * s[i] + b[i] * o[i] + a[i] * sz[i] + b[i] * z[i])
                        .rem_euclid(beta[i]);
                    let term = (-Ninv_M[i] * term).rem_euclid(beta[i]);
                    value += term << (5 * i);
                }
                matrix[j][l] = value;
            }
        }
        fb.insert(key, matrix);
    }
    print_progress(tmp_sk_ntt.degree, tmp_sk_ntt.degree, 64);
    fb
}

/// prepare_second_box_MM2 (with “one/zero” encoding, using beta_p)
pub fn prepare_second_box_MM2(
    sk: &LatticeVector,
    a1_o: &LatticeVector,
    a2_o: &LatticeVector,
    a1_z: &LatticeVector,
    a2_z: &LatticeVector,
    root: i64,
    _unroot: i64,
    _ninv: i64,
    beta: &[i64],
    beta_p: &[i64],
    k: usize,
) -> HashMap<String, Vec<Vec<i64>>> {
    let one = a2_o.clone() + a1_o.clone() * sk.clone();
    let zero = -(a2_z.clone() + a1_z.clone() * sk.clone());
    let tmp_sk = sk.clone() * one.clone();
    let tmp_sz = sk.clone() * zero.clone();

    let mut one_ntt = one.clone();
    one_ntt.goto_ntt(root);
    let mut zero_ntt = zero.clone();
    zero_ntt.goto_ntt(root);
    let mut tmp_sk_ntt = tmp_sk.clone();
    tmp_sk_ntt.goto_ntt(root);
    let mut tmp_sz_ntt = tmp_sz.clone();
    tmp_sz_ntt.goto_ntt(root);

    let M = product(beta);
    let M_p = product(beta_p);
    let egcd = xgcd(M, M_p);
    let Minv = egcd.x;
    let Minv_M_p = goto_crt_k(Minv, beta_p, k);

    let mut sb = HashMap::new();
    for dim in 0..tmp_sk_ntt.degree {
        print_progress(dim, tmp_sk_ntt.degree, 64);
        let key = format!("sb_dim_{}", dim);
        let mut matrix = vec![vec![0i64; 32]; 32];
        let s = goto_crt_k(tmp_sk_ntt.vector[dim], beta_p, k);
        let sz = goto_crt_k(tmp_sz_ntt.vector[dim], beta_p, k);
        let o = goto_crt_k(one_ntt.vector[dim], beta_p, k);
        let z = goto_crt_k(zero_ntt.vector[dim], beta_p, k);
        for j in 0..32 {
            let a = goto_crt_k(j as i64, beta_p, k);
            for l in 0..32 {
                let b = goto_crt_k(l as i64, beta_p, k);
                let mut value = 0;
                for i in 0..k {
                    let term = (a[i] * s[i] + b[i] * o[i] + a[i] * sz[i] + b[i] * z[i])
                        .rem_euclid(beta_p[i]);
                    let term = (term * Minv_M_p[i]).rem_euclid(beta_p[i]);
                    value += term << (5 * i);
                }
                matrix[j][l] = value;
            }
        }
        sb.insert(key, matrix);
    }
    print_progress(tmp_sk_ntt.degree, tmp_sk_ntt.degree, 64);
    sb
}

/// prepare_first_box_MM (no encoding version)
pub fn prepare_first_box_MM(
    sk: &mut LatticeVector,
    root: i64,
    unroot: i64,
    ninv: i64,
    beta: &[i64],
    k: usize,
) -> HashMap<String, Vec<Vec<i64>>> {
    sk.goto_ntt(root);
    let M = product(beta);
    let N = sk.modulus.value;
    let egcd = xgcd(N, M);
    let Ninv = egcd.x;
    let Ninv_M = goto_crt_k(Ninv, beta, k);

    let mut fb = HashMap::new();
    for dim in 0..sk.degree {
        print_progress(dim, sk.degree, 64);
        let key = format!("fb_dim_{}", dim);
        let mut matrix = vec![vec![0i64; 32]; 32];
        let s = goto_crt_k(sk.vector[dim], beta, k);
        for j in 0..32 {
            let a = goto_crt_k(j as i64, beta, k);
            for l in 0..32 {
                let b = goto_crt_k(l as i64, beta, k);
                let mut value = 0;
                for i in 0..k {
                    let term = (a[i] * s[i] + b[i]).rem_euclid(beta[i]);
                    let term = (-Ninv_M[i] * term).rem_euclid(beta[i]);
                    value += term << (5 * i);
                }
                matrix[j][l] = value;
            }
        }
        fb.insert(key, matrix);
    }
    print_progress(sk.degree, sk.degree, 64);
    sk.goback_ntt(unroot, ninv);
    fb
}

/// prepare_second_box_MM (no encoding version, using beta_p)
pub fn prepare_second_box_MM(
    sk: &mut LatticeVector,
    root: i64,
    unroot: i64,
    ninv: i64,
    beta: &[i64],
    beta_p: &[i64],
    k: usize,
) -> HashMap<String, Vec<Vec<i64>>> {
    sk.goto_ntt(root);
    let M = product(beta);
    let M_p = product(beta_p);
    let egcd = xgcd(M, M_p);
    let Minv = egcd.x;
    let Minv_M_p = goto_crt_k(Minv, beta_p, k);

    let mut sb = HashMap::new();
    for dim in 0..sk.degree {
        print_progress(dim, sk.degree, 64);
        let key = format!("sb_dim_{}", dim);
        let mut matrix = vec![vec![0i64; 32]; 32];
        let s = goto_crt_k(sk.vector[dim], beta_p, k);
        for j in 0..32 {
            let a = goto_crt_k(j as i64, beta_p, k);
            for l in 0..32 {
                let b = goto_crt_k(l as i64, beta_p, k);
                let mut value = 0;
                for i in 0..k {
                    let term = (a[i] * s[i] + b[i]).rem_euclid(beta_p[i]);
                    let term = (term * Minv_M_p[i]).rem_euclid(beta_p[i]);
                    value += term << (5 * i);
                }
                matrix[j][l] = value;
            }
        }
        sb.insert(key, matrix);
    }
    print_progress(sk.degree, sk.degree, 64);
    sk.goback_ntt(unroot, ninv);
    sb
}

// ---------- write_data ----------

/// Generates keys, encrypts masks, computes NTT parameters and lookup tables,
/// and writes JSON files with private, public, and white‐box decryption data.
pub fn write_data(
    degree: usize,
    modulus_val: i64,
    beta: &[i64],
    beta_p: &[i64],
    k: usize,
    chal: i64,
) -> Result<(), String> {
    // Set WB keys.
    let modulus = Modulus { value: modulus_val };
    let (pka, pkb, mut sk) = key_gen(degree, modulus);

    // Set homomorphic masks.
    let mut one_vec = vec![0i64; degree];
    one_vec[0] = 1;
    let one = LatticeVector::new(one_vec, modulus, false);
    let (a1_o, a2_o) = encrypt(&one, &pka, &pkb);

    let zero_vec = vec![0i64; degree];
    let zero = LatticeVector::new(zero_vec, modulus, false);
    let (a1_z, a2_z) = encrypt(&zero, &pka, &pkb);

    let mut rotate_vec = vec![0i64; degree];
    let rot_index = rand::thread_rng().gen_range(0..degree);
    rotate_vec[rot_index] = 1;
    let rotate = LatticeVector::new(rotate_vec, modulus, false);
    let (a1_rot, a2_rot) = encrypt(&rotate, &pka, &pkb);

    let mut mask_vec = vec![0i64; degree];
    for i in 0..degree {
        mask_vec[i] = (rand::thread_rng().gen::<u8>() % 2) as i64;
    }
    let mask = LatticeVector::new(mask_vec, modulus, false);
    let (a1_ma, a2_ma) = encrypt(&mask, &pka, &pkb);

    // Set NTT parameters.
    let root = find_primitive_root((2 * degree) as i64, (modulus_val - 1) as i64, modulus_val)
        .ok_or("No primitive root found")?;
    let unroot = xgcd(root, modulus_val).x;
    let ninv = xgcd(degree as i64, modulus_val).x;

    // Write private data.
    let private_data = json!({
        "sk": sk.vector,
        "a1_o": a1_o.vector,
        "a2_o": a2_o.vector,
        "a1_z": a1_z.vector,
        "a2_z": a2_z.vector,
    });
    fs::write("private_data.json", private_data.to_string())
        .map_err(|e| e.to_string())?;

    // Write public encryption data.
    let pub_enc_data = json!({
        "degree": degree,
        "modulus": modulus_val,
        "pka": pka.vector,
        "pkb": pkb.vector,
    });
    fs::write("pub_enc_data.json", pub_enc_data.to_string())
        .map_err(|e| e.to_string())?;

    // Prepare white-box decryption data.
    let mut wb_data = json!({
        "root": root,
        "unroot": unroot,
        "ninv": ninv,
        "beta": beta,
        "beta_p": beta_p,
        "k": k,
        "mask": mask.vector,
        "rotate": rot_index,
        "chal": chal,
    });

    // Depending on chal, compute and merge lookup tables.
    if chal == 0 {
        println!("prepare_first_box_MM");
        let fb = prepare_first_box_MM(&mut sk, root, unroot, ninv, beta, k);
        println!("prepare_second_box_MM");
        let sb = prepare_second_box_MM(&mut sk, root, unroot, ninv, beta, beta_p, k);
        wb_data.as_object_mut().unwrap().extend(
            fb.into_iter().map(|(k, v)| (k, json!(v)))
        );
        wb_data.as_object_mut().unwrap().extend(
            sb.into_iter().map(|(k, v)| (k, json!(v)))
        );
    } else if chal == 1 {
        println!("prepare_first_box_MM2");
        let fb = prepare_first_box_MM2(&sk, &a1_o, &a2_o, &a1_z, &a2_z, root, unroot, ninv, beta, k);
        println!("prepare_second_box_MM2");
        let sb = prepare_second_box_MM2(&sk, &a1_o, &a2_o, &a1_z, &a2_z, root, unroot, ninv, beta, beta_p, k);
        wb_data.as_object_mut().unwrap().extend(
            fb.into_iter().map(|(k, v)| (k, json!(v)))
        );
        wb_data.as_object_mut().unwrap().extend(
            sb.into_iter().map(|(k, v)| (k, json!(v)))
        );
    } else if chal == 2 {
        println!("prepare_first_box_MM3");
        let fb = prepare_first_box_MM3(&sk, &a1_rot, &a2_rot, &a1_ma, &a2_ma, root, unroot, ninv, beta, k);
        println!("prepare_second_box_MM3");
        let sb = prepare_second_box_MM3(&sk, &a1_rot, &a2_rot, &a1_ma, &a2_ma, root, unroot, ninv, beta, beta_p, k);
        wb_data.as_object_mut().unwrap().extend(
            fb.into_iter().map(|(k, v)| (k, json!(v)))
        );
        wb_data.as_object_mut().unwrap().extend(
            sb.into_iter().map(|(k, v)| (k, json!(v)))
        );
    }
    fs::write("wb_dec_data.json", wb_data.to_string())
        .map_err(|e| e.to_string())?;

    Ok(())
}
