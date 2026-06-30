use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";

pub struct Reporter {
    verbosity: u8,
    log_file: Option<std::fs::File>,
}

impl Reporter {
    pub fn new(verbosity: u8, log_path: Option<&Path>) -> std::io::Result<Self> {
        let log_file = match log_path {
            Some(path) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                Some(OpenOptions::new().create(true).append(true).open(path)?)
            }
            None => None,
        };

        Ok(Self {
            verbosity,
            log_file,
        })
    }

    pub fn stage_start(&mut self, stage: &str) -> Instant {
        self.emit(0, "", "");
        let plain = format!("=== STAGE: {} ===", stage.to_uppercase());
        let color = format!("{BOLD}{CYAN}{plain}{RESET}");
        self.emit(0, &plain, &color);
        Instant::now()
    }

    pub fn stage_end(&mut self, stage: &str, started: Instant) {
        let plain = format!("--- stage {stage} done in {:?} ---", started.elapsed());
        let color = format!("{GREEN}{plain}{RESET}");
        self.emit(0, &plain, &color);
    }

    pub fn info(&mut self, msg: impl AsRef<str>) {
        self.emit(0, msg.as_ref(), msg.as_ref());
    }

    pub fn error(&mut self, msg: impl AsRef<str>) {
        let plain = format!("[error] {}", msg.as_ref());
        let color = format!("{BOLD}{RED}{plain}{RESET}");
        self.emit(0, &plain, &color);
    }

    pub fn section(&mut self, title: impl AsRef<str>) {
        let plain = format!("  [{}]", title.as_ref());
        let color = format!("{YELLOW}{plain}{RESET}");
        self.emit(0, &plain, &color);
    }

    pub fn section_colored(&mut self, title: impl AsRef<str>, color_name: &str) {
        let plain = format!("  [{}]", title.as_ref());
        let color_code = match color_name {
            "cyan" => CYAN,
            "green" => GREEN,
            "magenta" => MAGENTA,
            "blue" => BLUE,
            _ => YELLOW,
        };
        let color = format!("{color_code}{plain}{RESET}");
        self.emit(0, &plain, &color);
    }

    pub fn kv(&mut self, level: u8, key: impl AsRef<str>, value: impl AsRef<str>) {
        let plain = format!("    - {}: {}", key.as_ref(), value.as_ref());
        let color = format!("{BLUE}    -{RESET} {}: {}", key.as_ref(), value.as_ref());
        self.emit(level, &plain, &color);
    }

    pub fn item(&mut self, level: u8, value: impl AsRef<str>) {
        let plain = format!("    * {}", value.as_ref());
        let color = format!("{BLUE}    *{RESET} {}", value.as_ref());
        self.emit(level, &plain, &color);
    }

    fn emit(&mut self, min_level: u8, plain: &str, colorized: &str) {
        if self.verbosity >= min_level {
            println!("{colorized}");
        }
        if let Some(file) = self.log_file.as_mut() {
            let _ = writeln!(file, "{plain}");
        }
    }
}
