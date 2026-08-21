use rustyline::DefaultEditor;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub enum CommandType {
    Shutdown,
}

pub async fn console(command_tx: mpsc::Sender<CommandType>) {
    let mut readline = DefaultEditor::new().unwrap_or_else(|e| {
        error!("Console startup failed: {}", e);
        panic!();
    });

    info!("The console is started.");
    println!("Baihua Server v0.1.4 Console");
    println!("Type 'help' to get help.");

    loop {
        match readline.readline("Baihua >> ") {
            Ok(line) => {
                let _ = readline.add_history_entry(&line);
                let trimmed = line.trim();

                let command_output = match trimmed {
                    "stop" => Some(CommandType::Shutdown),
                    "help" => {
                        println!(
                            "\
                            Available commands: \n \
                                stop    - Shut down the server.\n \
                                help    - Get help.\
                            "
                        );
                        None
                    }
                    "" => None,
                    _ => {
                        println!("Unknown command: '{}'. Type 'help' to see help.", trimmed);
                        None
                    }
                };

                if matches!(command_output, Some(CommandType::Shutdown)) {
                    break;
                }

                if let Some(command) = command_output
                    && command_tx.send(command).await.is_err()
                {
                    warn!("Sending commands failed, and the server may be down.");
                    break;
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("Input interrupted (Ctrl-C).");
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("Receive EOF (Ctrl-D).");
                break;
            }
            Err(err) => {
                error!("An error occurred when reading a line: {:?}", err);
                break;
            }
        }
    }
}
