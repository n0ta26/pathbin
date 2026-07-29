use std::ffi::OsStr;
use std::fmt::Write as FmtWrite;
use std::io::{self, Write};
use std::path::Path;
use std::process;

pub fn render_text(value: &str) -> String {
    let mut rendered = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_control() {
            rendered.extend(character.escape_default());
        } else {
            rendered.push(character);
        }
    }
    rendered
}

pub fn render_os(value: &OsStr) -> String {
    match value.to_str() {
        Some(value) => render_text(value),
        None => render_non_utf8(value),
    }
}

pub fn render_path(path: &Path) -> String {
    render_os(path.as_os_str())
}

#[cfg(unix)]
fn render_non_utf8(value: &OsStr) -> String {
    use std::os::unix::ffi::OsStrExt;

    let mut rendered = String::with_capacity(value.as_bytes().len());
    for byte in value.as_bytes() {
        match *byte {
            b'\\' => rendered.push_str("\\\\"),
            b'\n' => rendered.push_str("\\n"),
            b'\r' => rendered.push_str("\\r"),
            b'\t' => rendered.push_str("\\t"),
            0x20..=0x7e => rendered.push(char::from(*byte)),
            byte => write!(rendered, "\\x{byte:02X}").expect("writing to a String cannot fail"),
        }
    }
    rendered
}

#[cfg(windows)]
fn render_non_utf8(value: &OsStr) -> String {
    use std::char::decode_utf16;
    use std::os::windows::ffi::OsStrExt;

    let mut rendered = String::new();
    for decoded in decode_utf16(value.encode_wide()) {
        match decoded {
            Ok(character) if character.is_control() => {
                rendered.extend(character.escape_default());
            }
            Ok(character) => rendered.push(character),
            Err(error) => write!(rendered, "\\u{{{:04X}}}", error.unpaired_surrogate())
                .expect("writing to a String cannot fail"),
        }
    }
    rendered
}

#[cfg(not(any(unix, windows)))]
fn render_non_utf8(value: &OsStr) -> String {
    format!("{value:?}")
}

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

#[cfg(test)]
mod tests {
    use super::{render_os, render_text};

    #[test]
    fn printable_text_is_unchanged() {
        assert_eq!(render_text("normal/path-日本語"), "normal/path-日本語");
    }

    #[test]
    fn control_characters_are_escaped() {
        assert_eq!(
            render_text("line\ncolumn\t\u{1b}[31m"),
            "line\\ncolumn\\t\\u{1b}[31m"
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_names_are_rendered_as_distinct_byte_sequences() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let first = OsStr::from_bytes(b"tool-\xfe");
        let second = OsStr::from_bytes(b"tool-\xff");

        assert_eq!(render_os(first), "tool-\\xFE");
        assert_eq!(render_os(second), "tool-\\xFF");
        assert_ne!(render_os(first), render_os(second));
    }
}
