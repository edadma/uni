// RUST CONCEPT: Duration operations using chrono
// Durations represent time spans (positive or negative)

use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use chrono::Duration as ChronoDuration;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

// RUST CONCEPT: Helper to pop integer and convert to i64
fn pop_i64(interp: &mut Interpreter, context: &str) -> Result<i64, RuntimeError> {
    let val = interp.pop()?;
    match val {
        Value::Int32(i) => Ok(i as i64),
        Value::Integer(i) => i
            .to_i64()
            .ok_or_else(|| RuntimeError::TypeError(format!("{}: integer out of range", context))),
        Value::Number(n) if n.fract() == 0.0 && n.is_finite() => Ok(n as i64),
        _ => Err(RuntimeError::TypeError(format!(
            "{}: expected integer",
            context
        ))),
    }
}

// RUST CONCEPT: duration builtin - create duration from components
// Stack: days hours minutes seconds -- duration
pub fn duration_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let seconds = pop_i64(interp, "duration")?;
    let minutes = pop_i64(interp, "duration")?;
    let hours = pop_i64(interp, "duration")?;
    let days = pop_i64(interp, "duration")?;

    // RUST CONCEPT: Build duration from components
    // chrono::Duration supports construction from various units
    let duration = ChronoDuration::days(days)
        + ChronoDuration::hours(hours)
        + ChronoDuration::minutes(minutes)
        + ChronoDuration::seconds(seconds);

    interp.push(Value::Duration(duration));
    Ok(())
}

// RUST CONCEPT: duration->seconds builtin - convert duration to total seconds
// Stack: duration -- seconds
pub fn duration_to_seconds_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dur = match interp.pop()? {
        Value::Duration(d) => d,
        _ => {
            return Err(RuntimeError::TypeError(
                "duration->seconds: expected duration".to_string(),
            ))
        }
    };

    let secs = dur.num_seconds();
    interp.push(Value::Integer(BigInt::from(secs)));
    Ok(())
}

// RUST CONCEPT: date+ builtin - add duration to datetime
// Stack: datetime duration -- datetime
pub fn date_add_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dur = match interp.pop()? {
        Value::Duration(d) => d,
        _ => {
            return Err(RuntimeError::TypeError(
                "date+: expected duration".to_string(),
            ))
        }
    };

    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "date+: expected datetime".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Add duration to datetime
    // checked_add returns Option to handle overflow
    let new_dt = dt
        .checked_add_signed(dur)
        .ok_or_else(|| RuntimeError::DomainError("Date overflow".to_string()))?;

    interp.push(Value::DateTime(new_dt));
    Ok(())
}

// RUST CONCEPT: date- builtin - subtract datetime or duration
// Stack: datetime1 datetime2 -- duration (returns time between two datetimes)
// Stack: datetime duration -- datetime (subtracts duration from datetime)
pub fn date_sub_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let second = interp.pop()?;
    let first = interp.pop()?;

    match (first, second) {
        // datetime - datetime = duration
        (Value::DateTime(dt1), Value::DateTime(dt2)) => {
            // RUST CONCEPT: Subtract datetimes to get duration
            let duration = dt1.signed_duration_since(dt2);
            interp.push(Value::Duration(duration));
            Ok(())
        }
        // datetime - duration = datetime
        (Value::DateTime(dt), Value::Duration(dur)) => {
            let new_dt = dt
                .checked_sub_signed(dur)
                .ok_or_else(|| RuntimeError::DomainError("Date underflow".to_string()))?;
            interp.push(Value::DateTime(new_dt));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "date-: expected (datetime datetime) or (datetime duration)".to_string(),
        )),
    }
}

// RUST CONCEPT: duration< builtin - compare durations
// Stack: duration1 duration2 -- boolean
pub fn duration_less_than_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dur2 = match interp.pop()? {
        Value::Duration(d) => d,
        _ => {
            return Err(RuntimeError::TypeError(
                "duration<: expected duration".to_string(),
            ))
        }
    };

    let dur1 = match interp.pop()? {
        Value::Duration(d) => d,
        _ => {
            return Err(RuntimeError::TypeError(
                "duration<: expected duration".to_string(),
            ))
        }
    };

    interp.push(Value::Boolean(dur1 < dur2));
    Ok(())
}

// RUST CONCEPT: duration> builtin - compare durations
// Stack: duration1 duration2 -- boolean
pub fn duration_greater_than_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dur2 = match interp.pop()? {
        Value::Duration(d) => d,
        _ => {
            return Err(RuntimeError::TypeError(
                "duration>: expected duration".to_string(),
            ))
        }
    };

    let dur1 = match interp.pop()? {
        Value::Duration(d) => d,
        _ => {
            return Err(RuntimeError::TypeError(
                "duration>: expected duration".to_string(),
            ))
        }
    };

    interp.push(Value::Boolean(dur1 > dur2));
    Ok(())
}

// RUST CONCEPT: duration= builtin - compare durations
// Stack: duration1 duration2 -- boolean
pub fn duration_equal_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dur2 = match interp.pop()? {
        Value::Duration(d) => d,
        _ => {
            return Err(RuntimeError::TypeError(
                "duration=: expected duration".to_string(),
            ))
        }
    };

    let dur1 = match interp.pop()? {
        Value::Duration(d) => d,
        _ => {
            return Err(RuntimeError::TypeError(
                "duration=: expected duration".to_string(),
            ))
        }
    };

    interp.push(Value::Boolean(dur1 == dur2));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::execute_string;

    #[test]
    fn test_duration_creation() {
        let mut interp = Interpreter::new();
        execute_string("1 2 30 45 duration", &mut interp).unwrap();

        let dur = interp.pop().unwrap();
        assert!(matches!(dur, Value::Duration(_)));
    }

    #[test]
    fn test_duration_to_seconds() {
        let mut interp = Interpreter::new();
        // 1 day + 1 hour = 25 hours = 90000 seconds
        execute_string("1 1 0 0 duration duration->seconds", &mut interp).unwrap();

        let secs = interp.pop().unwrap();
        assert!(matches!(secs, Value::Integer(ref i) if i == &BigInt::from(90000)));
    }

    #[test]
    fn test_date_add() {
        let mut interp = Interpreter::new();
        execute_string("2025 10 1 12 0 0 datetime", &mut interp).unwrap();
        execute_string("1 0 0 0 duration", &mut interp).unwrap();
        execute_string("date+", &mut interp).unwrap();

        // Should be October 2, 2025
        execute_string("day", &mut interp).unwrap();
        let day = interp.pop().unwrap();
        assert!(matches!(day, Value::Int32(2)));
    }

    #[test]
    fn test_date_sub_datetime() {
        let mut interp = Interpreter::new();
        execute_string("2025 10 2 12 0 0 datetime", &mut interp).unwrap();
        execute_string("2025 10 1 12 0 0 datetime", &mut interp).unwrap();
        execute_string("date-", &mut interp).unwrap();

        // Should be 1 day = 86400 seconds
        execute_string("duration->seconds", &mut interp).unwrap();
        let secs = interp.pop().unwrap();
        assert!(matches!(secs, Value::Integer(ref i) if i == &BigInt::from(86400)));
    }

    #[test]
    fn test_date_sub_duration() {
        let mut interp = Interpreter::new();
        execute_string("2025 10 2 12 0 0 datetime", &mut interp).unwrap();
        execute_string("1 0 0 0 duration", &mut interp).unwrap();
        execute_string("date-", &mut interp).unwrap();

        // Should be October 1, 2025
        execute_string("day", &mut interp).unwrap();
        let day = interp.pop().unwrap();
        assert!(matches!(day, Value::Int32(1)));
    }

    #[test]
    fn test_duration_comparisons() {
        let mut interp = Interpreter::new();

        // Create two durations: 1 day and 2 days
        execute_string("1 0 0 0 duration", &mut interp).unwrap();
        execute_string("2 0 0 0 duration", &mut interp).unwrap();

        // Test duration< (1 day < 2 days)
        execute_string("over over duration<", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Clean up stack
        interp.pop().unwrap();
        interp.pop().unwrap();
    }
}
