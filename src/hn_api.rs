use std::error::Error;

use crate::types::{ClaudeRequest, Message, Section, Story};

pub async fn fetch_stories(section: Section) -> Result<Vec<Story>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let ids: Vec<u32> = client
        .get(section.get_api_url())
        .send()
        .await?
        .json()
        .await?;

    let mut stories = Vec::new();
    for id in ids.iter().take(100) {
        let story: Story = client
            .get(format!(
                "https://hacker-news.firebaseio.com/v0/item/{}.json",
                id
            ))
            .send()
            .await?
            .json()
            .await?;
        stories.push(story);
    }

    Ok(stories)
}

pub async fn get_claude_summary(text: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let request = ClaudeRequest {
        model: "claude-3-opus-20240229".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: format!(
                "Please summarize this Hacker News post concisely:\n\n{}",
                text
            ),
        }],
        max_tokens: 150,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", std::env::var("CLAUDE_API_KEY")?)
        .json(&request)
        .send()
        .await?;

    Ok(response.text().await?)
}
