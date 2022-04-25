import random
mod = int("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 0)
base = random.randint(0, mod - 1)
exponent = random.randint(0, mod - 1)
print(hex(base))
print(hex(exponent))

r = 1
q = base
k = exponent

while k > 0:
	if k % 2 != 0:
		r = (r * q) % mod
	
	q = (q * q) % mod
	k = k >> 1

print(hex(r))
