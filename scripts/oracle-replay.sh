#!/usr/bin/env bash
# Replay oracle cases against target/release/logis with rust-anything (roadmap phase 1b).
set -euo pipefail

cd "$(dirname "$0")/.."

if [ ! -d oracle/cases ]; then
  echo "oracle-replay: no recorded cases yet; run 'mise run oracle:record' first." >&2
  exit 1
fi

echo "oracle-replay: not implemented yet; see .context/plans/oracle-corpus.md" >&2
exit 1
