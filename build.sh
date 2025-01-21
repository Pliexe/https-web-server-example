#!/bin/bash
set -e

echo "Starting building of project"
echo

cargo build --release

echo "Done."
echo