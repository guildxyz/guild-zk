(async () => {
	const { membershipProofTest, generatePedersenParams, commitAddress } = await import("../pkg");

	try {
		const address = "0x0123456789012345678901234567890123456789";
		const pedersen = generatePedersenParams();
		const commitment = commitAddress(address, pedersen);
		const result = membershipProofTest(address, commitment, pedersen);
		console.log(result)
	} catch (error) {
		console.log(error)
	}
}) ()
