mod cli;
mod commands;
mod model;
mod output;
mod scanner;

use std::env;
use std::process;

use cli::{CliAction, ScanSource};

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    let args: Vec<_> = env::args_os().skip(1).collect();

    let (command, scan_source) = match cli::parse_action(&args) {
        CliAction::Execute {
            command,
            scan_source,
        } => (command, scan_source),
        CliAction::ShowUsage { message, exit_code } => {
            if let Some(line) = message {
                output::write_stderr([line]);
            }
            output::write_stderr(cli::usage_lines());
            return exit_code;
        }
    };

    let scan = match scan_source {
        ScanSource::Path => scanner::scan_path(),
        ScanSource::Homebrew => match scanner::scan_homebrew() {
            Ok(scan) => scan,
            Err(message) => {
                output::write_stderr([message]);
                return 1;
            }
        },
    };
    let result = commands::execute(&command, &scan);

    output::write_stdout(&result.stdout_lines);
    output::write_stderr(&result.stderr_lines);
    result.exit_code
}
