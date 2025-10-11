// RUST CONCEPT: Date/time operations using chrono
// This module implements date/time functionality similar to JavaScript's Date
// All datetimes are stored as DateTime<FixedOffset> which can represent any timezone

use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use chrono::{DateTime, Datelike, FixedOffset, Local, Offset, TimeZone, Timelike, Utc};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

// RUST CONCEPT: Helper to pop integer and convert to i32
fn pop_i32(interp: &mut Interpreter, context: &str) -> Result<i32, RuntimeError> {
    let val = interp.pop()?;
    match val {
        Value::Int32(i) => Ok(i),
        Value::Integer(i) => i
            .to_i32()
            .ok_or_else(|| RuntimeError::TypeError(format!("{}: integer out of range", context))),
        Value::Number(n) if n.fract() == 0.0 && n.is_finite() => Ok(n as i32),
        _ => Err(RuntimeError::TypeError(format!(
            "{}: expected integer",
            context
        ))),
    }
}

// RUST CONCEPT: Helper to pop integer and convert to u32
fn pop_u32(interp: &mut Interpreter, context: &str) -> Result<u32, RuntimeError> {
    let val = interp.pop()?;
    match val {
        Value::Int32(i) if i >= 0 => Ok(i as u32),
        Value::Integer(i) => i
            .to_u32()
            .ok_or_else(|| RuntimeError::TypeError(format!("{}: integer out of range", context))),
        Value::Number(n) if n.fract() == 0.0 && n >= 0.0 && n.is_finite() => Ok(n as u32),
        _ => Err(RuntimeError::TypeError(format!(
            "{}: expected non-negative integer",
            context
        ))),
    }
}

// RUST CONCEPT: now builtin - get current date/time in local timezone
// Stack: -- datetime
pub fn now_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Local::now() gets current time in system timezone
    let local_now = Local::now();
    // Convert to DateTime<FixedOffset> to store in Value enum
    let dt = local_now.fixed_offset();

    interp.push(Value::DateTime(dt));
    Ok(())
}

// RUST CONCEPT: datetime builtin - create datetime in local timezone
// Stack: year month day hour minute second -- datetime
pub fn datetime_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let second = pop_u32(interp, "datetime")?;
    let minute = pop_u32(interp, "datetime")?;
    let hour = pop_u32(interp, "datetime")?;
    let day = pop_u32(interp, "datetime")?;
    let month = pop_u32(interp, "datetime")?;
    let year = pop_i32(interp, "datetime")?;

    // RUST CONCEPT: Get local timezone offset
    let local_offset = Local::now().offset().fix();

    // RUST CONCEPT: Create datetime with validation
    // with_ymd_and_hms returns a result that handles invalid dates
    let dt = local_offset
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| {
            RuntimeError::DomainError(format!(
                "Invalid date/time: {}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hour, minute, second
            ))
        })?;

    interp.push(Value::DateTime(dt));
    Ok(())
}

// RUST CONCEPT: datetime-with-offset builtin - create datetime with specific offset
// Stack: year month day hour minute second offset-hours -- datetime
pub fn datetime_with_offset_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let offset_hours = pop_i32(interp, "datetime-with-offset")?;
    let second = pop_u32(interp, "datetime-with-offset")?;
    let minute = pop_u32(interp, "datetime-with-offset")?;
    let hour = pop_u32(interp, "datetime-with-offset")?;
    let day = pop_u32(interp, "datetime-with-offset")?;
    let month = pop_u32(interp, "datetime-with-offset")?;
    let year = pop_i32(interp, "datetime-with-offset")?;

    // RUST CONCEPT: Create FixedOffset from hours
    let offset = FixedOffset::east_opt(offset_hours * 3600).ok_or_else(|| {
        RuntimeError::DomainError(format!("Invalid offset: {} hours", offset_hours))
    })?;

    let dt = offset
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| {
            RuntimeError::DomainError(format!(
                "Invalid date/time: {}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hour, minute, second
            ))
        })?;

    interp.push(Value::DateTime(dt));
    Ok(())
}

// RUST CONCEPT: year builtin - extract year from datetime
// Stack: datetime -- year
pub fn year_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => return Err(RuntimeError::TypeError("year: expected datetime".to_string())),
    };

    interp.push(Value::Integer(BigInt::from(dt.year())));
    Ok(())
}

// RUST CONCEPT: month builtin - extract month (1-12) from datetime
// Stack: datetime -- month
pub fn month_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "month: expected datetime".to_string(),
            ))
        }
    };

    interp.push(Value::Int32(dt.month() as i32));
    Ok(())
}

// RUST CONCEPT: day builtin - extract day (1-31) from datetime
// Stack: datetime -- day
pub fn day_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => return Err(RuntimeError::TypeError("day: expected datetime".to_string())),
    };

    interp.push(Value::Int32(dt.day() as i32));
    Ok(())
}

// RUST CONCEPT: hour builtin - extract hour (0-23) from datetime
// Stack: datetime -- hour
pub fn hour_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => return Err(RuntimeError::TypeError("hour: expected datetime".to_string())),
    };

    interp.push(Value::Int32(dt.hour() as i32));
    Ok(())
}

// RUST CONCEPT: minute builtin - extract minute (0-59) from datetime
// Stack: datetime -- minute
pub fn minute_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "minute: expected datetime".to_string(),
            ))
        }
    };

    interp.push(Value::Int32(dt.minute() as i32));
    Ok(())
}

// RUST CONCEPT: second builtin - extract second (0-59) from datetime
// Stack: datetime -- second
pub fn second_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "second: expected datetime".to_string(),
            ))
        }
    };

    interp.push(Value::Int32(dt.second() as i32));
    Ok(())
}

// RUST CONCEPT: weekday builtin - get day of week (0=Monday, 6=Sunday)
// Stack: datetime -- weekday
pub fn weekday_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "weekday: expected datetime".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Weekday enum from chrono
    let weekday = dt.weekday();
    // Convert to number: Monday=0, Tuesday=1, ..., Sunday=6
    let num = weekday.num_days_from_monday();

    interp.push(Value::Int32(num as i32));
    Ok(())
}

// RUST CONCEPT: timestamp builtin - convert datetime to Unix timestamp (seconds since epoch)
// Stack: datetime -- timestamp
pub fn timestamp_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "timestamp: expected datetime".to_string(),
            ))
        }
    };

    let ts = dt.timestamp();
    interp.push(Value::Integer(BigInt::from(ts)));
    Ok(())
}

// RUST CONCEPT: timestamp->datetime builtin - convert Unix timestamp to datetime
// Stack: timestamp -- datetime
pub fn timestamp_to_datetime_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let ts = pop_i32(interp, "timestamp->datetime")? as i64;

    // RUST CONCEPT: Create datetime from timestamp in local timezone
    let utc_dt = DateTime::from_timestamp(ts, 0)
        .ok_or_else(|| RuntimeError::DomainError("Invalid timestamp".to_string()))?;

    // Convert to local timezone
    let local_offset = Local::now().offset().fix();
    let local_dt = utc_dt.with_timezone(&local_offset);

    interp.push(Value::DateTime(local_dt));
    Ok(())
}

// RUST CONCEPT: to-utc builtin - convert datetime to UTC
// Stack: datetime -- datetime
pub fn to_utc_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "to-utc: expected datetime".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Convert to UTC and back to FixedOffset
    let utc_dt = dt.with_timezone(&Utc);
    let utc_fixed = utc_dt.fixed_offset();

    interp.push(Value::DateTime(utc_fixed));
    Ok(())
}

// RUST CONCEPT: to-local builtin - convert datetime to local timezone
// Stack: datetime -- datetime
pub fn to_local_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "to-local: expected datetime".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Convert to local timezone
    let local_offset = Local::now().offset().fix();
    let local_dt = dt.with_timezone(&local_offset);

    interp.push(Value::DateTime(local_dt));
    Ok(())
}

// RUST CONCEPT: datetime->string builtin - format datetime as ISO 8601 string
// Stack: datetime -- string
pub fn datetime_to_string_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "datetime->string: expected datetime".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Use RFC3339 format (ISO 8601 with timezone)
    let s = dt.to_rfc3339();
    interp.push(Value::String(s.into()));
    Ok(())
}

// RUST CONCEPT: string->datetime builtin - parse ISO 8601 string to datetime
// Stack: string -- datetime
pub fn string_to_datetime_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let s = match interp.pop()? {
        Value::String(s) => s,
        _ => {
            return Err(RuntimeError::TypeError(
                "string->datetime: expected string".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Parse RFC3339/ISO 8601 format
    let dt = DateTime::parse_from_rfc3339(&s).map_err(|e| {
        RuntimeError::DomainError(format!("Failed to parse datetime: {}", e))
    })?;

    interp.push(Value::DateTime(dt));
    Ok(())
}

// RUST CONCEPT: date< builtin - compare datetimes (less than)
// Stack: datetime1 datetime2 -- boolean
pub fn date_less_than_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt2 = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "date<: expected datetime".to_string(),
            ))
        }
    };

    let dt1 = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "date<: expected datetime".to_string(),
            ))
        }
    };

    interp.push(Value::Boolean(dt1 < dt2));
    Ok(())
}

// RUST CONCEPT: date> builtin - compare datetimes (greater than)
// Stack: datetime1 datetime2 -- boolean
pub fn date_greater_than_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt2 = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "date>: expected datetime".to_string(),
            ))
        }
    };

    let dt1 = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "date>: expected datetime".to_string(),
            ))
        }
    };

    interp.push(Value::Boolean(dt1 > dt2));
    Ok(())
}

// RUST CONCEPT: date= builtin - compare datetimes (equal)
// Stack: datetime1 datetime2 -- boolean
pub fn date_equal_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let dt2 = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "date=: expected datetime".to_string(),
            ))
        }
    };

    let dt1 = match interp.pop()? {
        Value::DateTime(dt) => dt,
        _ => {
            return Err(RuntimeError::TypeError(
                "date=: expected datetime".to_string(),
            ))
        }
    };

    interp.push(Value::Boolean(dt1 == dt2));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::execute_string;

    #[test]
    fn test_now() {
        let mut interp = Interpreter::new();
        now_builtin(&mut interp).unwrap();

        let dt = interp.pop().unwrap();
        assert!(matches!(dt, Value::DateTime(_)));
    }

    #[test]
    fn test_datetime_creation() {
        let mut interp = Interpreter::new();
        execute_string("2025 10 1 14 30 0 datetime", &mut interp).unwrap();

        let dt = interp.pop().unwrap();
        assert!(matches!(dt, Value::DateTime(_)));
    }

    #[test]
    fn test_datetime_accessors() {
        let mut interp = Interpreter::new();
        execute_string("2025 10 1 14 30 45 datetime", &mut interp).unwrap();

        // Test year
        execute_string("dup year", &mut interp).unwrap();
        let year = interp.pop().unwrap();
        assert!(matches!(year, Value::Integer(ref i) if i == &BigInt::from(2025)));

        // Test month
        execute_string("dup month", &mut interp).unwrap();
        let month = interp.pop().unwrap();
        assert!(matches!(month, Value::Integer(ref i) if i == &BigInt::from(10)));

        // Test day
        execute_string("dup day", &mut interp).unwrap();
        let day = interp.pop().unwrap();
        assert!(matches!(day, Value::Integer(ref i) if i == &BigInt::from(1)));

        // Test hour
        execute_string("dup hour", &mut interp).unwrap();
        let hour = interp.pop().unwrap();
        assert!(matches!(hour, Value::Integer(ref i) if i == &BigInt::from(14)));

        // Test minute
        execute_string("dup minute", &mut interp).unwrap();
        let minute = interp.pop().unwrap();
        assert!(matches!(minute, Value::Integer(ref i) if i == &BigInt::from(30)));

        // Test second
        execute_string("second", &mut interp).unwrap();
        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Integer(ref i) if i == &BigInt::from(45)));
    }

    #[test]
    fn test_datetime_with_offset() {
        let mut interp = Interpreter::new();
        execute_string("2025 10 1 14 30 0 -5 datetime-with-offset", &mut interp).unwrap();

        let dt = interp.pop().unwrap();
        assert!(matches!(dt, Value::DateTime(_)));
    }

    #[test]
    fn test_date_comparisons() {
        let mut interp = Interpreter::new();

        // Create two datetimes
        execute_string("2025 10 1 14 30 0 datetime", &mut interp).unwrap();
        execute_string("2025 10 2 14 30 0 datetime", &mut interp).unwrap();

        // Test date< (Oct 1 < Oct 2)
        execute_string("over over date<", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test date> (Oct 1 > Oct 2)
        execute_string("over over date>", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Clean up stack
        interp.pop().unwrap();
        interp.pop().unwrap();
    }
}
