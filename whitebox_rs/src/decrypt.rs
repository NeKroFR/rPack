use numpy::ndarray::Array1;
use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::File;
use std::collections::HashMap;

type Array1i64 = Array1<i64>;


#[derive(Debug, Serialize, Deserialize, Clone)]
struct NTRUVector {
    vector: Array1i64,
    degree: usize,
    modulus: i64,
    ntt: bool,
}

impl NTRUVector {
    fn new(degree: usize, modulus: i64, ntt: bool) -> Self {
        NTRUVector {
            vector: Array1::zeros(degree),
            degree,
            modulus,
            ntt,
        }
    }

    // These methods are currently unused, but kept for potential future use or completeness.
    // They can be removed if confirmed to be unnecessary.
    /*
    fn add(&self, other: &NTRUVector) -> Self {
        let mut res = NTRUVector::new(self.degree, self.modulus, self.ntt);
        res.vector = self.vector.iter().zip(other.vector.iter())
            .map(|(s, o)| (s + (o % self.modulus)).rem_euclid(self.modulus))
            .collect();
        res
    }

    fn sub(&self, other: &NTRUVector) -> Self {
        let mut res = NTRUVector::new(self.degree, self.modulus, self.ntt);
        res.vector = self.vector.iter().zip(other.vector.iter())
            .map(|(s, o)| (s - (o % self.modulus)).rem_euclid(self.modulus))
            .collect();
        res
    }

    fn mul(&self, other: &NTRUVector) -> Self {
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

    fn neg(&mut self) -> &mut Self {
        self.vector = self.vector.iter()
            .map(|s| (-s).rem_euclid(self.modulus))
            .collect();
        self
    }
    */


    fn goto_ntt(&mut self, root: i64) {
        if self.ntt {
            println!("This vector is already ntt");
        } else {
            let n = self.degree;
            self.ntt = true;
            self.degree = n;
            let levels = n.ilog2();
            let mut powtable = Vec::new();
            let mut temp = 1;
            for i in 0..n {
                self.vector[i] = (self.vector[i] * temp).rem_euclid(self.modulus);
                if i % 2 == 0 {
                    powtable.push(temp);
                }
                temp = (temp * root).rem_euclid(self.modulus);
            }

            fn reverse_bits(x: usize, bits: u32) -> usize {
                let mut y = 0;
                let mut x_temp = x;
                for _ in 0..bits {
                    y = (y << 1) | (x_temp & 1);
                    x_temp >>= 1;
                }
                y
            }

            for i in 0..n {
                let j = reverse_bits(i, levels);
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
                    for j in i..(i + halfsize) {
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
    }

    fn goback_ntt(&mut self, unroot: i64, ninv: i64) {
        if !self.ntt {
            println!("This vector is not ntt");
        } else {
            self.ntt = false;
            let n = self.degree;
            let mut res = NTRUVector::new(n, self.modulus, false);
            res.vector.assign(&self.vector);

            let levels = n.ilog2();
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


            fn reverse_bits(x: usize, bits: u32) -> usize {
                let mut y = 0;
                let mut x_temp = x;
                for _ in 0..bits {
                    y = (y << 1) | (x_temp & 1);
                    x_temp >>= 1;
                }
                y
            }
            for i in 0..n {
                let j = reverse_bits(i, levels);
                if j > i {
                    res.vector.swap(i, j);
                }
            }

            let mut size = 2;
            while size <= n {
                let halfsize = size / 2;
                let tablestep = n / size;
                for i in (0..n).step_by(size) {
                    let mut k = 0;
                    for j in i..(i + halfsize) {
                        let l = j + halfsize;
                        let left = res.vector[j];
                        let right = (res.vector[l] * powtable[k]).rem_euclid(self.modulus);
                        res.vector[j] = (left + right).rem_euclid(self.modulus);
                        res.vector[l] = (left - right).rem_euclid(self.modulus);
                        k += tablestep;
                    }
                }
                size *= 2;
            }
            self.vector = res.vector.iter().enumerate()
                .map(|(i, val)| ((val * ninv).rem_euclid(self.modulus) * powtable2[i]).rem_euclid(self.modulus))
                .collect();
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct WBVector {
    #[serde(flatten)]
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
    }

    fn mont_mult(dim: usize, a: i64, b: i64, n: i64, white_data: &WhiteData) -> i64 {
        let b_val = &white_data.beta;
        let b_p_val = &white_data.beta_p;
        let k_val = white_data.k;

        let a_m = goto_crt(a, b_val, k_val);
        let b_m = goto_crt(b, b_val, k_val);
        let a_m_p = goto_crt(a, b_p_val, k_val);
        let b_m_p = goto_crt(b, b_p_val, k_val);

        let m_val: i64 = b_val.iter().product();
        let m_p_val: i64 = b_p_val.iter().product();

        let minv_m_p = goto_crt(xgcd(m_val, m_p_val).1, b_p_val, k_val);
        let n_m_p = goto_crt(n, b_p_val, k_val);


        let fb = &white_data.fb_dim;
        let mut q = vec![0i64; k_val];
        q[0] = (fb[&format!("fb_dim_{}", dim)][a_m[0] as usize][b_m[0] as usize] % (1 << 5)) as i64;
        q[1] = ((fb[&format!("fb_dim_{}", dim)][a_m[1] as usize][b_m[1] as usize] % (1 << 10)) >> 5) as i64;
        q[2] = ((fb[&format!("fb_dim_{}", dim)][a_m[2] as usize][b_m[2] as usize] % (1 << 15)) >> 10) as i64;
        q[3] = ((fb[&format!("fb_dim_{}", dim)][a_m[3] as usize][b_m[3] as usize] % (1 << 20)) >> 15) as i64;
        q[4] = (fb[&format!("fb_dim_{}", dim)][a_m[4] as usize][b_m[4] as usize] >> 20) as i64;

        let q_crt = goback_crt(&q, b_val, k_val);
        let q_crt_vec = goto_crt(q_crt, b_p_val, k_val);


        let sb = &white_data.sb_dim;
        let mut r = vec![0i64; k_val];
        for i in 0..k_val {
            r[i] = ((q_crt_vec[i] * n_m_p[i] % b_p_val[i]) * minv_m_p[i]).rem_euclid(b_p_val[i]);
        }
        r[0] = (r[0] + (sb[&format!("sb_dim_{}", dim)][a_m_p[0] as usize][b_m_p[0] as usize] % (1 << 5)) as i64).rem_euclid(b_p_val[0]);
        r[1] = (r[1] + ((sb[&format!("sb_dim_{}", dim)][a_m_p[1] as usize][b_m_p[1] as usize] % (1 << 10)) >> 5) as i64).rem_euclid(b_p_val[1]);
        r[2] = (r[2] + ((sb[&format!("sb_dim_{}", dim)][a_m_p[2] as usize][b_m_p[2] as usize] % (1 << 15)) >> 10) as i64).rem_euclid(b_p_val[2]);
        r[3] = (r[3] + ((sb[&format!("sb_dim_{}", dim)][a_m_p[3] as usize][b_m_p[3] as usize] % (1 << 20)) >> 15) as i64).rem_euclid(b_p_val[3]);
        r[4] = (r[4] + (sb[&format!("sb_dim_{}", dim)][a_m_p[4] as usize][b_m_p[4] as usize] >> 20) as i64).rem_euclid(b_p_val[4]);

        let r_crt = goback_crt(&r, b_p_val, k_val);

        (r_crt * m_val).rem_euclid(n)
    }
}


fn goto_crt(x: i64, base: &Vec<i64>, l: usize) -> Vec<i64> {
    (0..l).map(|i| (x % base[i]).rem_euclid(base[i])).collect()
}

fn goback_crt(x_b: &Vec<i64>, base: &Vec<i64>, l: usize) -> i64 {
    let mut x = 0;
    let b_prod: i64 = base.iter().product();
    for i in 0..l {
        let b_i = b_prod / base[i];
        x = (x + (x_b[i] * b_i % b_prod).rem_euclid(b_prod) * xgcd(b_i, base[i]).1 % b_prod).rem_euclid(b_prod);
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

#[derive(Debug, Serialize, Deserialize)]
struct WhiteData {
    root: i64,
    unroot: i64,
    ninv: i64,
    beta: Vec<i64>,
    beta_p: Vec<i64>,
    k: usize,
    mask: Vec<f64>,
    rotate: usize,
    chal: usize,
    #[serde(flatten)]
    fb_dim: HashMap<String, Vec<Vec<i64>>>,
    #[serde(flatten)]
    sb_dim: HashMap<String, Vec<Vec<i64>>>,
}


fn decrypt_white(a1_vec: &NTRUVector, a2_vec: &NTRUVector, degree: usize, modulus: i64, white_data: &WhiteData) -> Array1i64 {
    let mut tmp_a1 = WBVector::from_ntru_vector(NTRUVector {
        vector: a1_vec.vector.clone(),
        degree: a1_vec.degree,
        modulus: a1_vec.modulus,
        ntt: a1_vec.ntt,
    });
    let mut tmp_a2 = WBVector::from_ntru_vector(NTRUVector {
        vector: a2_vec.vector.clone(),
        degree: a2_vec.degree,
        modulus: a2_vec.modulus,
        ntt: a2_vec.ntt,
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
    let rot = white_data.rotate;

    let mut m = Array1::zeros(degree);
    for i in 0..degree {
        let m_val;
        if chal == 2 {
            m_val = tmp_ntru.vector[(i + rot) % degree].rem_euclid(tmp_ntru.modulus);
            if m_val > modulus / 2 {
                m[i] = (1 - ((m_val + (mask[i] as i64)) % 2)).rem_euclid(2);
            } else {
                m[i] = ((m_val + (mask[i] as i64)) % 2).rem_euclid(2);
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


#[derive(Debug, Deserialize)]
struct CipherData {
    a1: Vec<i64>,
    a2: Vec<i64>,
}

#[derive(Debug, Deserialize)]
struct PubEncData {
    degree: usize,
    modulus: i64,
    pka: Vec<i64>,
    pkb: Vec<i64>,
}


fn load_data() -> Result<(PubEncData, WhiteData, CipherData), serde_json::Error> {
    let pub_enc_data_file = File::open("pub_enc_data.json").map_err(serde_json::Error::io)?;
    let wb_dec_data_file = File::open("wb_dec_data.json").map_err(serde_json::Error::io)?;
    let ciphertext_file = File::open("ciphertext.json").map_err(serde_json::Error::io)?;

    let pub_enc_data: PubEncData = serde_json::from_reader(pub_enc_data_file)?;
    let white_data: WhiteData = serde_json::from_reader(wb_dec_data_file)?;
    let ciphertext: CipherData = serde_json::from_reader(ciphertext_file)?;

    println!("WB_vector.white data loaded successfully.");
    Ok((pub_enc_data, white_data, ciphertext))
}


fn decrypt_message(data: &PubEncData, white_data: &WhiteData, ciphertext: &CipherData) -> Array1i64 {
    let degree = data.degree;
    let modulus = data.modulus;

    let _pka = NTRUVector {
        vector: Array1::from_vec(data.pka.clone()),
        degree,
        modulus,
        ntt: false,
    };
    let _pkb = NTRUVector {
        vector: Array1::from_vec(data.pkb.clone()),
        degree,
        modulus,
        ntt: false,
    };
    let a1 = NTRUVector {
        vector: Array1::from_vec(ciphertext.a1.clone()),
        degree,
        modulus,
        ntt: false,
    };
    let a2 = NTRUVector {
        vector: Array1::from_vec(ciphertext.a2.clone()),
        degree,
        modulus,
        ntt: false,
    };

    decrypt_white(&a1, &a2, degree, modulus, white_data)
}


fn binary_to_text(binary_str: String) -> String {
    let mut binary_str_mut = binary_str;
    if binary_str_mut.len() % 8 != 0 {
        binary_str_mut = format!("{:0<width$}", binary_str_mut, width = binary_str_mut.len() + (8 - binary_str_mut.len() % 8));
    }

    let binary_values: Vec<&str> = (0..binary_str_mut.len())
        .step_by(8)
        .map(|i| &binary_str_mut[i..i + 8])
        .collect();

    let ascii_chars: Vec<char> = binary_values
        .iter()
        .filter_map(|bv| {
            let val = u8::from_str_radix(bv, 2).unwrap_or(0);
            Some(val as char)
        })
        .collect();

    ascii_chars.into_iter().collect()
}


pub fn decrypt_json() -> serde_json::Result<()> {
    println!("Loading data...");
    let (data, white_data, ciphertext) = load_data()?;

    println!("Decrypting message...");
    let decrypted_message = decrypt_message(&data, &white_data, &ciphertext);

    let decrypted_message_str = decrypted_message.iter().map(|&x| x.to_string()).collect::<String>();
    let readable_message = binary_to_text(decrypted_message_str);

    println!("Decrypted message: {}", readable_message);
    Ok(())
}
