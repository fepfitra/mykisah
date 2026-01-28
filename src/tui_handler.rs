use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};
use std::{
    io::{self, stdout, Write},
};

pub async fn run_tui() -> anyhow::Result<()> {
    let mut history: Vec<(String, String)> = Vec::new();

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

        if input.to_lowercase() == "ping" {
            history.push(("AI".to_string(), "pong".to_string()));
        }
    }

    Ok(())
}
