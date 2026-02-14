//! Test helpers for NovaType integration tests.

use std::path::{Path, PathBuf};

/// Get the workspace root directory.
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Get path to a test fixture file.
pub fn fixture_path(name: &str) -> PathBuf {
    workspace_root().join("tests").join("fixtures").join(name)
}

/// Check if Python is available on this system.
pub fn python_available() -> bool {
    let commands = if cfg!(windows) {
        vec!["python", "python3"]
    } else {
        vec!["python3", "python"]
    };

    commands.iter().any(|cmd| {
        std::process::Command::new(cmd)
            .args(["--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

/// Get the Python command that works on this system.
pub fn python_command() -> Option<String> {
    let commands = if cfg!(windows) {
        vec!["python", "python3"]
    } else {
        vec!["python3", "python"]
    };

    commands.into_iter().find_map(|cmd| {
        std::process::Command::new(cmd)
            .args(["--version"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|_| cmd.to_string())
    })
}

/// Check if the nova Python package and matplotlib are available.
pub fn nova_python_available() -> bool {
    if let Some(python) = python_command() {
        std::process::Command::new(&python)
            .args(["-c", "import nova; import matplotlib"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    }
}
