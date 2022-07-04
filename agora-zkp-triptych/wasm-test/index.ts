(async () => {
	const {sign, verify} = await import("../pkg");

	// TODO write proof to file
	// TODO add verifier example that parses the ring and proof and verifies it in rust
	const privkey = "0xc1e0a5f33f8551ca8725f0b20cfe7a033ff863809100c3301009f2efac3810d6";
	const msgHash = "0xaaaaaaaabbbbbbbbccccccccddddddddeeeeeeeeffffffff0000000011111111";
	const index = 4;
	const ring = require("./ring.json");
	try {
		let proof = sign(msgHash, privkey, index, ring);
		let result = verify(msgHash, proof, ring);
		console.log(result)
	} catch(error) {
		console.error(error)
	}
}) ()
