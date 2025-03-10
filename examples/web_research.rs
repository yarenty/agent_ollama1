use kowalski::{
    agent::{Agent, ToolingAgent},
    config::Config,
    role::{Audience, Preset, Role},
};
use std::io::{self, Write};
use log::info;
use env_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load configuration
    let config = Config::load()?;
    let mut tooling_agent = ToolingAgent::new(config)?;

    // Start a conversation
    info!("🤖 Starting tooling agent...");
    let conversation_id = tooling_agent.start_conversation("llama2");
    info!("Tooling Agent Conversation ID: {}", conversation_id);

    // Perform a web search
    let query = "Latest developments in Rust programming";
    println!("\n🔍 Searching: {}", query);
    let search_results = tooling_agent.search(query).await?;
    
    // Process search results
    for result in &search_results {
        tooling_agent.add_message(
            &conversation_id,
            "search",
            format!("{} : {}", result.title, result.snippet).as_str()
        ).await;
        
        println!("\n📑 Result:");
        println!("Title: {}", result.title);
        println!("URL: {}", result.url);
        println!("Snippet: {}", result.snippet);
    }

    // Add search query to conversation
    tooling_agent.add_message(
        &conversation_id,
        "user",
        format!("Search for {} and provide a summary", query).as_str()
    ).await;

    // Process the first search result in detail
    if let Some(first_result) = search_results.first() {
        println!("\n🌐 Processing first result: {}", first_result.url);
        let page = tooling_agent.fetch_page(&first_result.url).await?;

        // Add page content to conversation
        tooling_agent.add_message(
            &conversation_id,
            "search",
            format!("Full page: {} : {}", page.title, page.content).as_str()
        ).await;

        // Generate a simplified summary
        let role = Role::translator(Some(Audience::Family), Some(Preset::Simplify));
        println!("\n📝 Generating summary...");
        
        let mut response = tooling_agent
            .chat_with_history(&conversation_id, "Provide a simple summary", Some(role))
            .await?;

        // Process the streaming response
        let mut buffer = String::new();
        while let Some(chunk) = response.chunk().await? {
            match tooling_agent.process_stream_response(&conversation_id, &chunk).await {
                Ok(Some(content)) => {
                    print!("{}", content);
                    io::stdout().flush()?;
                    buffer.push_str(&content);
                }
                Ok(None) => {
                    tooling_agent.add_message(&conversation_id, "assistant", &buffer).await;
                    println!("\n✅ Summary complete!\n");
                    break;
                }
                Err(e) => {
                    eprintln!("\n❌ Error processing stream: {}", e);
                    break;
                }
            }
        }
    }

    Ok(())
} 