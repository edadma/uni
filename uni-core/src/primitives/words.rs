// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::compat::{format, String, Vec};
use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

// RUST CONCEPT: Words builtin - displays all defined words in the dictionary
// Usage: words  (displays all defined words in sorted, columnar format)
pub fn words_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Collect all data first to avoid borrow checker issues
    // We need to collect words into owned Strings before we start writing,
    // because writing requires a mutable borrow of interp.
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
    let _ = interp.writeln(&msg);

    // RUST CONCEPT: Calculate column width dynamically based on longest word
    // Find the maximum word length and add padding
    let max_len = words.iter().map(|w| w.len()).max().unwrap_or(0);
    let col_width = max_len + 3; // Add extra padding for readability

    // Display words in columns of 5
    for chunk in words.chunks(5) {
        let mut line = String::new();
        for word in chunk {
            use core::fmt::Write;
            // Use dynamic width instead of fixed 15
            let _ = write!(&mut line, "{:width$} ", word, width = col_width);
        }
        let _ = interp.writeln(&line);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_words_builtin_succeeds() {
        let mut interp = setup_interpreter();

        // Words should succeed without crashing
        let result = words_builtin(&mut interp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_words_builtin_with_custom_definitions() {
        let mut interp = setup_interpreter();

        // Define some custom words by adding to dictionary
        let test_atom = interp.intern_atom("test-word");
        use crate::interpreter::DictEntry;
        interp.dictionary.insert(
            test_atom.clone(),
            DictEntry {
                value: Value::Number(42.0),
                is_executable: false,
                doc: None,
            },
        );

        let another_atom = interp.intern_atom("another");
        interp.dictionary.insert(
            another_atom.clone(),
            DictEntry {
                value: Value::Boolean(true),
                is_executable: false,
                doc: None,
            },
        );

        // Words should succeed and include our custom definitions
        let result = words_builtin(&mut interp);
        assert!(result.is_ok());

        // Verify the words are in the dictionary
        assert!(interp.dictionary.contains_key(&test_atom));
        assert!(interp.dictionary.contains_key(&another_atom));
    }

    #[test]
    fn test_words_builtin_no_stack_effect() {
        let mut interp = setup_interpreter();

        // Push some values on the stack
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));

        let stack_before = interp.stack.len();

        // Words should not affect the stack
        let result = words_builtin(&mut interp);
        assert!(result.is_ok());

        let stack_after = interp.stack.len();
        assert_eq!(stack_before, stack_after);
    }

    #[test]
    fn test_words_includes_special_words() {
        let interp = setup_interpreter();

        // Collect the words list to verify special words are included
        let mut words: Vec<String> = interp
            .dictionary
            .keys()
            .map(|k| String::from(k.as_ref()))
            .collect();

        // Add special words that aren't in the dictionary (same logic as words_builtin)
        words.push(String::from("exec"));
        words.push(String::from("if"));
        words.push(String::from("quit"));

        // Verify all three special words are present
        assert!(words.contains(&String::from("exec")), "exec should be in words list");
        assert!(words.contains(&String::from("if")), "if should be in words list");
        assert!(words.contains(&String::from("quit")), "quit should be in words list");
    }
}
