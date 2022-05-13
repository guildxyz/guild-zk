#!/bin/sh

# if a second argument is present that means
# we would like to build wasm to a different target from "web"
if [ $1 ]; then
	WASM_TARGET=$1
else
	WASM_TARGET="bundler"
fi

echo "Wasm build target: ${WASM_TARGET}"

wasm-pack build client --target ${WASM_TARGET} --out-name index --out-dir ../wasm-out