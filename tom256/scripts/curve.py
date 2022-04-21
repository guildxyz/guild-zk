import sympy.ntheory as nt
import math
# NIST P-256
#np = 2**224 * (2**32 - 1) + 2**192 + 2**96 - 1
#nn = int("0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551", 0)
#mu = int("0x100000000fffffffffffffffeffffffff43190552df1a6c21012ffd85eedf9bfe", 0)
# SECP256K1
#import sys
#sys.exit()

# secp256k1 (y^2 = x^3 + 7)
p = int("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F", 0) # prime modulus
a = int("0x0000000000000000000000000000000000000000000000000000000000000000", 0) # equation coeff a
b = int("0x0000000000000000000000000000000000000000000000000000000000000007", 0) # equation coeff b
n = int("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 0) # order
gx = int("0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 0) # generator point x
gy = int("0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 0) # generator point y
assert(nt.isprime(p))

p = 2**128 - 7
a = 2**127 + 11
b = int(2**128 / 3) + 11
print(hex((a + b) % p))
import sys
sys.exit()

# cycle of secp256k1
cp = 115792089237316195423570985008687907852837564279074904382605163141518161494337
ca = int("0x0", 0)
cb = int("0x7", 0)
cn = 115792089237316195423570985008687907853269984665640564039457584007908834671663
cgx = 78026902008297824509709579663571890787184771476327813915676535855501198592151
cgy = 48326479491039320890938009910231643833588253676904532147209089159274188120223

assert(nt.isprime(p))
assert(cn == p) # cycle order == base prime modulus

print("================================")
print("SECP256K1")
print("================================")
print("p =", hex(p))
print("a =", hex(a))
print("b =", hex(b))
print("n =", hex(n))
print("gx =", hex(gx))
print("gy =", hex(gy))
print("================================")
print("CYCLE OF SECP256K1")
print("================================")
print("p =", hex(cp))
print("a =", hex(ca))
print("b =", hex(cb))
print("n =", hex(cn))
print("gx =", hex(cgx))
print("gy =", hex(cgy))
