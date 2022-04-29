(async () => {
	const { wasm_build_test } = await import("../pkg");

	console.log(wasm_build_test("0xef"))
	console.log(wasm_build_test("0xefz"))
}) ()
