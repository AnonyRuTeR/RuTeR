mod cli;
mod config;
mod llm;
mod runtime;

use std::process::ExitCode;

use clap::Parser;
use cli::Cli;

fn main() -> ExitCode {
    // 本地开发默认尝试加载 .env，不覆盖已存在环境变量。
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();
    runtime::workflow::run(cli)
}
