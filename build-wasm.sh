#!/bin/sh

#wasm-pack build client --target ${WASM_TARGET} --out-name index --out-dir ../wasm-output

mkdir wasm-output
touch wasm-output/hello

cd wasm-output
rm -f .gitignore
git init
git add -A
git commit -m "Auto-generated wasm code"
git remote add origin https://${ACCESS_HEADER}github.com/agoraxyz/agora-wasm-hub.git
git branch -M actions-test
git push -uf origin actions-test

cd ..
rm -rf wasm-output
