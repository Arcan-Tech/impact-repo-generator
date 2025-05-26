use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    proabilites: String,

    #[arg(short, long)]
    repository: String,

    #[arg(short, long, default_value_t = 10)]
    commits: u32,
}
