//! Apex repository automation entry point.

use std::collections::HashSet;
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
        (Some("verify"), Some("targets")) if arguments.next().is_none() => verify_targets(),
        (Some("verify"), Some("quality")) if arguments.next().is_none() => verify_quality(),
        _ => Err("usage: cargo xtask verify <workspace|identifiers|targets|quality>".to_owned()),
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

const TARGET_MATRIX: &str = include_str!("../../docs/governance/target-matrix.txt");
const TARGET_MATRIX_SIZE: usize = 6;

fn verify_targets() -> Result<(), String> {
    let targets = target_matrix_entries()?;
    for target in &targets {
        verify_target(target)?;
    }
    println!("PASS: six Rust targets are recognized by the locked toolchain");
    Ok(())
}

fn target_matrix_entries() -> Result<Vec<&'static str>, String> {
    let targets: Vec<_> = TARGET_MATRIX
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    validate_target_matrix(&targets)?;
    Ok(targets)
}

fn validate_target_matrix(targets: &[&str]) -> Result<(), String> {
    let unique_targets: HashSet<_> = targets.iter().copied().collect();
    if unique_targets.len() != targets.len() {
        return Err("target matrix contains duplicate targets".to_owned());
    }
    if targets.len() != TARGET_MATRIX_SIZE {
        return Err(format!(
            "target matrix must contain exactly {TARGET_MATRIX_SIZE} targets, found {}",
            targets.len()
        ));
    }
    Ok(())
}

fn verify_target(target: &str) -> Result<(), String> {
    let output = Command::new("rustc")
        .args(["--target", target, "--print", "cfg"])
        .output()
        .map_err(|error| format!("failed to execute rustc for target {target}: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        if details.is_empty() {
            Err(format!(
                "Rust target dry-run failed for {target} with status {}",
                output.status
            ))
        } else {
            Err(format!(
                "Rust target dry-run failed for {target} with status {}: {details}",
                output.status
            ))
        }
    }
}

fn verify_quality() -> Result<(), String> {
    let root = workspace_root();
    run_quality_command(root, "fmt", &["fmt", "--all", "--", "--check"])?;
    run_quality_command(root, "check", &["check", "--workspace", "--all-targets"])?;
    run_quality_command(
        root,
        "clippy",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_quality_command(root, "test", &["test", "--workspace"])?;
    run_quality_command(root, "deny", &["deny", "--offline", "check"])?;
    println!("PASS: format, check, clippy, test, and dependency quality gates");
    Ok(())
}

fn run_quality_command(root: &Path, label: &str, arguments: &[&str]) -> Result<(), String> {
    let output = Command::new(env!("CARGO"))
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to execute {label} quality gate: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if details.is_empty() {
            format!("{label} quality gate failed with status {}", output.status)
        } else {
            format!(
                "{label} quality gate failed with status {}: {details}",
                output.status
            )
        })
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

    #[test]
    fn validates_six_target_matrix() {
        assert!(verify_targets().is_ok());
    }

    #[test]
    fn rejects_duplicate_target_matrix_entry() {
        let targets = ["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"];
        let result = validate_target_matrix(&targets);

        assert!(matches!(
            result,
            Err(error) if error.contains("duplicate targets")
        ));
    }

    #[test]
    fn rejects_unknown_target_with_actionable_error() {
        let result = verify_target("apex-unknown-target");

        assert!(matches!(
            result,
            Err(error)
                if error.contains("apex-unknown-target")
                    && error.contains("Rust target dry-run failed")
        ));
    }

    #[test]
    fn quality_gate_rejects_warning_fixture() {
        let fixture = workspace_root().join("xtask/tests/fixtures/warning");
        let result = run_quality_command(
            &fixture,
            "warning fixture",
            &["check", "--manifest-path", "Cargo.toml"],
        );

        assert!(matches!(
            result,
            Err(error)
                if error.contains("warning fixture quality gate failed")
                    && error.contains("unused variable")
        ));
    }
}
