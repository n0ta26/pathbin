#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    ambient_bin: PathBuf,
    prefix: PathBuf,
    homebrew_bin: PathBuf,
    homebrew_sbin: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "pathbin-homebrew-source-{}-{sequence}",
            std::process::id()
        ));
        let ambient_bin = root.join("ambient/bin");
        let prefix = root.join("homebrew");
        let homebrew_bin = prefix.join("bin");
        let homebrew_sbin = prefix.join("sbin");
        for directory in [&ambient_bin, &homebrew_bin, &homebrew_sbin] {
            fs::create_dir_all(directory).expect("create test directory");
        }

        Self {
            root,
            ambient_bin,
            prefix,
            homebrew_bin,
            homebrew_sbin,
        }
    }

    fn run(&self, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_pathbin"))
            .args(arguments)
            .current_dir(&self.root)
            .env("PATH", &self.ambient_bin)
            .env("HOMEBREW_PREFIX", &self.prefix)
            .output()
            .expect("run pathbin")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn create_executable(directory: &Path, name: &str) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, "#!/bin/sh\n").expect("create executable");
    let mut permissions = fs::metadata(&path).expect("read permissions").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("mark executable");
    path
}

fn assert_success(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

#[test]
fn scans_only_homebrew_bin_and_sbin_in_resolution_order() {
    let fixture = Fixture::new();
    create_executable(&fixture.ambient_bin, "ambient-only");
    let bin_tool = create_executable(&fixture.homebrew_bin, "tool");
    let sbin_tool = create_executable(&fixture.homebrew_sbin, "tool");
    let admin = create_executable(&fixture.homebrew_sbin, "admin");

    let list = assert_success(&fixture.run(&["--homebrew", "list"]));
    assert!(!list.contains("ambient-only"));
    assert!(list.contains(&format!("tool\t{}\n", bin_tool.display())));
    assert!(list.contains(&format!("admin\t{}\n", admin.display())));

    assert_eq!(
        assert_success(&fixture.run(&["all", "tool", "--homebrew"])),
        format!(
            "[active] {}\n[shadowed] {}\n",
            bin_tool.display(),
            sbin_tool.display()
        )
    );

    let stats = assert_success(&fixture.run(&["stats", "--homebrew"]));
    assert!(stats.contains("PATH entries: 2\n"));
    assert!(stats.contains("Existing directories: 2\n"));
    assert!(stats.contains("Executable binaries: 3\n"));
}

#[test]
fn homebrew_option_is_available_to_every_subcommand_in_either_position() {
    let fixture = Fixture::new();
    create_executable(&fixture.homebrew_bin, "tool");

    let invocations: &[&[&str]] = &[
        &["--homebrew", "list"],
        &["where", "tool", "--homebrew"],
        &["--homebrew", "all", "tool"],
        &["shadowed", "--homebrew"],
        &["--homebrew", "duplicates"],
        &["broken", "--homebrew"],
        &["--homebrew", "stats"],
        &["doctor", "--homebrew"],
    ];

    for arguments in invocations {
        let output = fixture.run(arguments);
        assert_eq!(
            output.status.code(),
            Some(0),
            "invocation failed: {arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn rejects_a_relative_homebrew_prefix() {
    let fixture = Fixture::new();
    let output = Command::new(env!("CARGO_BIN_EXE_pathbin"))
        .args(["--homebrew", "list"])
        .current_dir(&fixture.root)
        .env("PATH", &fixture.ambient_bin)
        .env("HOMEBREW_PREFIX", "relative-prefix")
        .output()
        .expect("run pathbin");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "HOMEBREW_PREFIX must be an absolute path.\n"
    );
}
