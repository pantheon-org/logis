#!/usr/bin/env bash
# Record chezmoi behaviour into oracle/cases/<os>/ with rust-anything (roadmap phase 1b).
set -euo pipefail

cd "$(dirname "$0")/.."

oracle="$(mise which chezmoi)"
inputs="oracle/inputs"

if ! compgen -G "$inputs/*.jsonl" >/dev/null; then
  echo "oracle-record: no inputs in $inputs/ yet." >&2
  echo "Generate them from the chezmoi txtar scripts first (scripts/txtar2inputs, phase 1b)." >&2
  exit 1
fi

echo "oracle-record: recording with $oracle" >&2
echo "oracle-record: not implemented yet; see .context/plans/oracle-corpus.md" >&2
exit 1
