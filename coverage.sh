# Generates code coverage report at ./target/debug/coverage
export RUSTFLAGS="-Zinstrument-coverage"
export LLVM_PROFILE_FILE="%p-%m.profraw"
cargo test
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/ \
  --ignore "/*" \
  --ignore "**/testing/*" \
  --ignore "**/schema.rs" \
  --ignore "**/mock_*.rs"
rm -rf ./**/*.profraw