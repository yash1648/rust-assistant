use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;

pub fn print_colored(msg: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color)).set_bold(true);
    let _ = stdout.set_color(&spec);
    let _ = writeln!(&mut stdout, "{}", msg);
    let _ = stdout.reset();
}

pub fn success(msg: &str) { print_colored(msg, Color::Green) }
pub fn warn(msg: &str) { print_colored(msg, Color::Yellow) }
pub fn error(msg: &str) { print_colored(msg, Color::Red) }
pub fn info(msg: &str) { print_colored(msg, Color::Cyan) }
