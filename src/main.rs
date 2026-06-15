mod boq;
mod cli;
mod client;
mod output;
mod spinner;
mod wiki;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use client::CodeWikiClient;
use spinner::Spinner;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let (repo, query_type) = repo_and_query_type(&cli.command);

    if let Ok(mock_text) = std::env::var("CODEWIKI_MOCK_TEXT") {
        println!(
            "{}",
            output::format_for_claude(&mock_text, repo, query_type)
        );
        return Ok(());
    }

    let spinner = Spinner::start("Bootstrapping Code Wiki...");
    let client = CodeWikiClient::connect().await?;

    spinner.set_message(&command_spinner_message(&cli.command));
    let text = match &cli.command {
        Command::Ask { repo, question } => {
            let raw = client.ask(repo, question).await?;
            wiki::resolve_links(&raw)
        }
        Command::Structure { repo } => {
            let w = client.read_wiki(repo).await?;
            wiki::render_structure(&w)
        }
        Command::Read { repo, out_dir, depth } => {
            let w = client.read_wiki(repo).await?;
            if let Some(dir) = out_dir {
                std::fs::create_dir_all(dir)?;
                let split_files = wiki::split_by_depth(&w, *depth);
                for file in &split_files {
                    let path = dir.join(&file.path);
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, &file.content)?;
                }
                format!("Successfully wrote {} markdown files to {}", split_files.len(), dir.display())
            } else {
                wiki::render_markdown(&w)
            }
        }
    };
    spinner.finish();

    println!("{}", output::format_for_claude(&text, repo, query_type));
    Ok(())
}

fn command_spinner_message(command: &Command) -> String {
    match command {
        Command::Ask { repo, .. } => format!("Asking Code Wiki about {}...", repo),
        Command::Structure { repo } => format!("Fetching wiki structure for {}...", repo),
        Command::Read { repo, .. } => format!("Reading wiki contents for {}...", repo),
    }
}

fn repo_and_query_type(command: &Command) -> (&str, &str) {
    match command {
        Command::Ask { repo, .. } => (repo, "ask"),
        Command::Structure { repo } => (repo, "structure"),
        Command::Read { repo, .. } => (repo, "read"),
    }
}