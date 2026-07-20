use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_serializes_with_stable_field_names() {
        let story = Story {
            id: 42,
            title: "Test".to_string(),
            url: Some("https://example.com".to_string()),
            text: None,
            by: "pg".to_string(),
            score: 100,
            descendants: 12,
            kids: vec![1, 2],
        };
        let json = serde_json::to_value(&story).unwrap();
        assert_eq!(json["id"], 42);
        assert_eq!(json["title"], "Test");
        assert_eq!(json["url"], "https://example.com");
        assert_eq!(json["by"], "pg");
        assert_eq!(json["score"], 100);
        assert_eq!(json["descendants"], 12);
    }
}
