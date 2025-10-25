// ASYNC CONCEPT: Words builtin - displays all defined words in the dictionary
// Usage: words  (displays all defined words in sorted, columnar format)

use crate::compat::{format, String, Vec};
use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;
use core::future::Future;
use core::pin::Pin;

pub fn words_builtin(interp: &mut AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>
{
    Box::pin(async move {
        // Collect all words first to avoid borrow checker issues
        let mut words: Vec<String> = interp
            .dictionary
            .keys()
            .map(|k| String::from(k.as_ref()))
            .collect();

        // Add special words that aren't in the dictionary
        words.push(String::from("exec"));
        words.push(String::from("if"));
        words.push(String::from("quit"));
        words.sort();

        // Display header
        let msg = format!("Defined words ({}):", words.len());
        interp.writeln_async(&msg).await.map_err(|_| {
            RuntimeError::TypeError("Failed to write to output".into())
        })?;

        // Calculate column width dynamically
        let max_len = words.iter().map(|w| w.len()).max().unwrap_or(0);
        let col_width = max_len + 3;

        // Display words in columns of 5
        for chunk in words.chunks(5) {
            let mut line = String::new();
            for word in chunk {
                use core::fmt::Write;
                let _ = write!(&mut line, "{:width$} ", word, width = col_width);
            }
            interp.writeln_async(&line).await.map_err(|_| {
                RuntimeError::TypeError("Failed to write to output".into())
            })?;
        }

        Ok(())
    })
}
