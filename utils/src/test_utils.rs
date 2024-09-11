use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    mem,
    sync::{Mutex, MutexGuard, PoisonError},
};

pub struct EnvMutex(Mutex<()>);

impl Default for EnvMutex {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvMutex {
    /// Creates a new mutex. Separate mutexes can be used for changing env vars that do not intersect
    /// (e.g., env vars for different config).
    pub const fn new() -> Self {
        Self(Mutex::new(()))
    }

    pub fn lock(&self) -> EnvMutexGuard<'_> {
        let guard = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        EnvMutexGuard {
            _inner: guard,
            redefined_vars: HashMap::new(),
        }
    }
}

/// Guard provided by [`EnvMutex`] that allows mutating env variables. All changes are rolled back
/// when the guard is dropped.
#[must_use = "Environment will be reset when the guard is dropped"]
#[derive(Debug)]
pub struct EnvMutexGuard<'a> {
    _inner: MutexGuard<'a, ()>,
    redefined_vars: HashMap<OsString, Option<OsString>>,
}

impl Drop for EnvMutexGuard<'_> {
    fn drop(&mut self) {
        for (env_name, value) in mem::take(&mut self.redefined_vars) {
            if let Some(value) = value {
                unsafe {
                    env::set_var(env_name, value);
                }
            } else {
                unsafe {
                    env::remove_var(env_name);
                }
            }
        }
    }
}

impl EnvMutexGuard<'_> {
    /// Sets env vars specified in `.env`-like format.
    pub unsafe fn set_env(&mut self, fixture: &str) {
        for line in fixture.split('\n').map(str::trim) {
            if line.is_empty() {
                // Skip empty lines.
                continue;
            }

            let (variable_name, variable_value) = line.split_once('=').unwrap_or_else(|| {
                panic!("Incorrect line for setting environment variable: {}", line);
            });
            let variable_name: &OsStr = variable_name.as_ref();
            let variable_value: &OsStr = variable_value.trim_matches('"').as_ref();

            if !self.redefined_vars.contains_key(variable_name) {
                let prev_value = env::var_os(variable_name);
                self.redefined_vars
                    .insert(variable_name.to_os_string(), prev_value);
            }
            env::set_var(variable_name, variable_value);
        }
    }
}
