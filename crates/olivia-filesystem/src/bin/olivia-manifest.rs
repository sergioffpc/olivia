//! Writes the manifest of a game's source for the web: `olivia-manifest <dir>` lists the files of
//! `<dir>/assets/` in `<dir>/assets.txt`.

use std::env;
use std::error::Error as _;
use std::path::Path;
use std::process::ExitCode;

#[expect(
    clippy::print_stderr,
    reason = "a command line tool reports on the terminal"
)]
fn main() -> ExitCode {
    let Some(dir) = env::args_os().nth(1) else {
        eprintln!("usage: olivia-manifest <dir>");
        return ExitCode::FAILURE;
    };
    if let Err(error) = olivia_filesystem::write_manifest(Path::new(&dir)) {
        match error.source() {
            Some(source) => eprintln!("olivia-manifest: {error}: {source}"),
            None => eprintln!("olivia-manifest: {error}"),
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
