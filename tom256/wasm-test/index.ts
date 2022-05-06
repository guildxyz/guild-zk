(async () => {
	const { membership_proof_test } = await import("../pkg");

	try {
		const result = membership_proof_test(4);
		console.log(result)
	} catch (error) {
		console.log(error)
	}
}) ()
