//! Agent with Web Scraper
//!
//! Shows how an agent can use the web_scraper tool

use anyhow::Result;
use spec_ai::agent::AgentBuilder;
use spec_ai::config::{AgentProfile, AppConfig};
use spec_ai::persistence::Persistence;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Agent Web Scraper Demo ===\n");

    let config = AppConfig::load().unwrap_or_default();
    let db_path = PathBuf::from("examples/scraper_demo.duckdb");
    let persistence = Persistence::new(&db_path)?;

    let profile = AgentProfile {
        prompt: Some("You are a helpful research assistant. When asked to scrape a webpage, use the web_scraper tool to get the content, then summarize it clearly.".to_string()),
        style: Some("professional".to_string()),
        temperature: Some(0.7),
        // Configure your actual model provider here
        model_provider: Some("lmstudio".to_string()),  // or "openai", "ollama", etc.
        model_name: Some("qwen/qwen3-vl-8b".to_string()),
        allowed_tools: Some(vec![
            "web_scraper".to_string(),
            "echo".to_string(),
        ]),
        memory_k: 10,
        ..AgentProfile::default()
    };

    let mut agent = AgentBuilder::new()
        .with_profile(profile)
        .with_config(config)
        .with_persistence(persistence)
        .with_session_id("scraper-test")
        .build()?;

    // Test prompts
    let prompts = vec![
        "Scrape https://example.com and tell me what you find",
        "What does the page at https://www.rust-lang.org/ say about Rust?",
    ];

    for (i, prompt) in prompts.iter().enumerate() {
        println!("\n--- Test {} ---", i + 1);
        println!("Prompt: {}\n", prompt);

        match agent.run_step(prompt).await {
            Ok(response) => {
                println!("Response:\n{}\n", response.response);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }

        println!("\n{}", "=".repeat(60));
    }

    Ok(())
}
