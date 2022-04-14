import sympy.ntheory as nt
a = int("0xffffffff0000000100000000000000017e72b42b30e7317793135661b1c4b114", 0)
b = int("0xb441071b12f4a0366fb552f8e21ed4ac36b06aceeb354224863e60f20219fc56", 0)
#p = int("0xffffffff0000000100000000000000017e72b42b30e7317793135661b1c4b117", 0)
p = 2**224 * (2**32 - 1) + 2**192 + 2**96 - 1

gx = int("0x3", 0)
gy = int("0x5a6dd32df58708e64e97345cbe66600decd9d538a351bb3c30b4954925b1f02d", 0)

gy2 = (gx**3 + a * gx + b) % p
#print(hex(gy2))
#print(hex(gy**2 % p))
print(nt.isprime(p))

r = 2**256 % p
r2 = 2**512 % p

# NOTE these 3 all return 0
#print((p * 2**512) % p)
#print(p * r2 % p)
#print((p * r2) % p)

print("p = ", hex(p))
#print("gx = ", hex(gx))
#print("gy = ", hex(gy))
#print("a = ", hex(a))
#print("b = ", hex(b))
print("r = ", hex(r))
print("r2 = ", hex(r2))

p = int("0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff", 0)
print(nt.isprime(p))

print(hex(2**256))

#export const p256 = new WeierstrassGroup(
#    BigInt('0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff'),
#    BigInt('0xffffffff00000001000000000000000000000000fffffffffffffffffffffffc'),
#    BigInt('0x5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b'),
#    BigInt('0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551'),
#    [
#        BigInt('0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296'),
#        BigInt('0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5'),
#    ]
#)
#
#export const war256 = new WeierstrassGroup(
#    'war256',
#    BigInt('0xffffffff0000000100000000000000017e72b42b30e7317793135661b1c4b117'), p
#    BigInt('0xffffffff0000000100000000000000017e72b42b30e7317793135661b1c4b114'), a
#    BigInt('0xb441071b12f4a0366fb552f8e21ed4ac36b06aceeb354224863e60f20219fc56'), b
#    BigInt('0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff'), order
#    [BigInt('0x3'), BigInt('0x5a6dd32df58708e64e97345cbe66600decd9d538a351bb3c30b4954925b1f02d')]
#)
#
#// tomEdwards256: ax^2+y^2 = 1 + dx^2y^2
#export const tomEdwards256 = new TEdwards(
#    'tomEdwards256',
#    BigInt('0x3fffffffc000000040000000000000002ae382c7957cc4ff9713c3d82bc47d3af'),
#    BigInt('0x1abce3fd8e1d7a21252515332a512e09d4249bd5b1ec35e316c02254fe8cedf5d'),
#    BigInt('0x051781d9823abde00ec99295ba542c8b1401874bcbeb9e9c861174c7bca6a02aa'),
#    BigInt('0x0ffffffff00000001000000000000000000000000ffffffffffffffffffffffff'),
#    [
#        BigInt('0x7907055d0a7d4abc3eafdc25d431d9659fbe007ee2d8ddc4e906206ea9ba4fdb'),
#        BigInt('0xbe231cb9f9bf18319c9f081141559b0a33dddccd2221f0464a9cd57081b01a01'),
#    ]
#)
