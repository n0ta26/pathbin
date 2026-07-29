#![cfg(unix)]

use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const USAGE: &str = "\
Usage: pathbin <COMMAND>

Commands:
  list        List executable binaries in PATH
  where       Show where a command is located
  all         Show all matching binaries with the same name
  shadowed    Show binaries hidden by PATH priority
  duplicates  Show duplicate binary names
  broken      Show broken symlinks and missing PATH entries
  stats       Show PATH binary statistics
  doctor      Diagnose PATH-related problems
";

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    bin: PathBuf,
    tool: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "pathbin-cli-contract-{}-{sequence}",
            std::process::id()
        ));
        let bin = root.join("bin");
        fs::create_dir_all(&bin).expect("create PATH directory");
        let tool = bin.join("tool");
        fs::write(&tool, "#!/bin/sh\n").expect("create executable");
        let mut permissions = fs::metadata(&tool).expect("read permissions").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&tool, permissions).expect("mark executable");
        Self { root, bin, tool }
    }

    fn run(&self, arguments: &[&str]) -> Output {
        run_pathbin(Some(self.bin.as_os_str()), &self.root, arguments)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_pathbin(path: Option<&OsStr>, current_dir: &Path, arguments: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pathbin"));
    command.args(arguments).current_dir(current_dir);
    match path {
        Some(value) => {
            command.env("PATH", value);
        }
        None => {
            command.env_remove("PATH");
        }
    }
    command.output().expect("run pathbin")
}

fn assert_output(output: &Output, code: i32, stdout: &str, stderr: &str) {
    assert_eq!(output.status.code(), Some(code));
    assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
    assert_eq!(String::from_utf8_lossy(&output.stderr), stderr);
}

#[test]
fn every_public_subcommand_has_a_success_contract() {
    let fixture = Fixture::new();
    let tool = fixture.tool.display();

    assert_output(&fixture.run(&["list"]), 0, &format!("tool\t{tool}\n"), "");
    assert_output(
        &fixture.run(&["where", "tool"]),
        0,
        &format!("{tool}\n"),
        "",
    );
    assert_output(
        &fixture.run(&["all", "tool"]),
        0,
        &format!("[active] {tool}\n"),
        "",
    );
    assert_output(
        &fixture.run(&["shadowed"]),
        0,
        "No shadowed binaries found.\n",
        "",
    );
    assert_output(
        &fixture.run(&["duplicates"]),
        0,
        "No duplicate command names found.\n",
        "",
    );
    assert_output(
        &fixture.run(&["broken"]),
        0,
        "No broken PATH entries or symlinks found.\n",
        "",
    );

    let stats = fixture.run(&["stats"]);
    assert_eq!(stats.status.code(), Some(0));
    assert!(stats.stderr.is_empty());
    let stats_stdout = String::from_utf8_lossy(&stats.stdout);
    assert!(stats_stdout.contains("PATH entries: 1\n"));
    assert!(stats_stdout.contains("Executable binaries: 1\n"));
    assert!(stats_stdout.contains("Unique command names: 1\n"));

    assert_output(
        &fixture.run(&["doctor"]),
        0,
        "No obvious PATH problems detected.\n",
        "",
    );
}

#[test]
fn help_and_invalid_invocations_have_explicit_streams_and_exit_codes() {
    let fixture = Fixture::new();

    assert_output(&fixture.run(&["--help"]), 0, "", USAGE);
    assert_output(&fixture.run(&[]), 1, "", USAGE);
    assert_output(
        &fixture.run(&["unknown"]),
        1,
        "",
        &format!("Unknown command: unknown\n{USAGE}"),
    );
    assert_output(
        &fixture.run(&["list", "extra"]),
        1,
        "",
        &format!("Invalid arguments.\n{USAGE}"),
    );
    assert_output(
        &fixture.run(&["where"]),
        1,
        "",
        &format!("Invalid arguments.\n{USAGE}"),
    );
    assert_output(
        &fixture.run(&["all"]),
        1,
        "",
        &format!("Invalid arguments.\n{USAGE}"),
    );
}

#[test]
fn lookup_failures_use_stderr_and_exit_one() {
    let fixture = Fixture::new();

    assert_output(
        &fixture.run(&["where", "missing"]),
        1,
        "",
        "Command 'missing' was not found in PATH.\n",
    );
    assert_output(
        &fixture.run(&["all", "missing"]),
        1,
        "",
        "Command 'missing' was not found in PATH.\n",
    );
}
