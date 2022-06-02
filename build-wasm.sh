#!/bin/sh
PAT_TRIMMED="${PAT#"${PAT%%[![:space:]]*}"}" # remove leading whitespace
OUT_DIR="wasm-output"
TARGET_BRANCH=${TARGET_BRANCH##*/} # remove everything until the first '/'

wasm-pack build ${WASM_DIR} --target bundler --out-dir ${OUT_DIR}

echo ${TARGET_REPO}
echo ${TARGET_BRANCH}

cd ${WASM_DIR}/${OUT_DIR}
rm -f .gitignore
git init
git remote add origin https://${PAT_TRIMMED}@github.com/${TARGET_REPO}
git branch -M ${TARGET_BRANCH}
git add -A
git commit -m "Auto-generated wasm code"
git push -uf origin ${TARGET_BRANCH}

cd ..
rm -rf ${WASM_DIR}/${OUT_DIR}
