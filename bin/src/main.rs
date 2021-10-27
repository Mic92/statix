mod config;
mod err;
mod fix;
mod lint;
mod traits;

use std::io::{self, BufRead};

use crate::{
    err::{FixErr, SingleFixErr, StatixErr},
    traits::WriteDiagnostic,
};

use clap::Clap;
use config::{Opts, SubCommand};
use similar::TextDiff;

fn _main() -> Result<(), StatixErr> {
    let opts = Opts::parse();
    match opts.cmd {
        SubCommand::Check(check_config) => {
            let vfs = check_config.vfs()?;
            let (lints, errors): (Vec<_>, Vec<_>) =
                vfs.iter().map(lint::lint).partition(Result::is_ok);
            let lint_results = lints.into_iter().map(Result::unwrap);
            let errors = errors.into_iter().map(Result::unwrap_err);

            let mut stdout = io::stdout();
            lint_results.for_each(|r| {
                stdout.write(&r, &vfs, check_config.format).unwrap();
            });
            errors.for_each(|e| {
                eprintln!("{}", e);
            });
        }
        SubCommand::Fix(fix_config) => {
            let vfs = fix_config.vfs()?;
            for entry in vfs.iter() {
                if let Some(fix_result) = fix::all(entry.contents) {
                    if fix_config.diff_only {
                        let text_diff = TextDiff::from_lines(entry.contents, &fix_result.src);
                        let old_file = format!("{}", entry.file_path.display());
                        let new_file = format!("{} [fixed]", entry.file_path.display());
                        println!(
                            "{}",
                            text_diff
                                .unified_diff()
                                .context_radius(4)
                                .header(&old_file, &new_file)
                        );
                    } else {
                        let path = entry.file_path;
                        std::fs::write(path, &*fix_result.src).map_err(FixErr::InvalidPath)?;
                    }
                }
            }
        }
        // FIXME: this block nasty, configure in/out streams in `impl Single` maybe
        SubCommand::Single(single_config) => {
            let src = if let Some(path) = &single_config.target {
                std::fs::read_to_string(&path).map_err(SingleFixErr::InvalidPath)?
            } else {
                io::stdin().lock().lines().map(|l| l.unwrap()).collect::<Vec<String>>().join("\n")
            };

            let path_id = if let Some(path) = &single_config.target {
                path.display().to_string()
            } else {
                "<unknown>".to_owned()
            };

            let (line, col) = single_config.position;
            let single_fix_result = fix::single(line, col, &src)?;
            if single_config.diff_only {
                let text_diff = TextDiff::from_lines(src.as_str(), &single_fix_result.src);
                let old_file = format!("{}", path_id);
                let new_file = format!("{} [fixed]", path_id);
                println!(
                    "{}",
                    text_diff
                        .unified_diff()
                        .context_radius(4)
                        .header(&old_file, &new_file)
                );
            } else if let Some(path) = single_config.target {
                std::fs::write(&path, &*single_fix_result.src)
                    .map_err(SingleFixErr::InvalidPath)?;
            } else {
                print!("{}", &*single_fix_result.src)
            }
        }
    }
    Ok(())
}

fn main() {
    match _main() {
        Err(e) => eprintln!("{}", e),
        _ => (),
    }
}
