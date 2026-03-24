# env-test-guard

Test toolkit for Rust projects that interact with Git.

Provides temporary repository scaffolding, environment variable guards, commit/branch fixture builders, and Git capability detection for consistent CI and local test runs.

## Features

- **`GitRepo`** — Isolated temporary Git repositories with deterministic config. Automatically cleaned up on drop.
- **`EnvGuard`** — RAII guard that saves/restores environment variables for safe parallel test execution.
- **`GitEnv`** — Detects Git version, capabilities (worktrees, sparse checkout), and CI provider.

## Usage

```rust
use env_test_guard::{EnvGuard, GitRepo, GitEnv};

#[test]
fn test_branch_operations() {
    let env = GitEnv::detect();
    assert!(env.supports_worktrees());

    let mut repo = GitRepo::new().unwrap();
    repo.commit("add feature", &["src/lib.rs"], "pub fn hello() {}");
    repo.checkout_new_branch("feature-x");
    repo.commit("feature work", &["src/lib.rs"], "pub fn hello() { todo!() }");

    assert_eq!(repo.current_branch(), "feature-x");
    assert!(!repo.is_dirty());
}

#[test]
fn test_with_env_isolation() {
    let _guard = EnvGuard::new(&["GIT_DIR", "GIT_WORK_TREE"]);
    std::env::set_var("GIT_DIR", "/tmp/fake");
    // GIT_DIR is restored when _guard drops
}
```

## License

MIT
