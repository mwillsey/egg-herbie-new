#!/usr/bin/env bash
set -euo pipefail

for input in inputs/*; do
    echo Running $input...
    time (cat rules.json $input | cargo run --release --quiet > /dev/null)
done
