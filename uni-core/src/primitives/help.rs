use crate::compat::{Box, format, Rc, ToString};
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use core::future::Future;
use core::pin::Pin;

const IF_DOC: &str = "Conditional branching. Usage: condition true-branch false-branch if";
const EXEC_DOC: &str =
    "Execute the value at the top of the stack. Lists run as code, other values execute directly.";
const QUIT_DOC: &str = "Exit the REPL or terminate script execution. Usage: quit";

pub fn help_builtin(interp: &mut AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>
{
    Box::pin(async move {
        help_impl(interp).await
    })
}

async fn help_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let word = interp.pop()?;
    let atom = match word {
        Value::Atom(name) => name,
        _ => {
            return Err(RuntimeError::TypeError(
                "help expects an atom (use 'word help)".to_string(),
            ));
        }
    };

    let (doc, is_executable) = if atom.as_ref() == "if" {
        (Some(Rc::<str>::from(IF_DOC)), true)
    } else if atom.as_ref() == "exec" {
        (Some(Rc::<str>::from(EXEC_DOC)), true)
    } else if atom.as_ref() == "quit" {
        (Some(Rc::<str>::from(QUIT_DOC)), true)
    } else {
        let entry = interp
            .dictionary
            .get(&atom)
            .ok_or_else(|| RuntimeError::UndefinedWord(atom.to_string()))?;
        (entry.doc.clone(), entry.is_executable)
    };

    let name_str = atom.to_string();

    if let Some(doc_text) = doc {
        if doc_text.trim().is_empty() {
            let output = format!("{}: documentation is empty", name_str);
            let _ = interp.writeln_async(&output).await;
        } else {
            // Split multi-line docs and write each line separately
            let output = format!("{}:", name_str);
            let _ = interp.writeln_async(&output).await;

            // Split by newline and write each line
            for line in doc_text.split('\n') {
                let _ = interp.writeln_async(line).await;
            }
        }
    } else {
        let kind = if is_executable { "word" } else { "constant" };
        let output = format!("{} ({}) has no documentation yet", name_str, kind);
        let _ = interp.writeln_async(&output).await;
    }

    Ok(())
}
