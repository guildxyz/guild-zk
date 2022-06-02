#!/bin/sh

wasm-pack build ${WASM_DIR} --target bundler --out-dir wasm-output

cd wasm-output
rm -f .gitignore
git add -A
git commit -m "Auto-generated wasm code"
git remote add origin https://${ACCESS_HEADER}github.com/${TARGET_REPO}
git branch -M ${TARGET_BRANCH}
git push -uf origin ${TARGET_BRANCH}

cd ..
rm -rf wasm-output
