use crate::cli::Command;
use crate::model::ScanResult;

#[derive(Debug, Default)]
pub struct CommandResult {
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub exit_code: i32,
}

impl CommandResult {
    fn success(stdout_lines: Vec<String>) -> Self {
        Self {
            stdout_lines,
            stderr_lines: Vec::new(),
            exit_code: 0,
        }
    }

    fn failure(stderr_line: String) -> Self {
        Self {
            stdout_lines: Vec::new(),
            stderr_lines: vec![stderr_line],
            exit_code: 1,
        }
    }
}

pub fn execute(command: &Command, scan: &ScanResult) -> CommandResult {
    match command {
        Command::List => list_binaries(scan),
        Command::Where { name } => where_command(scan, name),
        Command::All { name } => all_commands(scan, name),
        Command::Shadowed => shadowed(scan),
        Command::Duplicates => duplicates(scan),
        Command::Broken => broken(scan),
        Command::Stats => stats(scan),
        Command::Doctor => doctor(scan),
    }
}

fn list_binaries(scan: &ScanResult) -> CommandResult {
    if scan.binaries().is_empty() {
        return CommandResult::success(vec!["No executable binaries found in PATH.".to_string()]);
    }

    let lines = scan
        .binaries()
        .iter()
        .map(|entry| format!("{}\t{}", entry.name(), entry.path().display()))
        .collect();
    CommandResult::success(lines)
}

fn where_command(scan: &ScanResult, command_name: &str) -> CommandResult {
    if let Some(matches) = scan.command_matches(command_name) {
        if let Some(first) = matches.first() {
            return CommandResult::success(vec![first.path().display().to_string()]);
        }
    }

    command_not_found(command_name)
}

fn all_commands(scan: &ScanResult, command_name: &str) -> CommandResult {
    if let Some(matches) = scan.command_matches(command_name) {
        let lines = matches
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let status = if index == 0 { "active" } else { "shadowed" };
                format!("[{status}] {}", entry.path().display())
            })
            .collect();
        return CommandResult::success(lines);
    }

    command_not_found(command_name)
}

fn shadowed(scan: &ScanResult) -> CommandResult {
    let mut lines = Vec::new();

    for (name, entries) in scan.duplicate_groups() {
        lines.push(name.to_string());
        for entry in entries.iter().skip(1) {
            lines.push(format!("  {}", entry.path().display()));
        }
    }

    if lines.is_empty() {
        lines.push("No shadowed binaries found.".to_string());
    }

    CommandResult::success(lines)
}

fn duplicates(scan: &ScanResult) -> CommandResult {
    let mut lines = Vec::new();

    for (name, entries) in scan.duplicate_groups() {
        lines.push(format!("{name}\t{}", entries.len()));
    }

    if lines.is_empty() {
        lines.push("No duplicate command names found.".to_string());
    }

    CommandResult::success(lines)
}

fn broken(scan: &ScanResult) -> CommandResult {
    let mut lines = Vec::new();

    if !scan.missing_entries().is_empty() {
        lines.push("Missing PATH entries:".to_string());
        for entry in scan.missing_entries() {
            lines.push(format!("  {}", entry.display()));
        }
    }

    if !scan.non_dir_entries().is_empty() {
        lines.push("Non-directory PATH entries:".to_string());
        for entry in scan.non_dir_entries() {
            lines.push(format!("  {}", entry.display()));
        }
    }

    if !scan.unreadable_entries().is_empty() {
        lines.push("Unreadable PATH directories:".to_string());
        for entry in scan.unreadable_entries() {
            lines.push(format!("  {}", entry.display()));
        }
    }

    if !scan.broken_symlinks().is_empty() {
        lines.push("Broken symlinks:".to_string());
        for link in scan.broken_symlinks() {
            lines.push(format!("  {}", link.display()));
        }
    }

    if lines.is_empty() {
        lines.push("No broken PATH entries or symlinks found.".to_string());
    }

    CommandResult::success(lines)
}

fn stats(scan: &ScanResult) -> CommandResult {
    let lines = vec![
        format!("PATH entries: {}", scan.path_entries_total()),
        format!("Existing directories: {}", scan.existing_dirs()),
        format!("Missing PATH entries: {}", scan.missing_entries().len()),
        format!("Non-directory entries: {}", scan.non_dir_entries().len()),
        format!(
            "Unreadable PATH directories: {}",
            scan.unreadable_entries().len()
        ),
        format!("Empty PATH entries: {}", scan.empty_path_entries()),
        format!("Executable binaries: {}", scan.binaries().len()),
        format!("Unique command names: {}", scan.unique_command_count()),
        format!("Duplicate command names: {}", scan.duplicate_name_count()),
        format!("Shadowed binaries: {}", scan.shadowed_binary_count()),
        format!("Broken symlinks: {}", scan.broken_symlinks().len()),
    ];

    CommandResult::success(lines)
}

fn doctor(scan: &ScanResult) -> CommandResult {
    let mut lines = Vec::new();
    let mut findings = 0usize;
    let mut has_warning_or_error = false;

    if scan.path_entries_total() == 0 {
        findings += 1;
        has_warning_or_error = true;
        lines.push("[ERROR] PATH is empty or not set.".to_string());
    }
    if scan.empty_path_entries() > 0 {
        findings += 1;
        has_warning_or_error = true;
        lines.push(format!(
            "[WARN] PATH contains {} empty entry/entries (current directory lookup).",
            scan.empty_path_entries()
        ));
    }
    if !scan.missing_entries().is_empty() {
        findings += 1;
        has_warning_or_error = true;
        lines.push(format!(
            "[WARN] PATH contains {} missing directory/directories.",
            scan.missing_entries().len()
        ));
    }
    if !scan.non_dir_entries().is_empty() {
        findings += 1;
        has_warning_or_error = true;
        lines.push(format!(
            "[WARN] PATH contains {} non-directory entry/entries.",
            scan.non_dir_entries().len()
        ));
    }
    if !scan.unreadable_entries().is_empty() {
        findings += 1;
        has_warning_or_error = true;
        lines.push(format!(
            "[WARN] PATH contains {} unreadable directory/directories.",
            scan.unreadable_entries().len()
        ));
    }
    if !scan.broken_symlinks().is_empty() {
        findings += 1;
        has_warning_or_error = true;
        lines.push(format!(
            "[WARN] Found {} broken symlink(s) in PATH directories.",
            scan.broken_symlinks().len()
        ));
    }
    if scan.duplicate_name_count() > 0 {
        findings += 1;
        lines.push(format!(
            "[INFO] Found {} duplicate command name(s).",
            scan.duplicate_name_count()
        ));
    }

    if findings == 0 {
        lines.push("No obvious PATH problems detected.".to_string());
        return CommandResult::success(lines);
    }

    lines.push(format!(
        "Doctor summary: {findings} issue category/categories detected."
    ));
    CommandResult {
        stdout_lines: lines,
        stderr_lines: Vec::new(),
        exit_code: i32::from(has_warning_or_error),
    }
}

fn command_not_found(command_name: &str) -> CommandResult {
    CommandResult::failure(format!("Command '{command_name}' was not found in PATH."))
}
