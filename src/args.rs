use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate a synthetic repository using a markov chain
    Generate {
        #[arg(short, long, help = "Markov model description YAML file.")]
        markov: PathBuf,

        #[arg(short, long, help = "Destination repository path")]
        repository: PathBuf,

        #[arg(
            short,
            long,
            default_value_t = 10,
            help = "Number of commits to generate"
        )]
        commits: u32,

        #[arg(
            long,
            default_value = "2023-01-01T12:00:00",
            help = "Generate commits starting at given date (YYYY-MM-DD)"
        )]
        start: String,

        #[arg(
            long,
            default_value_t = 10,
            help = "Average interval time between commits (hours)"
        )]
        hours: i64,

        #[arg(
            long,
            default_value = "INFO",
            help = "Logging level (trace, error, debug, warn, info)"
        )]
        log: String,
    },

    /// Export a markov model as a dot file to be visualized
    Dot {
        #[arg(short, long, help = "Markov model description YAML file.")]
        markov: PathBuf,

        #[arg(short, long, help = "Output dot file")]
        output: PathBuf,
    },
}
