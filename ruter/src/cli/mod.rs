use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

const APP_LONG_ABOUT: &str = "Automated Rust unit-test repair workflow.\n\
\n\
Default behavior:\n\
- dry-run by default (no source write-back)\n\
- only test code is allowed to be edited\n\
- topk defaults to 3\n\
- LLM is disabled by default\n\
- config priority: CLI > ENV > TOML > defaults\n\
\n\
Use `fix` for one-shot compile->analyze->plan->verify->apply/summarize.\n\
Use `step` for stage-by-stage debugging (compile/analyze/plan/verify/apply).";

const APP_AFTER_HELP: &str = "Common repair operations:\n\
  1) Dry-run fix:\n\
     cargo run -- fix <crate_path> --artifacts-dir <dir>\n\
  2) Apply accepted patch:\n\
     cargo run -- --apply fix <crate_path> --artifacts-dir <dir>\n\
  3) Online LLM verify path:\n\
     cargo run -- step verify <crate_path> --enable-llm --llm-mode online \\\n\
       --llm-api-url <url> --llm-model <model> --artifacts-dir <dir>\n\
\n\
Important defaults:\n\
  --topk: 3\n\
  --llm-timeout-secs: 60\n\
  --llm-max-rounds: 3\n\
  --llm-context-max-chars: 12000\n\
  --llm-output-token-ratio: 2.0\n\
  --llm-max-candidates: 3\n\
\n\
Debug option:\n\
  --llm-debug-dump-full-io: dump full prompt/response into artifacts";

#[derive(Debug, Parser)]
#[command(name = "ruter")]
#[command(about = "Automated E0433 patch workflow for Rust crates")]
#[command(long_about = APP_LONG_ABOUT)]
#[command(after_help = APP_AFTER_HELP)]
pub struct Cli {
    #[arg(short = 'v', action = ArgAction::Count, global = true)]
    pub verbose: u8,

    #[arg(long, global = true)]
    pub apply: bool,

    #[arg(long, global = true)]
    pub no_backup: bool,

    #[arg(long, global = true)]
    pub diff_file: Option<PathBuf>,

    #[arg(long, global = true)]
    pub log_file: Option<PathBuf>,

    #[arg(long, global = true)]
    pub run_tests: bool,

    #[arg(long, global = true)]
    pub artifacts_dir: Option<PathBuf>,

    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, help = "Top-K candidate width (default: 3)")]
    pub topk: Option<usize>,

    #[arg(long, global = true)]
    pub keep_updated_sources: bool,

    #[arg(
        long,
        global = true,
        help = "Enable LLM patcher path (default: disabled)"
    )]
    pub enable_llm: bool,

    #[arg(long, global = true, value_enum, help = "LLM mode (default: replay)")]
    pub llm_mode: Option<LlmModeArg>,

    #[arg(long, global = true)]
    pub llm_replay_file: Option<PathBuf>,

    #[arg(long, global = true)]
    pub llm_api_url: Option<String>,

    #[arg(long, global = true)]
    pub llm_model: Option<String>,

    #[arg(
        long,
        global = true,
        help = "LLM request timeout seconds (default: 60)"
    )]
    pub llm_timeout_secs: Option<u64>,

    #[arg(long, global = true, help = "Per-function max rounds (default: 3)")]
    pub llm_max_rounds: Option<u8>,

    #[arg(long, global = true, help = "Max candidates per round (default: 3)")]
    pub llm_max_candidates: Option<usize>,

    #[arg(long, global = true, help = "Context char budget (default: 12000)")]
    pub llm_context_max_chars: Option<usize>,

    #[arg(long, global = true)]
    pub llm_target_fn_hard_limit_chars: Option<usize>,

    #[arg(long, global = true)]
    pub llm_raw_excerpt_max_chars: Option<usize>,

    #[arg(
        long,
        global = true,
        help = "Online max_tokens cap ratio, cap=floor(input_tokens_est*ratio) (default: 2.0)"
    )]
    pub llm_output_token_ratio: Option<f64>,

    #[arg(
        long,
        global = true,
        help = "Dump full per-round LLM prompts and raw responses to artifacts"
    )]
    pub llm_debug_dump_full_io: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// One-shot workflow: compile -> analyze -> plan -> verify -> apply/dry-run -> summarize
    Fix { crate_path: PathBuf },
    /// Run a single stage in step mode
    Step { stage: Stage, crate_path: PathBuf },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Stage {
    Compile,
    Analyze,
    Plan,
    Verify,
    Apply,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LlmModeArg {
    Replay,
    Online,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fix_command_with_apply_and_backup_flags() {
        let cli = Cli::parse_from([
            "ruter",
            "fix",
            "./crate",
            "--apply",
            "--no-backup",
            "--run-tests",
            "--keep-updated-sources",
            "--enable-llm",
            "--llm-mode",
            "online",
            "--llm-replay-file",
            "./replay.json",
            "--llm-api-url",
            "https://example.test/v1",
            "--llm-model",
            "gpt-4o-mini",
            "--llm-timeout-secs",
            "60",
            "--llm-max-rounds",
            "3",
            "--llm-context-max-chars",
            "12000",
            "--llm-output-token-ratio",
            "2.0",
            "--topk",
            "3",
            "-vv",
        ]);

        assert!(matches!(cli.command, Command::Fix { .. }));
        assert!(cli.apply);
        assert!(cli.no_backup);
        assert!(cli.run_tests);
        assert!(cli.keep_updated_sources);
        assert!(cli.enable_llm);
        assert!(matches!(cli.llm_mode, Some(LlmModeArg::Online)));
        assert_eq!(cli.llm_replay_file, Some(PathBuf::from("./replay.json")));
        assert_eq!(cli.llm_api_url.as_deref(), Some("https://example.test/v1"));
        assert_eq!(cli.llm_model.as_deref(), Some("gpt-4o-mini"));
        assert_eq!(cli.llm_timeout_secs, Some(60));
        assert_eq!(cli.llm_max_rounds, Some(3));
        assert_eq!(cli.llm_context_max_chars, Some(12000));
        assert_eq!(cli.llm_output_token_ratio, Some(2.0));
        assert_eq!(cli.topk, Some(3));
        assert_eq!(cli.verbose, 2);
    }

    #[test]
    fn parse_step_command_with_stage() {
        let cli = Cli::parse_from(["ruter", "step", "plan", "./crate"]);

        match cli.command {
            Command::Step { stage, crate_path } => {
                assert!(matches!(stage, Stage::Plan));
                assert_eq!(crate_path, PathBuf::from("./crate"));
            }
            _ => panic!("expected step command"),
        }
        assert!(!cli.keep_updated_sources);
        assert!(!cli.enable_llm);
        assert!(cli.llm_mode.is_none());
        assert_eq!(cli.llm_replay_file, None);
        assert!(cli.llm_api_url.is_none());
        assert!(cli.llm_model.is_none());
        assert!(cli.llm_timeout_secs.is_none());
        assert!(cli.llm_max_rounds.is_none());
        assert!(cli.llm_context_max_chars.is_none());
        assert!(cli.llm_output_token_ratio.is_none());
        assert!(cli.topk.is_none());
    }
}
