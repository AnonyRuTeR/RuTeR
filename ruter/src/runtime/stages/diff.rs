use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn render_diff(
    crate_path: &Path,
    before: &HashMap<PathBuf, String>,
    after: &HashMap<PathBuf, String>,
) -> String {
    let mut files: Vec<_> = before.keys().collect();
    files.sort();

    let mut out = String::new();
    for file in files {
        let old_content = before.get(file).expect("file exists in before");
        let Some(new_content) = after.get(file) else {
            continue;
        };
        if old_content == new_content {
            continue;
        }

        let relative = file.strip_prefix(crate_path).unwrap_or(file);
        out.push_str(&format!("--- a/{}\n", relative.display()));
        out.push_str(&format!("+++ b/{}\n", relative.display()));
        out.push_str("@@\n");

        for change in ::diff::lines(old_content, new_content) {
            match change {
                ::diff::Result::Left(line) => {
                    out.push('-');
                    out.push_str(line);
                    out.push('\n');
                }
                ::diff::Result::Right(line) => {
                    out.push('+');
                    out.push_str(line);
                    out.push('\n');
                }
                ::diff::Result::Both(line, _) => {
                    out.push(' ');
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
    }

    out
}
