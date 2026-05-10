#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    List,
    Where { name: String },
    All { name: String },
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

pub fn parse_action(args: &[String]) -> CliAction {
    if args.is_empty() {
        return CliAction::ShowUsage {
            message: None,
            exit_code: 1,
        };
    }

    let command = args[0].as_str();
    if matches!(command, "-h" | "--help" | "help") {
        return CliAction::ShowUsage {
            message: None,
            exit_code: 0,
        };
    }

    match command {
        "list" => expect_no_extra_args(args, Command::List),
        "where" => expect_one_value_arg(args, |name| Command::Where { name }),
        "all" => expect_one_value_arg(args, |name| Command::All { name }),
        "shadowed" => expect_no_extra_args(args, Command::Shadowed),
        "duplicates" => expect_no_extra_args(args, Command::Duplicates),
        "broken" => expect_no_extra_args(args, Command::Broken),
        "stats" => expect_no_extra_args(args, Command::Stats),
        "doctor" => expect_no_extra_args(args, Command::Doctor),
        _ => CliAction::ShowUsage {
            message: Some(format!("Unknown command: {command}")),
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

fn expect_no_extra_args(args: &[String], command: Command) -> CliAction {
    if args.len() == 1 {
        CliAction::Execute(command)
    } else {
        invalid_arguments()
    }
}

fn expect_one_value_arg<F>(args: &[String], constructor: F) -> CliAction
where
    F: FnOnce(String) -> Command,
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
