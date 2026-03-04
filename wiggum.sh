#!/usr/bin/env bash
#
# wiggum.sh — Continuously run kimi to complete PLAN.md phases.
#
# Usage:
#   ./wiggum.sh              # run until all phases are done
#   ./wiggum.sh --max 3      # run at most 3 iterations
#

set -euo pipefail

MAX_ITERATIONS=0 # 0 = unlimited
DELAY_SECONDS=2

while [[ $# -gt 0 ]]; do
  case "$1" in
    --max) MAX_ITERATIONS="$2"; shift 2 ;;
    --delay) DELAY_SECONDS="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

ITERATION=0

while true; do
  ITERATION=$((ITERATION + 1))

  # Check if all phases are complete (no unchecked items remain)
  if ! grep -q '^\- \[ \]' PLAN.md 2>/dev/null; then
    echo "✅ All phases complete! ($((ITERATION - 1)) iterations run)"
    break
  fi

  REMAINING=$(grep -c '^\- \[ \]' PLAN.md)
  echo "══════════════════════════════════════════════════════"
  echo "  🔄 Iteration $ITERATION — $REMAINING tasks remaining"
  echo "══════════════════════════════════════════════════════"

  kimi -p "start" -y --print

  if [[ $MAX_ITERATIONS -gt 0 && $ITERATION -ge $MAX_ITERATIONS ]]; then
    echo "⏹  Reached max iterations ($MAX_ITERATIONS). Stopping."
    break
  fi

  echo "⏳ Waiting ${DELAY_SECONDS}s before next iteration..."
  sleep "$DELAY_SECONDS"
done
