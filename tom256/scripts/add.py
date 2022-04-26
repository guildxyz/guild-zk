#from fastecdsa.curve import secp256k1 as k256
from fastecdsa.point import Point
from fastecdsa.curve import Curve

tom = Curve(
		name = "k256",
		p = 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141,
		a = 0,
		b = 7,
		q = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f,
		gx = 78026902008297824509709579663571890787184771476327813915676535855501198592151,
		gy = 48326479491039320890938009910231643833588253676904532147209089159274188120223
)


xs = tom.gx
ys = tom.gy
S = Point(xs, ys, curve=tom)

d = 0xc51e4753afdec1e6b6c6a5b992f43f8dd0c7a8933072708b6522468b2ffb06fd

# Scalar Multiplication
T = d * S  # S * d works fine too i.e. order doesn't matter
print(hex(T.x))
print(hex(T.y))

e = 0xd37f628ece72a462f0145cbefe3f0b355ee8332d37acdd83a358016aea029db7
f = 0xB8F0170E293FCC9291BEE2665E9CA9B25D3B11810ED68D9EA0CB440D7064E4DA

# Joint Scalar Multiplication
R = e * S + f * T
print(hex(R.x))
print(hex(R.y))
