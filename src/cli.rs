use std::ffi::OsString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    List,
    Where { name: OsString },
    All { name: OsString },
    Shadowed,
    Duplicates,
    Broken,
    Stats,
    Doctor,
}

#[derive(Debug, Clone)]
pub enum CliAction {
    Execute(Command),
    ShowUsage {
        message: Option<String>,
        exit_code: i32,
    },
}

pub fn parse_action(args: &[OsString]) -> CliAction {
    if args.is_empty() {
        return CliAction::ShowUsage {
            message: None,
            exit_code: 1,
        };
    }

    let command = args[0].to_str();
    if matches!(command, Some("-h" | "--help" | "help")) {
        return CliAction::ShowUsage {
            message: None,
            exit_code: 0,
        };
    }

    match command {
        Some("list") => expect_no_extra_args(args, Command::List),
        Some("where") => expect_one_value_arg(args, |name| Command::Where { name }),
        Some("all") => expect_one_value_arg(args, |name| Command::All { name }),
        Some("shadowed") => expect_no_extra_args(args, Command::Shadowed),
        Some("duplicates") => expect_no_extra_args(args, Command::Duplicates),
        Some("broken") => expect_no_extra_args(args, Command::Broken),
        Some("stats") => expect_no_extra_args(args, Command::Stats),
        Some("doctor") => expect_no_extra_args(args, Command::Doctor),
        _ => CliAction::ShowUsage {
            message: Some(format!(
                "Unknown command: {}",
                crate::output::render_os(&args[0])
            )),
            exit_code: 1,
        },
    }
}

pub fn usage_lines() -> &'static [&'static str] {
    &[
        "Usage: pathbin <COMMAND>",
        "",
        "Commands:",
        "  list        List executable binaries in PATH",
        "  where       Show where a command is located",
        "  all         Show all matching binaries with the same name",
        "  shadowed    Show binaries hidden by PATH priority",
        "  duplicates  Show duplicate binary names",
        "  broken      Show broken symlinks and missing PATH entries",
        "  stats       Show PATH binary statistics",
        "  doctor      Diagnose PATH-related problems",
    ]
}

fn expect_no_extra_args(args: &[OsString], command: Command) -> CliAction {
    if args.len() == 1 {
        CliAction::Execute(command)
    } else {
        invalid_arguments()
    }
}

fn expect_one_value_arg<F>(args: &[OsString], constructor: F) -> CliAction
where
    F: FnOnce(OsString) -> Command,
{
    if args.len() == 2 {
        CliAction::Execute(constructor(args[1].clone()))
    } else {
        invalid_arguments()
    }
}

fn invalid_arguments() -> CliAction {
    CliAction::ShowUsage {
        message: Some("Invalid arguments.".to_string()),
        exit_code: 1,
    }
}
