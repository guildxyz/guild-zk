(async () => {
	const { membershipProofTest, generatePedersenParams, commitAddress } = await import("../pkg");

	try {
		const address = "0x0679349AeA848f928cE886fbAE10a85660CBFecD"
		const pedersen = generatePedersenParams();
		const commitment = commitAddress(address, pedersen);
		// TODO sign this msg
		// console.log(commitment.commitment.x + commitment.commitment.y + commitment.commitment.z)
		const result = membershipProofTest(address, commitment, pedersen);
		console.log(result)
	} catch (error) {
		console.log(error)
	}
}) ()
/*
{
  "message": "hello-test",
  "hashMessage": "0x1ab4850e7f0a85a521e87b274e3130efdb45f6a47e74e6dcebf5591c6bc8f16e",
  "signature": "0x45c4039b611c0cc207ff7fb7a6899ea0431aac2cf37515d74a71f2df00e2c3e0096fad5e7eda762898fffd4644f8a7a406bf6bde868814ea03058c882fcd23311c",
  "publicKey": "0x0408c6cd9400645819c8c556a6e83e0a7728f070a813bb9d24d5c24290e21fc5e438396f9333264d3e7c1d3e6ee1bc572b2f00b98db7065e9bf278f2b8dbe02718",
  "address": "0x0679349AeA848f928cE886fbAE10a85660CBFecD"
}
*/
