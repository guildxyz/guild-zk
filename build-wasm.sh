#!/bin/sh

ORIGIN=
OUT_DIR="wasm-output"
TARGET_BRANCH=${TARGET_BRANCH##*/}

#wasm-pack build ${WASM_DIR} --target bundler --out-dir ${OUT_DIR}

echo ${TARGET_REPO}
echo ${TARGET_BRANCH}

cd ${WASM_DIR}/${OUT_DIR}
touch hellobello
rm -f .gitignore
git init
git add -A
git commit -m "Auto-generated wasm code"
git remote add origin https://${ACCESS_HEADER}github.com/${TARGET_REPO}
git branch -M ${TARGET_BRANCH}
git push -uf origin ${TARGET_BRANCH}

cd ..
rm -rf ${WASM_DIR}/${OUT_DIR}
