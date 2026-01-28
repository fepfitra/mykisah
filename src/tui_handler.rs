use anyhow::{Result, Context};
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};
use std::{
    io::{self, stdout, Write},
    env,
};

use crate::openrouter_client::OpenRouterClient;
use crate::openrouter_api::{ChatMessage, MessageRole};


pub async fn run_tui() -> Result<()> {
    let mut history: Vec<(String, String)> = Vec::new();

    let openrouter_api_key = env::var("OPENROUTER_API_KEY")
        .context("OPENROUTER_API_KEY environment variable not set")?;
    let openrouter_model = env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| "nvidia/nemotron-nano-12b-v2-vl:free".to_string());

    let openrouter_client = OpenRouterClient::new(openrouter_api_key, openrouter_model);


    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

    execute!(stdout(), Print("Welcome to the AI Chat TUI!\n"))?;
    execute!(stdout(), Print("Type your message and press Enter. Type 'exit' to quit.\n\n"))?;

    loop {
        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;

        execute!(stdout(), Print("Welcome to the AI Chat TUI!\n"))?;
        execute!(stdout(), Print("Type your message and press Enter. Type 'exit' to quit.\n\n"))?;

        for (speaker, message) in &history {
            if speaker == "You" {
                execute!(stdout(), Print(format!("{}: {}\n", speaker.clone().cyan(), message)))?;
            } else {
                execute!(stdout(), Print(format!("{}: {}\n", speaker.clone().green(), message)))?;
            }
        }

        execute!(stdout(), Print(format!("\n{}: ", "You".cyan())))?;
        stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.to_lowercase() == "exit" {
            break;
        }

        history.push(("You".to_string(), input.clone()));

        let chat_messages = vec![ChatMessage {
            role: MessageRole::User,
            content: input.clone(),
        }];

        match openrouter_client.get_chat_completion(chat_messages).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    let ai_response = choice.message.content.clone();
                    history.push(("AI".to_string(), ai_response));
                } else {
                    history.push(("AI".to_string(), "OpenRouter returned no choices.".to_string()));
                }
            }
            Err(e) => {
                history.push(("AI".to_string(), format!("Error getting OpenRouter completion: {}", e)));
            }
        }
    }

    Ok(())
}
