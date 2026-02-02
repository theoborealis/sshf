use anyhow::{Context, Result};
use clap::{ArgGroup, Parser};
use std::path::PathBuf;
use tokio::net::UnixListener;

mod filter;

use filter::{FilterAgent, FilterMode};

#[derive(Parser, Debug)]
#[command(name = "sshf", about = "Simple SSH agent filtering proxy")]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["whitelist", "blacklist"])
))]
struct Args {
    /// Only allow keys matching this glob pattern (on comment)
    #[arg(long, value_name = "PATTERN")]
    whitelist: Option<String>,

    /// Block keys matching this glob pattern (on comment)
    #[arg(long, value_name = "PATTERN")]
    blacklist: Option<String>,

    /// Path to upstream ssh-agent socket
    #[arg(value_name = "INPUT_SOCKET")]
    input: PathBuf,

    /// Path for filtered socket to create
    #[arg(value_name = "OUTPUT_SOCKET")]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let (pattern, mode) = if let Some(p) = args.whitelist {
        (p, FilterMode::Whitelist)
    } else if let Some(p) = args.blacklist {
        (p, FilterMode::Blacklist)
    } else {
        unreachable!("clap ensures one of whitelist/blacklist is set")
    };

    // Validate input socket exists
    if !args.input.exists() {
        anyhow::bail!("Input socket does not exist: {}", args.input.display());
    }

    if args.output.exists() {
        anyhow::bail!("Output socket already exists: {}", args.output.display());
    }

    // Create the listener
    let listener = UnixListener::bind(&args.output)
        .with_context(|| format!("Failed to bind to socket: {}", args.output.display()))?;

    eprintln!(
        "sshf: filtering {} -> {} ({:?} '{}')",
        args.input.display(),
        args.output.display(),
        mode,
        pattern
    );

    // Set up cleanup on ctrl+c
    let output_path = args.output.clone();
    ctrlc::set_handler(move || {
        let _ = std::fs::remove_file(&output_path);
        std::process::exit(0);
    })
    .ok();

    // Create the filter agent
    let agent = FilterAgent::new(args.input, pattern, mode);

    // Start listening
    ssh_agent_lib::agent::listen(listener, agent).await?;

    Ok(())
}
