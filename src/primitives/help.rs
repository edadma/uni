use crate::compat::{Rc, format, ToString};
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

const IF_DOC: &str = "Conditional branching. Usage: condition true-branch false-branch if";
const EXEC_DOC: &str =
    "Execute the value at the top of the stack. Lists run as code, other values execute directly.";

pub fn help_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
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
            let _ = interp.writeln(&output);
        } else {
            let output = format!("{}:\n{}", name_str, doc_text);
            let _ = interp.writeln(&output);
        }
    } else {
        let kind = if is_executable { "word" } else { "constant" };
        let output = format!("{} ({}) has no documentation yet", name_str, kind);
        let _ = interp.writeln(&output);
    }

    Ok(())
}
