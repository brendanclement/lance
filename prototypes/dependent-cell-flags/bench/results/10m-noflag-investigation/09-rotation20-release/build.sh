#!/bin/bash
set -euo pipefail
S=/private/tmp/claude-501/-Users-brendan-code-lance--claude-worktrees-dependency-aware-cell-flags-43fc32/74020bc7-5e3b-4fa2-a700-40b87904a7b1/scratchpad
MEMBERS="lance-examples lance lance-arrow lance-bitpacking lance-core lance-derive lance-datafusion lance-datagen lance-testing lance-geo lance-encoding fsst lance-file lance-io lance-namespace lance-index lance-arrow-stats lance-arrow-scalar lance-index-core lance-select lance-linalg lance-table lance-tokenizer lance-test-macros lance-namespace-impls lance-namespace-datafusion lance-tools"
build() {  # checkout name
  echo "$(date +%T) build $2 in $1"
  (cd "$S/$1" && cargo bench -p lance --bench cell_flags_scan_counters --profile release --no-run 2>&1 | tail -2)
  local bin
  bin=$(find "$S/$1/target/release/deps" -maxdepth 1 -type f -perm -u+x -name 'cell_flags_scan_counters-*' ! -name '*.d' | head -1)
  mkdir -p "$S/bins-release/$2/target/release/deps"
  cp -p "$bin" "$S/bins-release/$2/target/release/deps/"
  echo "$(date +%T) saved $2 $(md5 -q "$bin") $(basename "$bin")"
}
build regr-base baseline
for d in regr-proto regr-fix; do
  mkdir -p "$S/$d/target"; cp -Rc "$S/regr-base/target/release" "$S/$d/target/release"
  (cd "$S/$d" && cargo clean --profile release $(for m in $MEMBERS; do printf -- '-p %s ' "$m"; done) 2>&1 | tail -1)
done
git -C "$S/regr-base" apply "$S/probe-field.patch"
build regr-base base-field
git -C "$S/regr-base" checkout -- rust/lance/src/dataset/fragment.rs
build regr-proto prototype
build regr-fix fix
df -h "$S" | tail -1
echo "all release builds done"
