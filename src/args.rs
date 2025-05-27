use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub probabilites: PathBuf,

    #[arg(short, long)]
    pub repository: PathBuf,

    #[arg(short, long, default_value_t = 10)]
    pub commits: u32,

    #[arg(
        long,
        default_value = "2023-01-01T12:00:00",
        help = "Generate commits starting at given date (YYYY-MM-DD)"
    )]
    pub start: String,

    #[arg(
        long,
        default_value_t = 10,
        help = "Average interval time between commits (hours)"
    )]
    pub hours: i64,

    #[arg(long, help = "Ovewrite existing repository")]
    pub force: bool,

    #[arg(
        short = 'C',
        long,
        default_value_t = 3,
        help = "Average commits per issue"
    )]
    pub commits_per_issue: u32,

    #[arg(
        long,
        default_value = "#",
        help = "Prefix to mark issues in commit messages"
    )]
    pub issue_prefix: String,
}
