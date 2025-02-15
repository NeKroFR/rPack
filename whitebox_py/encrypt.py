#!/usr/bin/env python3

import json
import numpy as np
import sys
from lattice import NTRU_vector, encrypt

message = sys.argv[1]
message_bytes = message.encode()

with open('pub_enc_data.json') as f:
    data = json.load(f)

degree = data['degree']
modulus = data['modulus']
pka = NTRU_vector(degree, modulus, False)
pkb = NTRU_vector(degree, modulus, False)
pka.vector = np.array(data['pka'])
pkb.vector = np.array(data['pkb'])

message_bits = np.array([int(b) for byte in message_bytes for b in f"{byte:08b}"])
if len(message_bits) > degree:
    print("Error: Message too long for encryption.")
    sys.exit(1)

message = np.zeros(degree, dtype=int)
message[:len(message_bits)] = message_bits

a1, a2 = encrypt(message, pka, pkb, degree, modulus)

with open('ciphertext.json', 'w') as f:
    json.dump({'a1': a1.vector.tolist(), 'a2': a2.vector.tolist()}, f)

print("Message encrypted and saved to ciphertext.json")
