use clap::{Parser, Subcommand, ValueEnum};

use crate::types::Section;

#[derive(Parser)]
#[command(
    name = "hackertuah",
    version,
    about = "Hacker News in your terminal — TUI by default, agent-friendly subcommands for scripts"
)]
#[allow(dead_code)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List stories from a Hacker News section
    Stories {
        /// Section to fetch
        #[arg(value_enum, default_value_t = CliSection::Top)]
        section: CliSection,
        /// Maximum number of stories to fetch
        #[arg(long, default_value_t = 30)]
        limit: usize,
        /// Emit JSON on stdout instead of human-readable text
        #[arg(long)]
        json: bool,
    },
}

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CliSection {
    Top,
    Ask,
    Show,
    Jobs,
}

impl From<CliSection> for Section {
    fn from(s: CliSection) -> Section {
        match s {
            CliSection::Top => Section::Top,
            CliSection::Ask => Section::Ask,
            CliSection::Show => Section::Show,
            CliSection::Jobs => Section::Jobs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn bare_invocation_has_no_subcommand() {
        let cli = Cli::try_parse_from(["hackertuah"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn stories_defaults_to_top_limit_30_human_output() {
        let cli = Cli::try_parse_from(["hackertuah", "stories"]).unwrap();
        match cli.command {
            Some(Commands::Stories {
                section,
                limit,
                json,
            }) => {
                assert_eq!(section, CliSection::Top);
                assert_eq!(limit, 30);
                assert!(!json);
            }
            _ => panic!("expected stories subcommand"),
        }
    }

    #[test]
    fn stories_accepts_section_limit_and_json() {
        let cli = Cli::try_parse_from(["hackertuah", "stories", "ask", "--limit", "5", "--json"])
            .unwrap();
        match cli.command {
            Some(Commands::Stories {
                section,
                limit,
                json,
            }) => {
                assert_eq!(section, CliSection::Ask);
                assert_eq!(limit, 5);
                assert!(json);
            }
            _ => panic!("expected stories subcommand"),
        }
    }

    #[test]
    fn unknown_section_is_a_usage_error() {
        assert!(Cli::try_parse_from(["hackertuah", "stories", "bestest"]).is_err());
    }

    #[test]
    fn cli_section_maps_to_types_section() {
        assert_eq!(Section::from(CliSection::Jobs).as_str(), "Jobs");
    }
}
