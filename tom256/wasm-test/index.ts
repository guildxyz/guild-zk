(async () => {
	const { generateProof, verifyProof } = await import("../pkg");

	try {
		//const start = performance.now();
		const input = {
			msgHash: "0x9788117298a1450f6002d25f0c21d83bc6001681a2e5e31c748c0f55504b11e9",
			signature: "0xd2943d5fa0ba2733bcbbd58853c6c1be65388d9198dcb5228e117f49409612a46394afb97a7610d16e7bea0062e71afc2a3039324c80df8ef38d3668164fad2c1c",
			pubkey:"0454e32170dd5a0b7b641aa77daa1f3f31b8df17e51aaba6cfcb310848d26351180b6ac0399d21460443d10072700b64b454d70bfba5e93601536c740bbd099682",
			index: 2,
			guildId: "almafa",
		};

		const ring = [
            "ddd40afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172",
            "ccc50afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172",
            "54e32170dd5a0b7b641aa77daa1f3f31b8df17e51aaba6cfcb310848d2635118", // our pubkey x
            "aaa70afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172",
            "bbb80afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172",
		];

		const proof = generateProof(input, ring);
		console.log(proof);

		const result = verifyProof(proof.proofBinary, ring);
		//const elapsed = performance.now() - start;
		console.log(result)
		//console.log(elapsed / 1000)
	} catch (error) {
		console.log(error)
	}
}) ()
