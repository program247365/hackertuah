use std::error::Error;

use crate::types::{ClaudeRequest, Comment, FlatComment, Message, Section, Story};

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

pub async fn fetch_comments(
    story: &Story,
) -> Result<Vec<FlatComment>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let mut flat_comments = Vec::new();
    fetch_comment_tree(&client, &story.kids, 0, &mut flat_comments, 4).await?;
    Ok(flat_comments)
}

async fn fetch_comment_tree(
    client: &reqwest::Client,
    kid_ids: &[u32],
    depth: usize,
    out: &mut Vec<FlatComment>,
    max_depth: usize,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    for &id in kid_ids {
        let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
        let resp = client.get(&url).send().await?;
        let comment: Comment = match resp.json().await {
            Ok(c) => c,
            Err(_) => continue,
        };
        if comment.deleted || comment.dead {
            continue;
        }
        let child_ids = comment.kids.clone();
        out.push(FlatComment { comment, depth });
        if depth < max_depth {
            Box::pin(fetch_comment_tree(
                client,
                &child_ids,
                depth + 1,
                out,
                max_depth,
            ))
            .await?;
        }
    }
    Ok(())
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
