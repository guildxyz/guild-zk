#from fastecdsa.curve import secp256k1 as k256
from fastecdsa.point import Point
from fastecdsa.curve import Curve

k256 = Curve(
		name = "k256",
		p = 
		a = 
		b = 
		q = 
		gx = 
		gy =
)


#xs = 0xB8F0170E293FCC9291BEE2665E9CA9B25D3B11810ED68D9EA0CB440D7064E4DA
#ys = 0x0691AA44502212591132AA6F27582B78F9976998DE355C4EE5960DB05AC0A2A3
#xs = 0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798
#ys = 0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8
S = Point(xs, ys, curve=k256)

T = 2 * S
print(T)
print(hex(T.x))
print(hex(T.y))

# Point Addition
R = S + T

# Point Subtraction: (xs, ys) - (xt, yt) = (xs, ys) + (xt, -yt)
R = S - T

# Point Doubling
R = S + S  # produces the same value as the operation below
R = 2 * S  # S * 2 works fine too i.e. order doesn't matter

d = 0xc51e4753afdec1e6b6c6a5b992f43f8dd0c7a8933072708b6522468b2ffb06fd

# Scalar Multiplication
R = d * S  # S * d works fine too i.e. order doesn't matter

e = 0xd37f628ece72a462f0145cbefe3f0b355ee8332d37acdd83a358016aea029db7

# Joint Scalar Multiplication
R = d * S + e * T
