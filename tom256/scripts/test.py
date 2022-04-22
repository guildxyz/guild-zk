import random

ps = int("0xfff1", 0)
pm = int("0xffffddddeeee01234577", 0)
pl = int("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141", 0)

a = int("0x617652b9bba98825bfe56f8632d46088bcbaf1dbac087c297682f9d2156e5139", 0)
b = int("0xe7d95f100dfa1650113d52cde817ae2bbde56dffbe69d1b6afc5d6884934fc4c", 0)
prod = int("0xd0febbc44b1942b614c343706e565dd7679efce7d1a630d386f7effbf5d5cf90", 0)

print(hex(a * b))



#print("ps = ", hex(ps))
#print("pm = ", hex(pm))
#print("pl = ", hex(pl))
#
#print("=====================")
#print("SMALL")
#print("=====================")
#for i in range(0, 10):
#	a = random.randint(0, ps - 1)
#	b = random.randint(0, ps - 1)
#	print(hex(a))
#	print(hex(b))
#	print(hex((a * b) % ps))
#
#print("=====================")
#print("MEDIUM")
#print("=====================")
#for i in range(0, 10):
#	a = random.randint(0, pm - 1)
#	b = random.randint(0, pm - 1)
#	print(hex(a))
#	print(hex(b))
#	print(hex((a * b) % pm))
#
#print("=====================")
#print("LARGE")
#print("=====================")
#for i in range(0, 10):
#	a = random.randint(0, pl - 1)
#	b = random.randint(0, pl - 1)
#	print(hex(a))
#	print(hex(b))
#	print(hex((a * b) % pl))
