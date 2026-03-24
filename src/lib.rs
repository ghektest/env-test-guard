//! # env-test-guard
//!
//! A test toolkit for Rust projects that interact with Git. Provides temporary
//! repository scaffolding, environment variable guards, commit/branch fixture
//! builders, and Git capability detection for consistent CI/local test runs.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use env_test_guard::{EnvGuard, GitRepo, GitEnv};
//!
//! #[test]
//! fn test_with_isolated_repo() {
//!     let env = GitEnv::detect();
//!     assert!(env.version().is_some(), "git must be installed");
//!
//!     let repo = GitRepo::new().unwrap();
//!     repo.commit("initial commit", &["README.md"], "# Hello");
//!
//!     let _guard = EnvGuard::new(&["GIT_DIR", "GIT_WORK_TREE"]);
//!     // environment is restored when guard drops
//! }
//! ```

mod env_guard;
mod git_env;
mod git_repo;

pub use env_guard::EnvGuard;
pub use git_env::GitEnv;
pub use git_repo::GitRepo;
