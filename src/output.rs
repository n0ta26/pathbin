use std::io::{self, Write};
use std::process;

pub fn write_stdout<I, S>(lines: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    write_lines(io::stdout(), lines);
}

pub fn write_stderr<I, S>(lines: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    write_lines(io::stderr(), lines);
}

fn write_lines<I, S, W>(mut stream: W, lines: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    W: Write,
{
    for line in lines {
        if let Err(err) = writeln!(stream, "{}", line.as_ref()) {
            handle_output_error(err);
        }
    }
}

fn handle_output_error(err: io::Error) -> ! {
    if err.kind() == io::ErrorKind::BrokenPipe {
        process::exit(0);
    }

    let _ = writeln!(io::stderr(), "Failed to write output: {err}");
    process::exit(1);
}
