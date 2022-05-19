(async () => {
	const { generateProof, verifyProof, generatePedersenParameters, commitAddress } = await import("../pkg");

	try {
		const start = performance.now();
		const address = "0x2e3Eca6005eb4e30eA51692011612554586feaC9";
		const pedersen = generatePedersenParameters();
		const commitment = commitAddress(address, pedersen);
		const input = {
			msgHash: "0xb42062702a4acb9370edf5c571f2c7a6f448f8c42f3bfa59e622c1c064a94a14",
			signature: "0xb2a7ff958cd78c8e896693b7b76550c8942d6499fb8cd621efb54909f9d51da02bfaadf918f09485740ba252445d40d44440fd810dbf8a9a18049157adcdaa8c1c",
			pubkey: "0x0418a30afe39c280d2f43f05c070988dae7fbae9cdfd5fb6461acd7657e765e172fd55b3589c74fd4987b6004465afff77b039e631a68cdc7df9cd8cfd5cbe2887",
			ring: [
				"0x0e3Eca6005eb4e30eA51692011612554586feaC9",
            	"0x1e3Eca6005eb4e30eA51692011612554586feaC9",
            	address,
            	"0x3e3Eca6005eb4e30eA51692011612554586feaC9",
            	"0x4e3Eca6005eb4e30eA51692011612554586feaC9",
			],
			index: 2,
		};
		const proof = generateProof(input, commitment, pedersen);
		const result = verifyProof(proof);
		const elapsed = performance.now() - start;
		console.log(result)
		console.log(elapsed / 1000)
	} catch (error) {
		console.log(error)
	}
}) ()
