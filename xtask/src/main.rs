//! Apex repository automation entry point.

use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), String> {
    match (arguments.next().as_deref(), arguments.next().as_deref()) {
        (Some("verify"), Some("workspace")) if arguments.next().is_none() => {
            verify_workspace(workspace_root())
        }
        (Some("verify"), Some("identifiers")) if arguments.next().is_none() => {
            verify_identifiers(workspace_root())
        }
        _ => Err("usage: cargo xtask verify <workspace|identifiers>".to_owned()),
    }
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
}

fn verify_identifiers(root: &Path) -> Result<(), String> {
    let script = root.join("scripts/validate_identifier_registry.py");
    let command = if cfg!(windows) { "python" } else { "python3" };
    let status = Command::new(command)
        .arg(script)
        .current_dir(root)
        .status()
        .map_err(|error| format!("failed to execute identifier validator: {error}"))?;

    if status.success() {
        println!("PASS: identifier registry and source references are valid");
        Ok(())
    } else {
        Err(format!(
            "identifier registry validation failed with status {status}"
        ))
    }
}

fn verify_workspace(root: &Path) -> Result<(), String> {
    let manifest = root.join("Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(&manifest)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to execute cargo metadata: {error}"))?;

    if output.status.success() {
        println!("PASS: Cargo workspace members and paths are valid");
        Ok(())
    } else {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        if details.is_empty() {
            Err(format!(
                "Cargo workspace validation failed with status {}",
                output.status
            ))
        } else {
            Err(format!(
                "Cargo workspace validation failed with status {}: {details}",
                output.status
            ))
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn accepts_repository_workspace() {
        assert!(verify_workspace(workspace_root()).is_ok());
    }

    #[test]
    fn rejects_missing_workspace_member_with_actionable_error() {
        let fixture = workspace_root().join("xtask/tests/fixtures/missing-member");
        let result = verify_workspace(&fixture);

        assert!(matches!(
            result,
            Err(error)
                if error.contains("Cargo workspace validation failed")
                    && error.contains("missing-crate")
        ));
    }

    #[test]
    fn validates_identifier_registry() {
        assert!(verify_identifiers(workspace_root()).is_ok());
    }
}
