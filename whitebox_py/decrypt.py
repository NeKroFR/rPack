import json
import numpy as np
import random as rand

class NTRU_vector():

    white = None

    def __init__(self, degree, modulus, ntt):
        self.vector = np.zeros(degree, dtype=np.int64)
        self.degree = degree
        self.modulus = modulus
        self.ntt = ntt

    def __add__(self, other):
        res = NTRU_vector(self.degree, self.modulus, self.ntt)
        res.vector = np.array([self.vector[i] + other.vector[i] % self.modulus for i in range(self.degree)])
        return res

    def __sub__(self, other):
        res = NTRU_vector(self.degree, self.modulus, self.ntt)
        res.vector = np.array([self.vector[i] - other.vector[i] % self.modulus for i in range(self.degree)])
        return res

    def __mul__(self, other):
        res = NTRU_vector(self.degree, self.modulus, self.ntt)
        if self.ntt:
            for i in range(self.degree):
                x = int(self.vector[i])
                y = int(other.vector[i])
                z = x * y
                res.vector[i] = z
        else:
            for i in range(self.degree):
                for j in range(self.degree):
                    d = i + j
                    if d < self.degree:
                        res.vector[d] = (res.vector[d] + self.vector[i] * other.vector[j]) % self.modulus
                    else:
                        d = d % self.degree
                        res.vector[d] = (res.vector[d] - self.vector[i] * other.vector[j]) % self.modulus
        return res

    def __neg__(self):
        self.vector = np.array([-self.vector[i] % self.modulus for i in range(self.degree)])
        return self

    def goto_ntt(self, root):
        if self.ntt:
            print("This vector is already ntt")
        else:
            n = self.degree
            self.ntt = True
            self.degree = n
            levels = n.bit_length() - 1
            powtable = []
            temp = 1
            for i in range(n):
                self.vector[i] = self.vector[i] * temp % self.modulus
                if not i % 2:
                    powtable.append(temp)
                temp = temp * root % self.modulus

            def reverse(x, bits):
                y = 0
                for i in range(bits):
                    y = (y << 1) | (x & 1)
                    x >>= 1
                return y
            for i in range(n):
                j = reverse(i, levels)
                if j > i:
                    self.vector[i], self.vector[j] = self.vector[j], self.vector[i]

            size = 2
            while size <= n:
                halfsize = size // 2
                tablestep = n // size
                for i in range(0, n, size):
                    k = 0
                    for j in range(i, i + halfsize):
                        l = j + halfsize
                        left = self.vector[j]
                        right = self.vector[l] * powtable[k]
                        self.vector[j] = (left + right) % self.modulus
                        self.vector[l] = (left - right) % self.modulus
                        k += tablestep
                size *= 2

    def goback_ntt(self, unroot, ninv):
        if not self.ntt:
            print("This vector is not ntt")
        else:
            self.ntt = False
            n = self.degree
            res = NTRU_vector(n, self.modulus, False)
            res.vector = self.vector

            levels = n.bit_length() - 1
            powtable = []
            powtable2 = []
            temp = 1
            for i in range(n):
                if not i % 2:
                    powtable.append(temp)
                powtable2.append(temp)
                temp = temp * unroot % self.modulus

            def reverse(x, bits):
                y = 0
                for i in range(bits):
                    y = (y << 1) | (x & 1)
                    x >>= 1
                return y
            for i in range(n):
                j = reverse(i, levels)
                if j > i:
                    res.vector[i], res.vector[j] = res.vector[j], res.vector[i]

            size = 2
            while size <= n:
                halfsize = size // 2
                tablestep = n // size
                for i in range(0, n, size):
                    k = 0
                    for j in range(i, i + halfsize):
                        l = j + halfsize
                        left = res.vector[j]
                        right = res.vector[l] * powtable[k]
                        res.vector[j] = (left + right) % self.modulus
                        res.vector[l] = (left - right) % self.modulus
                        k += tablestep
                size *= 2

            self.vector = np.array([res.vector[i] * ninv * powtable2[i] % self.modulus for i in range(self.degree)])


class WB_vector(NTRU_vector):

    def __mul__(self, other):
        res = NTRU_vector(self.degree, self.modulus, self.ntt)
        if self.ntt:
            self.my_mult(other, res)
        else:
            print("WB vector must be turned in ntt form")
        return res

    def my_mult(self, other, res):
        for i in range(self.degree):
            x = int(self.vector[i])
            y = int(other.vector[i])
            z = WB_vector.mont_mult(i, x, y, self.modulus)
            res.vector[i] = z

    @classmethod
    def mont_mult(cls, dim, a, b, N):
        B = cls.white['beta']
        B_p = cls.white['beta_p']
        k = cls.white['k']
        a_M = goto_crt(a, B, k)
        b_M = goto_crt(b, B, k)
        a_M_p = goto_crt(a, B_p, k)
        b_M_p = goto_crt(b, B_p, k)
        M = np.prod(B)
        M_p = np.prod(B_p)
        Minv_M_p = goto_crt(xgcd(M, M_p)[1], B_p, k)
        N_M_p = goto_crt(N, B_p, k)

        fb = cls.white['fb_dim_' + str(dim)]
        q = [0] * k
        q[0] = fb[a_M[0]][b_M[0]] % (1 << 5)
        q[1] = (fb[a_M[1]][b_M[1]] % (1 << 10)) >> 5
        q[2] = (fb[a_M[2]][b_M[2]] % (1 << 15)) >> 10
        q[3] = (fb[a_M[3]][b_M[3]] % (1 << 20)) >> 15
        q[4] = (fb[a_M[4]][b_M[4]]) >> 20

        q = goback_crt(q, B, k)
        q = goto_crt(q, B_p, k)

        sb = cls.white['sb_dim_' + str(dim)]
        r = [(q[i] * N_M_p[i] * Minv_M_p[i]) % B_p[i] for i in range(k)]
        r[0] += sb[a_M_p[0]][b_M_p[0]] % (1 << 5)
        r[1] += (sb[a_M_p[1]][b_M_p[1]] % (1 << 10)) >> 5
        r[2] += (sb[a_M_p[2]][b_M_p[2]] % (1 << 15)) >> 10
        r[3] += (sb[a_M_p[3]][b_M_p[3]] % (1 << 20)) >> 15
        r[4] += (sb[a_M_p[4]][b_M_p[4]]) >> 20

        r = goback_crt(r, B_p, k)

        return r * M


def goto_crt(x, base, l):
    return [x % base[i] for i in range(l)]


def goback_crt(x_b, base, l):
    x = 0
    B = np.prod(base)
    for i in range(l):
        B_i = B / base[i]
        x += (x_b[i] * B_i * xgcd(B_i, base[i])[1])
    return x % B


def xgcd(b, n):
    x0, x1, y0, y1 = 1, 0, 0, 1
    while n != 0:
        q, b, n = b // n, n, b % n
        x0, x1 = x1, x0 - q * x1
        y0, y1 = y1, y0 - q * y1
    return b, x0, y0


def decrypt_white(a1, a2, degree, modulus, debug=[]):
    tmp_a1 = WB_vector(degree, modulus, False)
    tmp_a1.vector = a1.vector
    tmp_a2 = WB_vector(degree, modulus, False)
    tmp_a2.vector = a2.vector

    root = WB_vector.white['root']
    unroot = WB_vector.white['unroot']
    ninv = WB_vector.white['ninv']
    tmp_a1.goto_ntt(root)
    tmp_a2.goto_ntt(root)

    tmp = tmp_a1 * tmp_a2

    tmp.goback_ntt(unroot, ninv)

    chal = WB_vector.white['chal']
    if chal == 2:
        mask = WB_vector.white['mask']
        rot = WB_vector.white['rotate']

    m = np.zeros(degree, dtype=int)
    for i in range(degree):
        if chal == 2:
            m[i] = tmp.vector[(i + rot) % degree] % tmp.modulus
            if m[i] > modulus / 2:
                m[i] = 1 - ((m[i] + mask[i]) % 2)
            else:
                m[i] = (m[i] + mask[i]) % 2
        else:
            m[i] = tmp.vector[i] % tmp.modulus
            if m[i] > modulus / 2:
                m[i] = 1 - (m[i] % 2)
            else:
                m[i] = m[i] % 2
    return m


def load_data():
    try:
        with open('pub_enc_data.json') as f:
            data = json.load(f)
    except FileNotFoundError:
        print("There is no data for Encryption")
        exit()

    try:
        with open('wb_dec_data.json') as f:
            WB_vector.white = json.load(f)
            print("WB_vector.white data loaded successfully.")
            #print(WB_vector.white)
    except FileNotFoundError:
        print("There is no data for WB Decryption")
        exit()

    return data


def decrypt_message(data):
    degree = data['degree']
    modulus = data['modulus']
    
    pka = NTRU_vector(degree, modulus, False)
    pkb = NTRU_vector(degree, modulus, False)
    pka.vector = np.array(data['pka'])
    pkb.vector = np.array(data['pkb'])
    
    try:
        with open('ciphertext.json') as f:
            ciphertext = json.load(f)
            a1 = NTRU_vector(degree, modulus, False)
            a2 = NTRU_vector(degree, modulus, False)
            a1.vector = np.array(ciphertext['a1'])
            a2.vector = np.array(ciphertext['a2'])
    except FileNotFoundError:
        print("Ciphertext not found. Make sure the encryption process was done first.")
        exit()

    decrypted_message = decrypt_white(a1, a2, degree, modulus)
    return decrypted_message


def binary_to_text(binary_str):
    if len(binary_str) % 8 != 0:
        binary_str = binary_str.ljust(len(binary_str) + (8 - len(binary_str) % 8), '0')
    
    binary_values = [binary_str[i:i+8] for i in range(0, len(binary_str), 8)]
    
    ascii_chars = [chr(int(bv, 2)) for bv in binary_values if int(bv, 2) < 256]
    return ''.join(ascii_chars)


def main():
    print("Loading data...")
    data = load_data()
    
    print("Decrypting message...")
    decrypted_message = decrypt_message(data)
    
    decrypted_message_str = ''.join(map(str, decrypted_message))
    
    readable_message = binary_to_text(decrypted_message_str)
    
    print("Decrypted message:", readable_message)



if __name__ == "__main__":
    main()
