#!/usr/bin/env bash
set -euo pipefail

# Generate code coverage reports for the from-zero-to-systems workspace.
#
# Prerequisites: cargo install cargo-llvm-cov
#
# Usage:
#   ./scripts/coverage.sh          # run all coverage, generate HTML report
#   ./scripts/coverage.sh --text   # print text summary to stdout
#   ./scripts/coverage.sh --open   # generate HTML and open in browser

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MODE="${1:-html}"
COVERAGE_DIR="$ROOT/target/llvm-cov"

case "$MODE" in
  --text)
    echo "=== Rust coverage (play crate) ==="
    cargo llvm-cov test -p play --summary-only
    echo ""
    echo "=== Rust coverage (risk-sampler) ==="
    cargo llvm-cov test -p risk-sampler --summary-only
    ;;
  --open)
    cargo llvm-cov test -p play -p risk-sampler --html --output-dir "$COVERAGE_DIR/html"
    echo "Coverage report: $COVERAGE_DIR/html/index.html"
    xdg-open "$COVERAGE_DIR/html/index.html" 2>/dev/null || open "$COVERAGE_DIR/html/index.html" 2>/dev/null || echo "Open manually: $COVERAGE_DIR/html/index.html"
    ;;
  *)
    cargo llvm-cov test -p play -p risk-sampler --html --output-dir "$COVERAGE_DIR/html"
    echo ""
    echo "=== Summary ==="
    cargo llvm-cov test -p play -p risk-sampler --summary-only 2>&1 | tail -3
    echo ""
    echo "HTML report: $COVERAGE_DIR/html/index.html"
    ;;
esac

# Python coverage (if pytest-cov is installed)
if python3 -c "import pytest_cov" 2>/dev/null; then
  echo ""
  echo "=== Python coverage (risk-sampler) ==="
  cd "$ROOT/crates/01-risk-sampler/python"
  python3 -m pytest test_risk_sampler.py --cov=risk_sampler --cov-report=term-missing -q
fi

# C coverage (if gcov is available)
if command -v gcov &>/dev/null; then
  echo ""
  echo "=== C coverage (risk-sampler) ==="
  cd "$ROOT/crates/01-risk-sampler/c"
  make clean >/dev/null 2>&1 || true
  gcc -Wall -Wextra -std=c11 --coverage -c risk_sampler.c -o risk_sampler.o 2>/dev/null
  gcc -Wall -Wextra -std=c11 --coverage -c test_risk_sampler.c -o test_risk_sampler.o 2>/dev/null
  gcc --coverage -o test_cov risk_sampler.o test_risk_sampler.o -lm
  ./test_cov >/dev/null 2>&1
  gcov risk_sampler.c 2>/dev/null | grep -E "^(File|Lines)" || true
  rm -f test_cov *.o *.gcda *.gcno *.gcov 2>/dev/null || true
fi
