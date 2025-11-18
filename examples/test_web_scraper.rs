//! Test Web Scraper Tool
//!
//! Quick example to test the web_scraper tool directly

use anyhow::Result;
use serde_json::json;
use spec_ai::tools::ToolRegistry;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to see debug output
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== Web Scraper Tool Test ===\n");

    // Create tool registry with all built-in tools
    let registry = ToolRegistry::with_builtin_tools(None, None);

    // Get the web_scraper tool
    let scraper = registry
        .get("web_scraper")
        .expect("web_scraper tool should be registered");

    println!("Tool: {}", scraper.name());
    println!("Description: {}\n", scraper.description());

    // Test 1: Simple scrape
    println!("Test 1: Scraping example.com...");
    let args = json!({
        "url": "https://example.com"
    });

    match scraper.execute(args).await {
        Ok(result) => {
            if result.success {
                println!("✓ Success!\n");

                // Parse and pretty-print the result
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&result.output) {
                    println!("Response:");
                    println!("{}\n", serde_json::to_string_pretty(&response)?);
                } else {
                    println!("Output:\n{}\n", result.output);
                }
            } else {
                println!("✗ Failed: {:?}\n", result.error);
            }
        }
        Err(e) => println!("✗ Error: {}\n", e),
    }

    // Test 2: Scrape with link extraction
    println!("\n---\n");
    println!("Test 2: Scraping docs.rs with link extraction...");
    let args = json!({
        "url": "https://docs.rs/spider/",
        "max_pages": 1,
        "extract_links": true
    });

    match scraper.execute(args).await {
        Ok(result) => {
            if result.success {
                println!("✓ Success!\n");

                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&result.output) {
                    // Show just the summary
                    if let Some(pages) = response["pages"].as_array() {
                        println!("Scraped {} page(s)", pages.len());
                        if let Some(page) = pages.first() {
                            println!("Title: {}", page["title"]);
                            println!("URL: {}", page["url"]);
                            if let Some(links) = page["links"].as_array() {
                                println!("Found {} links", links.len());
                                println!("First 3 links:");
                                for link in links.iter().take(3) {
                                    println!("  - {}", link);
                                }
                            }

                            // Show truncated content
                            if let Some(content) = page["content"].as_str() {
                                let preview = if content.len() > 200 {
                                    format!("{}...", &content[..200])
                                } else {
                                    content.to_string()
                                };
                                println!("\nContent preview:\n{}", preview);
                            }
                        }
                    }
                }
            } else {
                println!("✗ Failed: {:?}\n", result.error);
            }
        }
        Err(e) => println!("✗ Error: {}\n", e),
    }

    println!("\n=== Tests Complete ===");
    Ok(())
}
