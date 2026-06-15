use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "codewiki",
    about = "Query GitHub repo wikis via Google Code Wiki"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Ask an AI-powered question about a repository
    Ask {
        /// Repository name (e.g. facebook/react)
        repo: String,
        /// Question to ask
        question: String,
    },
    /// List wiki section titles for a repository
    Structure {
        /// Repository name (e.g. facebook/react)
        repo: String,
    },
    /// Read full wiki contents (Markdown) for a repository
    Read {
        /// Repository name (e.g. facebook/react)
        repo: String,
        /// Output directory to write split markdown files (defaults to "wiki" if passed without argument)
        #[arg(short, long, num_args = 0..=1, default_missing_value = "wiki")]
        out_dir: Option<PathBuf>,
        /// Depth of splitting headings into files and directories (default: 2)
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn parses_ask_command() {
        let cli = Cli::try_parse_from(["codewiki", "ask", "facebook/react", "How it works?"])
            .expect("ask command should parse");
        match cli.command {
            Command::Ask { repo, question } => {
                assert_eq!(repo, "facebook/react");
                assert_eq!(question, "How it works?");
            }
            _ => panic!("expected ask command"),
        }
    }

    #[test]
    fn parses_structure_command() {
        let cli = Cli::try_parse_from(["codewiki", "structure", "facebook/react"])
            .expect("structure command should parse");
        match cli.command {
            Command::Structure { repo } => assert_eq!(repo, "facebook/react"),
            _ => panic!("expected structure command"),
        }
    }

    #[test]
    fn parses_read_command() {
        let cli = Cli::try_parse_from(["codewiki", "read", "facebook/react"])
            .expect("read command should parse");
        match cli.command {
            Command::Read { repo, out_dir, depth } => {
                assert_eq!(repo, "facebook/react");
                assert_eq!(out_dir, None);
                assert_eq!(depth, 2);
            }
            _ => panic!("expected read command"),
        }

        let cli_with_flag = Cli::try_parse_from(["codewiki", "read", "facebook/react", "-o"])
            .expect("read command with flag should parse");
        match cli_with_flag.command {
            Command::Read { repo, out_dir, depth } => {
                assert_eq!(repo, "facebook/react");
                assert_eq!(out_dir, Some(PathBuf::from("wiki")));
                assert_eq!(depth, 2);
            }
            _ => panic!("expected read command with flag"),
        }

        let cli_with_custom = Cli::try_parse_from(["codewiki", "read", "facebook/react", "-o", "custom_dir"])
            .expect("read command with custom dir should parse");
        match cli_with_custom.command {
            Command::Read { repo, out_dir, depth } => {
                assert_eq!(repo, "facebook/react");
                assert_eq!(out_dir, Some(PathBuf::from("custom_dir")));
                assert_eq!(depth, 2);
            }
            _ => panic!("expected read command with custom dir"),
        }
    }

    #[test]
    fn parses_read_command_with_depth() {
        let cli = Cli::try_parse_from(["codewiki", "read", "facebook/react", "-d", "3"])
            .expect("read command should parse");
        match cli.command {
            Command::Read { repo, out_dir, depth } => {
                assert_eq!(repo, "facebook/react");
                assert_eq!(out_dir, None);
                assert_eq!(depth, 3);
            }
            _ => panic!("expected read command"),
        }
    }

    #[test]
    fn fails_when_required_args_are_missing() {
        let result = Cli::try_parse_from(["codewiki", "ask", "facebook/react"]);
        assert!(result.is_err());
    }
}