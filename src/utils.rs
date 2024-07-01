use anyhow::Result;
use time::{macros::format_description, Date};

/**
 * Used to ask user for confirmation of action
 */
pub fn ask_permission(message: &str, force: bool) -> Result<bool> {
    if force {
        return Ok(true);
    }

    println!("{}", message);

    let mut input = String::new();

    std::io::stdin().read_line(&mut input)?;

    let trimmed_input = input.trim().to_lowercase();

    match trimmed_input.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => {
            return Err(
                anyhow::anyhow!("Invalid input. Please enter 'y', 'n', 'Y', or 'N'.").into(),
            )
        }
    }
}

pub fn get_date_format() -> &'static [time::format_description::BorrowedFormatItem<'static>] {
    format_description!("[year]-[month]-[day]")
}

pub fn date_parser(value: &str) -> Result<Date> {
    let format = get_date_format();

    match Date::parse(&value, &format) {
        Ok(date) => Ok(date),
        Err(_) => Err(anyhow::anyhow!(
            "[Invalid date format ] - [input: {}] - [expected: YYYY-MM-DD]",
            value
        )),
    }
}

pub fn optional_date_parser(value: &str) -> Result<Option<Date>> {
    if value == "" {
        return Ok(None);
    }
    Ok(Some(date_parser(value)?))
}

/**
 * Used to convert string "NOW" to Date struct
 */
pub fn created_at_parser(value: &str) -> Result<Date> {
    use std::time::SystemTime;
    use time::OffsetDateTime;

    if value.eq("NOW") {
        // Get current time
        let now = SystemTime::now();
        let now = OffsetDateTime::from(now).date();
        Ok(now)
    } else {
        date_parser(value)
    }
}

pub fn title_parser(value: &str) -> Result<String> {
    let function = string_len_parser(1, 1000);
    function(value)
}

pub fn category_parser(value: &str) -> Result<String> {
    let function = string_len_parser(1, 200);
    function(value)
}

pub fn info_parser(value: &str) -> Result<String> {
    let function = string_len_parser(0, 10000);
    function(value)
}

pub fn string_len_parser(
    low: usize,
    high: usize,
) -> impl Fn(&str) -> Result<std::string::String, anyhow::Error> {
    return move |value: &str| -> Result<String> {
        let len = value.len();

        if len < low || len > high {
            Err(anyhow::anyhow!(
                "Field not withing bounds [{}, {}]",
                low,
                high
            ))
        } else {
            Ok(value.to_string())
        }
    };
}
