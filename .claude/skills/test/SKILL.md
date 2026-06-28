---
name: test
description: Run the ASI test suite with cargo test
disable-model-invocation: true
---

# Run Tests

Run the project's test suite.

## All tests
```bash
cargo test
```

## Single crate
```bash
cargo test -p asi-server
cargo test -p asi-lib
cargo test -p asi-ai-sdk
```

## Single test
```bash
cargo test -p asi-lib test_nanoid_length
cargo test -p asi-server --test api_integration
cargo test -p asi-server -- agent::tools::run_command
```

## With output
```bash
cargo test -- --nocapture
```

The project has 107 tests across 6 crates and integration test files.
