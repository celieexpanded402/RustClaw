#!/bin/bash
# Extract questions from benchmark_500.md into a format the runner can use
grep -E "^[0-9]+\. " tests/benchmark_500.md | sed 's/^[0-9]*\. //' > tests/questions_500.txt
wc -l tests/questions_500.txt
