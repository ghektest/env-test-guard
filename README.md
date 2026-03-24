# env-test-guard

RAII guard for environment variables in parallel Rust tests.

When running `cargo test`, tests execute in parallel by default. Since environment
variables are process-global, concurrent `set_var`/`remove_var` calls can cause
flaky tests. `EnvGuard` captures variable state on creation and restores it on drop.

## Usage

```rust
use env_test_guard::EnvGuard;

#[test]
fn test_with_custom_env() {
    let _guard = EnvGuard::new(&["DATABASE_URL", "LOG_LEVEL"]);

    std::env::set_var("DATABASE_URL", "postgres://test:test@localhost/test_db");
    std::env::set_var("LOG_LEVEL", "debug");

    // ... your test logic ...
    // Original values are restored when _guard drops
}
```

## License

MIT
