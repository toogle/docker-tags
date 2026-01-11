use std::process::ExitCode;

use anyhow::{Result, anyhow};
use clap::Parser;
use docker_tags::Image;
use regex::Regex;

/// Docker Tags CLI
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Sort tags in reverse order
    #[arg(short = 'r', long, action)]
    reverse: bool,

    /// Maximum number of tags to fetch
    #[arg(short = 'n', long)]
    limit: Option<usize>,

    /// Filter tags by a pattern
    #[arg(short = 'f', long = "filter")]
    pattern: Option<String>,

    /// Docker image name
    image: String,
}

async fn print_tags(
    image_name: &str,
    reverse: bool,
    pattern: Option<&str>,
    limit: Option<usize>,
) -> Result<()> {
    let image =
        Image::try_from(image_name).map_err(|_| anyhow!("Invalid image name: {image_name:?}"))?;
    let mut tags = image.fetch_tags().await?;

    tags.sort();
    if reverse {
        tags.reverse();
    }
    if let Some(pattern) = pattern {
        let re = Regex::new(pattern).map_err(|_| anyhow!("Invalid regex pattern: {pattern:?}"))?;
        tags.retain(|tag| re.is_match(&tag.to_string()));
    }
    if let Some(limit) = limit {
        tags.truncate(limit);
    }

    for tag in tags {
        println!("{tag}");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    if let Err(err) = print_tags(
        args.image.as_str(),
        args.reverse,
        args.pattern.as_deref(),
        args.limit,
    )
    .await
    {
        println!("Error: {err}");
        for (level, cause) in err.chain().skip(1).enumerate() {
            eprintln!(
                "{:indent$}Caused by: {}",
                "",
                cause,
                indent = (level + 1) * 2
            );
        }
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}
