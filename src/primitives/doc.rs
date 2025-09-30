use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

pub fn doc_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let doc_text = match value {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError(
                "doc expects a string value".to_string(),
            ));
        }
    };

    let target = interp
        .take_pending_doc_target()
        .ok_or_else(|| RuntimeError::TypeError("doc must follow def or val".to_string()))?;

    interp.attach_doc(&target, doc_text)?;
    Ok(())
}
