(async () => {
	const { generateProof, verifyProof } = await import("../pkg");

	try {
		const start = performance.now();
		const input = {
			msgHash: "0x2c31a901b06d2727f458c7eb5c15eb7a794d69f841970f95c39ac092274c2a5a",
			signature: "0xc945f22f92bc9afa7c8929637d3f8694b95a6ae9e276103b2061a0f88d61d8e92aaa9b9eec482d8befd1e1d2a9e2e219f21bd660278aefa9b0641184280cc2d91b",
			pubkey:"0x041296d6ed4e96bc378b8a460de783cdfbf58afbe04b355f1c225fb3e0b92cdc6e349d7005833c933898e2b88eae1cf40250c16352ace3915de65ec86f5bb9b349",
			index: 2,
			guildId: "almafa",
		};

		const ring = [
            "ddd40afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172",
            "ccc50afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172",
            "1296d6ed4e96bc378b8a460de783cdfbf58afbe04b355f1c225fb3e0b92cdc6e", // our pubkey x
            "aaa70afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172",
            "bbb80afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172",
		];

		const proof = generateProof(input, ring);
		const result = verifyProof(proof, ring);
		const elapsed = performance.now() - start;
		console.log(result)
		console.log(elapsed / 1000)
	} catch (error) {
		console.error(error)
	}
}) ()
