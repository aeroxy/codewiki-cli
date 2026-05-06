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
    },
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;

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
            Command::Read { repo } => assert_eq!(repo, "facebook/react"),
            _ => panic!("expected read command"),
        }
    }

    #[test]
    fn fails_when_required_args_are_missing() {
        let result = Cli::try_parse_from(["codewiki", "ask", "facebook/react"]);
        assert!(result.is_err());
    }
}
