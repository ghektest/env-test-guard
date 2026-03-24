//! Temporary Git repository scaffolding for tests.
//!
//! Creates isolated, throwaway repositories with deterministic configuration.
//! Each [`GitRepo`] lives in a temp directory that is automatically cleaned
//! up when the value is dropped.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// An isolated temporary Git repository for testing.
///
/// On creation, initializes a fresh repo with deterministic author/committer
/// identity and timestamps. On drop, the temp directory is removed.
///
/// # Example
///
/// ```rust,no_run
/// use env_test_guard::GitRepo;
///
/// let repo = GitRepo::new().unwrap();
/// repo.write_file("src/main.rs", "fn main() {}");
/// repo.run_git(&["add", "."]);
/// repo.run_git(&["commit", "-m", "initial"]);
///
/// let log = repo.git_output(&["log", "--oneline"]);
/// assert!(log.contains("initial"));
/// ```
pub struct GitRepo {
    _temp_dir: tempfile::TempDir,
    root: PathBuf,
    branches: HashMap<String, PathBuf>,
    env_overrides: Vec<(String, String)>,
}

impl GitRepo {
    /// Create a new temporary git repository with isolated configuration.
    ///
    /// The repository is initialized with:
    /// - A deterministic author/committer identity
    /// - An initial empty commit on the `main` branch
    /// - Advice messages disabled
    /// - Isolated from global/system git config
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::TempDir::new()?;
        let root = temp_dir.path().join("repo");
        std::fs::create_dir(&root)?;

        let env_overrides = vec![
            ("GIT_AUTHOR_NAME".into(), "Test User".into()),
            ("GIT_AUTHOR_EMAIL".into(), "test@example.com".into()),
            ("GIT_COMMITTER_NAME".into(), "Test User".into()),
            ("GIT_COMMITTER_EMAIL".into(), "test@example.com".into()),
            ("GIT_AUTHOR_DATE".into(), "2025-01-01T00:00:00+00:00".into()),
            ("GIT_COMMITTER_DATE".into(), "2025-01-01T00:00:00+00:00".into()),
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("HOME".into(), temp_dir.path().to_string_lossy().into()),
        ];

        let repo = Self {
            _temp_dir: temp_dir,
            root,
            branches: HashMap::new(),
            env_overrides,
        };

        repo.run_git(&["init", "-b", "main", "-q"]);
        repo.run_git(&["config", "advice.mergeConflict", "false"]);
        repo.run_git(&["config", "advice.resolveConflict", "false"]);

        // Create initial commit so HEAD exists
        repo.run_git(&["commit", "--allow-empty", "-m", "initial commit"]);

        Ok(repo)
    }

    /// Path to the repository root.
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Write a file relative to the repo root, creating parent directories.
    pub fn write_file(&self, relative_path: &str, contents: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, contents).unwrap();
    }

    /// Read a file relative to the repo root.
    pub fn read_file(&self, relative_path: &str) -> String {
        std::fs::read_to_string(self.root.join(relative_path)).unwrap()
    }

    /// Stage a file and create a commit with the given message.
    pub fn commit(&self, message: &str, files: &[&str], content: &str) {
        for file in files {
            self.write_file(file, content);
        }
        self.run_git(&["add", "."]);
        self.run_git(&["commit", "-m", message]);
    }

    /// Create a new branch from the current HEAD.
    pub fn create_branch(&mut self, name: &str) {
        self.run_git(&["branch", name]);
        self.branches.insert(name.to_string(), self.root.clone());
    }

    /// Switch to an existing branch.
    pub fn checkout(&self, branch: &str) {
        self.run_git(&["checkout", branch, "-q"]);
    }

    /// Create a branch and switch to it.
    pub fn checkout_new_branch(&mut self, name: &str) {
        self.run_git(&["checkout", "-b", name, "-q"]);
        self.branches.insert(name.to_string(), self.root.clone());
    }

    /// Set up a bare remote and push the current branch to it.
    pub fn setup_remote(&self, branch: &str) -> PathBuf {
        let remote_path = self.root.parent().unwrap().join("remote.git");
        Command::new("git")
            .args(["init", "--bare", "-q"])
            .arg(&remote_path)
            .envs(self.env_overrides.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .output()
            .unwrap();

        self.run_git(&[
            "remote",
            "add",
            "origin",
            &remote_path.to_string_lossy(),
        ]);
        self.run_git(&["push", "-u", "origin", branch, "-q"]);

        remote_path
    }

    /// Run a git command in the repo context and assert success.
    pub fn run_git(&self, args: &[&str]) {
        let output = self.git_command(args).output().unwrap();
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "git {} failed: {}",
                args.join(" "),
                stderr
            );
        }
    }

    /// Run a git command and return its stdout as a string.
    pub fn git_output(&self, args: &[&str]) -> String {
        let output = self.git_command(args).output().unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    /// Build a git command configured for this repo's isolated environment.
    pub fn git_command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("git");
        cmd.args(args)
            .current_dir(&self.root)
            .envs(self.env_overrides.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        cmd
    }

    /// Return the current HEAD commit hash (short form).
    pub fn head_short(&self) -> String {
        self.git_output(&["rev-parse", "--short", "HEAD"])
    }

    /// Return the current branch name.
    pub fn current_branch(&self) -> String {
        self.git_output(&["branch", "--show-current"])
    }

    /// Return the list of all local branch names.
    pub fn branches(&self) -> Vec<String> {
        self.git_output(&["branch", "--format=%(refname:short)"])
            .lines()
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if the working tree has uncommitted changes.
    pub fn is_dirty(&self) -> bool {
        !self
            .git_output(&["status", "--porcelain"])
            .is_empty()
    }

    /// Add an environment variable override for all git commands in this repo.
    pub fn set_env(&mut self, key: &str, value: &str) {
        self.env_overrides.push((key.to_string(), value.to_string()));
    }
}
