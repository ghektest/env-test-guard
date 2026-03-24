//! RAII guard for environment variables in parallel tests.
//!
//! Environment variables are process-global, so concurrent tests that modify
//! them can cause flaky failures. [`EnvGuard`] captures variable state on
//! creation and restores it on drop.

use std::collections::HashMap;

/// RAII guard that saves and restores environment variables.
///
/// On creation, captures the current values of the specified keys.
/// On drop, restores them to their original values (or removes them
/// if they were previously unset).
///
/// # Example
///
/// ```rust
/// use env_test_guard::EnvGuard;
///
/// let guard = EnvGuard::new(&["MY_VAR"]);
/// std::env::set_var("MY_VAR", "temporary");
/// drop(guard);
/// // MY_VAR is back to its original value (or removed if it was unset)
/// ```
pub struct EnvGuard {
    snapshot: HashMap<String, Option<String>>,
}

impl EnvGuard {
    /// Create a guard that will restore the given environment variables on drop.
    pub fn new(keys: &[&str]) -> Self {
        let mut snapshot = HashMap::new();
        for key in keys {
            snapshot.insert(key.to_string(), std::env::var(key).ok());
        }
        Self { snapshot }
    }

    /// Temporarily set an environment variable within this guard's scope.
    pub fn set(&self, key: &str, value: &str) {
        unsafe { std::env::set_var(key, value); }
    }

    /// Temporarily remove an environment variable within this guard's scope.
    pub fn remove(&self, key: &str) {
        unsafe { std::env::remove_var(key); }
    }

    /// Returns the number of variables being guarded.
    pub fn len(&self) -> usize {
        self.snapshot.len()
    }

    /// Returns true if no variables are being guarded.
    pub fn is_empty(&self) -> bool {
        self.snapshot.is_empty()
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.snapshot {
            match value {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }
}
