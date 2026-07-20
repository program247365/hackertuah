use clap::{Parser, Subcommand, ValueEnum};

use crate::hn_api;
use crate::types::{Section, Story};

#[derive(Parser)]
#[command(
    name = "hackertuah",
    version,
    about = "Hacker News in your terminal — TUI by default, agent-friendly subcommands for scripts"
)]
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

pub async fn run(cmd: Commands) -> i32 {
    match cmd {
        Commands::Stories {
            section,
            limit,
            json,
        } => run_stories(section.into(), limit, json).await,
    }
}

async fn run_stories(section: Section, limit: usize, json: bool) -> i32 {
    eprintln!("fetching up to {} {} stories...", limit, section.as_str());
    let stories = match hn_api::fetch_stories(section, limit).await {
        Ok(stories) => stories,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };
    if json {
        match serde_json::to_string_pretty(&stories) {
            Ok(out) => println!("{}", out),
            Err(e) => {
                eprintln!("error: {}", e);
                return 1;
            }
        }
    } else if stories.is_empty() {
        println!("(no stories)");
    } else {
        for (i, story) in stories.iter().enumerate() {
            print!("{}", format_story(story, i + 1));
        }
    }
    0
}

pub fn format_story(story: &Story, rank: usize) -> String {
    let mut out = format!("{:>3}. {}\n", rank, story.title);
    out.push_str(&format!(
        "     {} points by {} | {} comments\n",
        story.score, story.by, story.descendants
    ));
    if let Some(url) = &story.url {
        out.push_str(&format!("     {}\n", url));
    }
    out.push_str(&format!("     id:{}\n", story.id));
    out
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

    fn sample_story() -> Story {
        Story {
            id: 42,
            title: "Test story".to_string(),
            url: Some("https://example.com".to_string()),
            text: None,
            by: "pg".to_string(),
            score: 100,
            descendants: 12,
            kids: vec![],
        }
    }

    #[test]
    fn format_story_shows_rank_title_meta_url_and_id() {
        let out = format_story(&sample_story(), 1);
        assert!(out.contains("  1. Test story"));
        assert!(out.contains("100 points by pg | 12 comments"));
        assert!(out.contains("https://example.com"));
        assert!(out.contains("id:42"));
    }

    #[test]
    fn format_story_omits_url_line_when_absent() {
        let mut story = sample_story();
        story.url = None;
        let out = format_story(&story, 3);
        assert!(out.contains("  3. Test story"));
        assert!(!out.contains("https://"));
    }
}
