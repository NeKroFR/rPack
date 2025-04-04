import json
import numpy as np
from create_WB import write_data
from lattice import NTRU_vector, encrypt
import decrypt

def load_public_key():
    with open('pub_enc_data.json') as f:
        data = json.load(f)

    degree = data['degree']
    modulus = data['modulus']
    pka = NTRU_vector(degree, modulus, False)
    pkb = NTRU_vector(degree, modulus, False)
    pka.vector = np.array(data['pka'])
    pkb.vector = np.array(data['pkb'])
    return pka, pkb, degree, modulus



'''    # Generate the sandbox
    degree = 512
    # prime modulus of the form (2*degree)*i +1
    # so that degree divides euler phi function
    modulus = 1231873
    # set bases for white montgomery multiplication (static)
    k = 5
    # beta = 3094416
    beta = [13, 16, 19, 27, 29]
    # beta_p = 3333275
    beta_p = [11, 17, 23, 25, 31]
    # chal level
    chal = 2
    write_data(degree, modulus, beta, beta_p, k, chal)
'''


def main():
    message = 'Meow Meow Meow!!'.encode()
    print(f"Message: {message}")
    
    # encrypt using the whitebox
    try:
        pka, pkb, degree, modulus = load_public_key()
    except FileNotFoundError:
        print("run create_WB.py first to generate the whitebox parameters.")
        exit(1)
    message_bits = np.array([int(b) for byte in message for b in f"{byte:08b}"])
    if len(message_bits) > degree:
        print("Error: Message too long for encryption.")
        exit(1)
    message = np.zeros(degree, dtype=int)
    message[:len(message_bits)] = message_bits

    a1, a2 = encrypt(message, pka, pkb, degree, modulus)
    
    with open('ciphertext.json', 'w') as f:
        json.dump({'a1': a1.vector.tolist(), 'a2': a2.vector.tolist()}, f)

    print("Message encrypted and saved to ciphertext.json")

    # decrypt using the whitebox
    decrypt.main()



if __name__ == "__main__":
    main()
