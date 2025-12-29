use crate::error::{Error as SqlxError, Result};
use std::path::Path;
use std::process::Command;

/// Runs the given PHP script via the `php` CLI and returns an error if it fails.
#[allow(dead_code)]
fn run_php_file(php_file: &Path) -> Result<String> {
    // Spawn `php -f <script_name>`
    let output = Command::new("php")
        .arg("-f")
        .arg(php_file)
        .output()
        .map_err(|err| SqlxError::Other(format!("Failed to execute php on {php_file:?}: {err}")))?;

    // Print PHP stdout for debugging
    println!(
        "--- PHP stdout ---\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    // If PHP wrote to stderr, print that too
    if !output.stderr.is_empty() {
        eprintln!(
            "--- PHP stderr ---\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Check exit status
    if !output.status.success() {
        return Err(SqlxError::Other(format!(
            "PHP script {php_file:?} exited with code {}",
            output.status.code().unwrap_or(-1)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[allow(dead_code)]
fn run_php_example(name: &str) -> Result<String> {
    run_php_file(
        &Path::new(
            &std::env::var("CARGO_MANIFEST_DIR")
                .map_err(|e| SqlxError::Other(format!("env CARGO_MANIFEST_DIR: {e}")))?,
        )
        .join(format!("examples/{name}.php")),
    )
}

#[allow(dead_code)]
fn run_php_test(name: &str) -> Result<String> {
    run_php_file(
        &Path::new(
            &std::env::var("CARGO_MANIFEST_DIR")
                .map_err(|e| SqlxError::Other(format!("env CARGO_MANIFEST_DIR: {e}")))?,
        )
        .join(format!("tests/{name}.php")),
    )
}
