#!/bin/bash
# This script runs fuzz targets using cargo-fuzz (libFuzzer)
# It first generates targets, then creates an initial corpus, and finally runs the fuzzers.

set -e  # Exit immediately if a command exits with a non-zero status.
set -x  # Print commands and their arguments as they are executed.

# Ensure fuzz targets are generated
echo "Generating fuzz targets..."
pushd src/msg_targets
rm -f msg_*.rs # Use -f to ignore non-existent files
./gen_target.sh
# Check if gen_target.sh created/modified files and error if so (optional, depends on workflow)
# [ "$(git diff --exit-code)" != "" ] && { echo "msg_targets generated files not committed"; exit 1; }
popd
pushd src/bin
rm -f *_target.rs # Use -f to ignore non-existent files
./gen_target.sh
# Check if gen_target.sh created/modified files and error if so (optional, depends on workflow)
# [ "$(git diff --exit-code)" != "" ] && { echo "bin targets generated files not committed"; exit 1; }
popd
echo "Fuzz targets generated."

# Install cargo-fuzz (requires nightly toolchain implicitly later)
echo "Installing cargo-fuzz..."
cargo install --force cargo-fuzz
echo "cargo-fuzz installed."

# Remove LTO if present (might interfere with fuzzing instrumentation)
# Use sed -i.bak for macOS compatibility
if grep -q 'lto = true' Cargo.toml; then
    echo "Removing LTO setting from Cargo.toml..."
    sed -i.bak 's/lto = true//' Cargo.toml
    rm -f Cargo.toml.bak # Clean up backup file
    echo "LTO setting removed."
else
    echo "LTO setting not found in Cargo.toml."
fi


# Set RUSTFLAGS required for fuzzing
export RUSTFLAGS="--cfg=fuzzing --cfg=secp256k1_fuzz --cfg=hashes_fuzz"
echo "RUSTFLAGS set to: $RUSTFLAGS"

CORPUS_DIR="corpus"

# --- Fuzz Execution ---
echo "Starting fuzz execution..."

# Iterate over target binaries again to run fuzzing
for TARGET_PATH in src/bin/*_target.rs; do
    FILENAME=$(basename "$TARGET_PATH")
    TARGET_NAME="${FILENAME%.*}" # Remove .rs extension
    TARGET_CORPUS_DIR="$CORPUS_DIR/$TARGET_NAME"

    echo "Running fuzzer for target: $TARGET_NAME using corpus $TARGET_CORPUS_DIR"

    # Set time limit for each fuzzer run (e.g., 60 seconds)
    # Adjust MAX_TOTAL_TIME as needed. cargo fuzz passes arguments after '--' to libFuzzer.
    # Use runs (-runs=N) instead or in addition if preferred.
    MAX_TOTAL_TIME=60
    # MAX_RUNS=1000000 # Example if using runs instead of time

    # Run the fuzzer using the generated corpus directory
    cargo +nightly fuzz run "$TARGET_NAME" --features "libfuzzer_fuzz" -- \
        "$TARGET_CORPUS_DIR" \
        -max_total_time="$MAX_TOTAL_TIME"
        # -runs="$MAX_RUNS" # Uncomment if using runs limit

    # cargo fuzz run will exit with non-zero status on crash due to set -e
    echo "Fuzzing finished for target: $TARGET_NAME."
done

echo "All fuzz targets executed successfully." 