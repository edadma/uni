// Async REPL implementation using editline with tokio spawn_blocking

use editline::{LineEditor, terminals::StdioTerminal};
use std::io::Write;
use uni_core::{AsyncInterpreter, execute_string};
use crate::stdout_output::StdoutOutput;

pub async fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    println!("Uni Async v0.1.0");
    println!("Type 'quit' to exit, 'words' to see available words");
    println!();

    // Create interpreter with stdout output
    let mut interp = AsyncInterpreter::new();
    let output = Box::new(StdoutOutput::new());
    interp.set_async_output(output);

    // Inject Linux time source for date/time operations
    #[cfg(feature = "std")]
    {
        use uni_core::hardware::linux::LinuxTimeSource;
        interp.set_time_source(Box::new(LinuxTimeSource::new()));
    }

    // Load prelude (higher-level words defined in Uni)
    if let Err(e) = interp.load_prelude().await {
        eprintln!("Warning: Failed to load prelude: {:?}", e);
    }

    // Create editline editor and terminal (sync)
    let mut editor = LineEditor::new(1024, 50);
    let mut terminal = StdioTerminal::new();

    loop {
        // Print prompt
        print!("uni> ");
        std::io::stdout().flush()?;

        // Read a line using editline in a blocking task
        let line_result = tokio::task::spawn_blocking(move || {
            let result = editor.read_line(&mut terminal);
            (editor, terminal, result)
        }).await?;

        // Destructure the result
        let (ed, term, read_result) = line_result;
        editor = ed;
        terminal = term;

        match read_result {
            Ok(line) => {
                // Process the line
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Execute the line
                match execute_string(trimmed, &mut interp).await {
                    Ok(()) => {
                        // Success - optionally show stack
                        if !interp.stack.is_empty() {
                            print!("Stack: ");
                            for (i, value) in interp.stack.iter().enumerate() {
                                if i > 0 {
                                    print!(" ");
                                }
                                print!("{}", value);
                            }
                            println!();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(editline::Error::Eof) => {
                // EOF (Ctrl-D)
                println!("\nGoodbye!");
                break;
            }
            Err(editline::Error::Interrupted) => {
                // Ctrl-C - just continue
                println!("^C");
                continue;
            }
            Err(e) => {
                eprintln!("Input error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
