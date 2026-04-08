#!/bin/bash
# RustClaw 500-Question Tool Calling Benchmark
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"

MODEL=${1:-"unknown"}
BIN="./target/release/rustclaw"
RESULTS_DIR="tests/results"
TS=$(date +%Y%m%d_%H%M%S)
OUT="$RESULTS_DIR/bench500_${MODEL}_${TS}.txt"
QUESTIONS="tests/questions_500.txt"

mkdir -p "$RESULTS_DIR"

log() { echo "$1" | tee -a "$OUT"; }

log "=== RustClaw 500-Q Benchmark ==="
log "Model: $MODEL | $(date)"
log ""

PASS=0
FAIL=0
TOTAL=0
TIMEOUT_COUNT=0
START_TIME=$(date +%s)

while IFS= read -r question; do
  TOTAL=$((TOTAL + 1))

  # Category based on question number
  if [ $TOTAL -le 50 ]; then CAT="A-Daily"
  elif [ $TOTAL -le 130 ]; then CAT="B-Basic"
  elif [ $TOTAL -le 230 ]; then CAT="C-Medium"
  elif [ $TOTAL -le 330 ]; then CAT="D-Advanced"
  elif [ $TOTAL -le 400 ]; then CAT="E-Hallucination"
  else CAT="F-MultiStep"
  fi

  # macOS doesn't have timeout; use perl fallback
  resp=$(perl -e 'alarm 90; exec @ARGV' $BIN agent "$question" 2>&1) || {
    resp=$(echo "$resp")
    if [ -z "$resp" ]; then
      resp="[TIMEOUT or ERROR]"
      TIMEOUT_COUNT=$((TIMEOUT_COUNT + 1))
    fi
  }

  len=${#resp}
  status="❌"

  if [ "$len" -gt 30 ]; then
    status="✅"
    PASS=$((PASS + 1))
  elif echo "$resp" | grep -qiE "不能|拒絕|危險|無法執行|cannot|refuse|won't|shouldn't|not allowed"; then
    status="✅"
    PASS=$((PASS + 1))
  else
    FAIL=$((FAIL + 1))
  fi

  short=$(echo "$resp" | head -1 | cut -c1-80)
  log "Q${TOTAL} [${CAT}] ${status} (${len}c) ${question}"

  # Progress every 50 questions
  if [ $((TOTAL % 50)) -eq 0 ]; then
    ELAPSED=$(( $(date +%s) - START_TIME ))
    log "--- Progress: ${TOTAL}/500 | Pass: ${PASS} | Elapsed: ${ELAPSED}s ---"
  fi
done < "$QUESTIONS"

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

log ""
log "========================================"
log "TOTAL: $PASS/$TOTAL ($((PASS * 100 / TOTAL))%)"
log "  ✅ Pass: $PASS"
log "  ❌ Fail: $FAIL"
log "  ⏱ Timeout: $TIMEOUT_COUNT"
log "  Time: ${ELAPSED}s (~$((ELAPSED / 60))min)"
log "  Avg: $((ELAPSED / TOTAL))s/question"
log ""
log "By Category:"
# Count per category from the output
for cat in "A-Daily" "B-Basic" "C-Medium" "D-Advanced" "E-Hallucination" "F-MultiStep"; do
  cat_total=$(grep -c "\[$cat\]" "$OUT" || true)
  cat_pass=$(grep "\[$cat\] ✅" "$OUT" | wc -l | tr -d ' ')
  if [ "$cat_total" -gt 0 ]; then
    log "  $cat: $cat_pass/$cat_total ($((cat_pass * 100 / cat_total))%)"
  fi
done
log "========================================"

# Cleanup test artifacts
rm -f test.txt hello.txt hello.py hello.rs hello test.rs test.sh benchmark.py notes.md
rm -f stats.txt TODO.md ARCHITECTURE.md CHANGELOG.md .editorconfig justfile Makefile
rm -f deploy.sh healthcheck.sh backup.sh smoke_test.sh recovery.sh
rm -f project.tar project.tar.gz rustclaw.tar.gz
rm -rf tmp/ scripts/ data/ utils/ examples/ bench/ .devcontainer/
rm -f src/main.rs.bak src/tools/weather.rs .env
rm -f docker-compose.yml .github/workflows/ci.yml .github/ISSUE_TEMPLATE.md
rm -f CONTRIBUTING.md contributing.md
git checkout -- README.md Cargo.toml .gitignore 2>/dev/null || true

echo "Results: $OUT"
