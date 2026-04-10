use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Story {
    pub id: u32,
    pub title: String,
    pub url: Option<String>,
    pub text: Option<String>,
    pub by: String,
    pub score: i32,
    #[serde(default)]
    pub descendants: u32,
    #[serde(default)]
    pub kids: Vec<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Comment {
    pub id: u32,
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub kids: Vec<u32>,
    #[serde(default)]
    pub time: u64,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub dead: bool,
}

pub struct FlatComment {
    pub comment: Comment,
    pub depth: usize,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Section {
    Top,
    Ask,
    Show,
    Jobs,
}

impl Section {
    pub fn as_str(&self) -> &str {
        match self {
            Section::Top => "Top",
            Section::Ask => "Ask",
            Section::Show => "Show",
            Section::Jobs => "Jobs",
        }
    }

    pub fn get_api_url(&self) -> String {
        match self {
            Section::Top => "https://hacker-news.firebaseio.com/v0/topstories.json".to_string(),
            Section::Ask => "https://hacker-news.firebaseio.com/v0/askstories.json".to_string(),
            Section::Show => "https://hacker-news.firebaseio.com/v0/showstories.json".to_string(),
            Section::Jobs => "https://hacker-news.firebaseio.com/v0/jobstories.json".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Menu,
    Summary,
    Comments,
    CommandPalette,
    Search,
}

#[derive(Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
}

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
