use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn print_colored(msg: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color)).set_bold(true);
    let _ = stdout.set_color(&spec);
    let _ = writeln!(&mut stdout, "{}", msg);
    let _ = stdout.reset();
}

pub fn success(msg: &str) {
    print_colored(msg, Color::Green)
}
pub fn error(msg: &str) {
    print_colored(msg, Color::Red)
}
pub fn info(msg: &str) {
    print_colored(msg, Color::Cyan)
}
pub fn warning(msg: &str) {
    print_colored(msg, Color::Yellow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_functions_dont_panic() {
        // These write to stdout — just verify they don't panic
        success("test success");
        info("test info");
        warning("test warning");
        error("test error");
    }
}
