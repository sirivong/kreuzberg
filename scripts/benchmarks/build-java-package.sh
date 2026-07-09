#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

source "$REPO_ROOT/scripts/lib/common.sh"

validate_repo_root "$REPO_ROOT" || exit 1

cd "$REPO_ROOT/packages/java"

mvn -q -B -U package \
  -DskipTests \
  -Dcheckstyle.skip=true \
  -Dpmd.skip=true \
  -Djacoco.skip=true

mvn -q -B dependency:copy-dependencies \
  -DincludeScope=runtime \
  -DoutputDirectory=target/dependency

BENCH_SCRIPT="$REPO_ROOT/tools/benchmark-harness/scripts/XbergExtractJava.java"
if [ -f "$BENCH_SCRIPT" ]; then
  CP="target/classes"
  for jar in target/dependency/*.jar; do
    [ -f "$jar" ] && CP="$CP:$jar"
  done
  javac -cp "$CP" -d target/classes "$BENCH_SCRIPT"
fi
