#!/usr/bin/env bash
# The three-way acceptance control for the checker itself.
#
# A checker that works and a checker that blocks everything are indistinguishable
# from a passing control A alone, and one that blocks nothing is indistinguishable
# from a passing control B alone. Only running all three separates them:
#
#   A  the over-claim               MUST BLOCK   exit 1, and by >= 5 distinct lenses
#   B  the bounded conclusion       MUST PASS    exit 0, and the packet must freeze
#   C  an absent vault              MUST be DISTINGUISHABLE from both, exit 2
#
# C is the one most often skipped, and it is the one that catches a checker whose
# "clean" verdict is really "I never read anything".
#
# Assertions are on EXIT CODES and a gate count, never on log text: log text
# changes with wording, and a gate that stops firing must not be able to hide
# behind a reworded message.
#
# Lives here rather than inline in ci.yml so the pre-commit hook and CI run the
# same bytes. Duplicated control logic drifts, and a drifted control is worse
# than an absent one because it still reports green.
#
# Usage: tests/controls.sh [path-to-peira-binary]
set -euo pipefail

BIN="${1:-target/debug/peira}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [ ! -x "$BIN" ]; then
  echo "error: peira binary not found at '$BIN' — build it first" >&2
  exit 127
fi

fail() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    echo "::error::$1"
  else
    echo "error: $1" >&2
  fi
  exit 1
}

# --- Control A — the over-claim must BLOCK -----------------------------------
set +e
"$BIN" gates tests/vaults/overclaim
code=$?
set -e
[ "$code" -eq 1 ] || fail "control A must exit 1 (violations); got $code"

# `pipefail` is off inside the substitution on purpose: control A's whole point is
# that `gates` exits 1, so with pipefail the pipeline would inherit that 1 and
# `set -e` would kill the script before the count is ever compared.
n=$(set +o pipefail; "$BIN" gates tests/vaults/overclaim | grep -c 'PEIR-')
[ "$n" -ge 5 ] || fail "control A must be blocked by at least 5 lenses; got $n"
echo "control A: blocked by $n gates, exit $code"

# --- Control B — the bounded conclusion must PASS ----------------------------
"$BIN" gates tests/vaults/bounded
"$BIN" lint tests/vaults/bounded
"$BIN" packet tests/vaults/bounded c-bounded > /dev/null
echo "control B: passes, packet freezes"

# --- Control C — an absent vault must be distinguishable from a clean one ----
set +e
"$BIN" gates ./definitely-not-a-vault
code=$?
set -e
[ "$code" -eq 2 ] || fail "an absent vault must exit 2, distinct from A's 1; got $code"
echo "control C: absent vault exits 2, distinct from both A and B"
