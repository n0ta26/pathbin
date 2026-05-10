mod cli;
mod commands;
mod model;
mod output;
mod scanner;

use std::env;
use std::process;

use cli::CliAction;

fn main() {
    process::exit(run());
}

fn run() -> i32 {
    let args: Vec<String> = env::args().skip(1).collect();

    let command = match cli::parse_action(&args) {
        CliAction::Execute(command) => command,
        CliAction::ShowUsage { message, exit_code } => {
            if let Some(line) = message {
                output::write_stderr([line]);
            }
            output::write_stderr(cli::usage_lines());
            return exit_code;
        }
    };

    let scan = scanner::scan_path();
    let result = commands::execute(&command, &scan);

    output::write_stdout(&result.stdout_lines);
    output::write_stderr(&result.stderr_lines);
    result.exit_code
}
