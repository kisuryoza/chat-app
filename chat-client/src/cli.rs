use std::sync::Arc;

use chat_core::prelude::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Cli {
    Quit,
    Login,
    Register,
    Handshake,
    Text(Arc<str>),
}

pub fn ask_for_command() -> Result<Cli> {
    println!("Commands:");
    println!("          ':q'");
    println!("          ':login'");
    println!("          ':register'");
    println!("          ':handshake'");

    let cmd = process_input()?;
    if let Cli::Text(_) = cmd {
        Err(Error::generic("Not a command"))
    } else {
        Ok(cmd)
    }
}

pub fn process_input() -> Result<Cli> {
    let input = read_input()?;

    if !input.starts_with(':') {
        return Ok(Cli::Text(input));
    }

    match input.as_ref() {
        ":q" => Ok(Cli::Quit),
        ":login" => Ok(Cli::Login),
        ":register" => Ok(Cli::Register),
        ":handshake" => Ok(Cli::Handshake),
        _ => Err(Error::generic("Wrong command")),
    }
}

fn read_input() -> Result<Arc<str>> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(Error::io)?;
    let input = input.trim().trim_end_matches('\n');
    let input = Arc::from(input);
    Ok(input)
}

pub fn ask_for_credentials() -> Result<(String, String)> {
    println!("Enter username:");
    let username = read_input()?;

    println!("Enter password:");
    let password = read_input()?;

    Ok((username.to_string(), password.to_string()))
}
