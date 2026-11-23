#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Validates a PR title (= squash commit message) against docs/GIT_WORKFLOW.md §2.
# Usage: PR_TITLE="feat(cli): add review" scripts/check-pr-title.sh
set -euo pipefail
title="${PR_TITLE:?PR_TITLE is not set}"
types='feat|fix|perf|refactor|test|docs|build|ci|chore|revert'
scopes='core|cli|web|mobile|ui|tokens|schemas|db|spec|research|ci|deps|release|repo'
re="^(${types})(\((${scopes})(,(${scopes}))*\))?!?: [^A-Z ].{2,}[^.]$"
if [[ "$title" =~ $re ]] && [ "${#title}" -le 100 ]; then
  echo "PR title ok: $title"
else
  echo "PR title doesn't follow docs/GIT_WORKFLOW.md §2:"
  echo "  $title"
  echo "Expected: <type>(<scope>): <lowercase summary, no full stop>, max 100 characters"
  echo "  types:  ${types//|/, }"
  echo "  scopes: ${scopes//|/, }"
  exit 1
fi
