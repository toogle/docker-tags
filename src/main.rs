use std::{env, process::ExitCode};

use anyhow::{Result, anyhow};
use docker_tags::Image;

async fn print_tags(image_name: &str, reverse: bool) -> Result<()> {
    let image =
        Image::try_from(image_name).map_err(|_| anyhow!("Invalid image name: {image_name:?}"))?;
    let mut tags = image.fetch_tags().await?;
    tags.sort();
    if reverse {
        tags.reverse();
    }
    for tag in tags {
        println!("{tag}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [-r] <image>", args[0]);
        return ExitCode::from(1);
    }

    let mut reverse = false;
    let mut image_name = "";
    for arg in &args[1..] {
        if arg == "-r" {
            reverse = true;
        } else if image_name.is_empty() {
            image_name = arg;
        } else {
            eprintln!("Unknown argument: {arg}");
            return ExitCode::from(1);
        }
    }

    if let Err(err) = print_tags(image_name, reverse).await {
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
