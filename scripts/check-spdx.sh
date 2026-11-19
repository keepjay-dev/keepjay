#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Fails if a source file lacks an SPDX licence header in its first 3 lines (NFR-MAINT-006).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
missing=0
while IFS= read -r f; do
  if ! head -n 3 "$f" | grep -q "SPDX-License-Identifier:"; then
    echo "missing SPDX header: $f"
    missing=1
  fi
done < <(git ls-files --cached --others --exclude-standard \
  '*.rs' '*.ts' '*.tsx' '*.js' '*.mjs' '*.css' '*.sql' '*.sh' |
  grep -vE '(^|/)(node_modules|dist|target|gen)/')
if [ "$missing" -ne 0 ]; then
  echo "Add e.g. '// SPDX-License-Identifier: AGPL-3.0-or-later' to the files above."
  exit 1
fi
echo "SPDX headers: ok"
