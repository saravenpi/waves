use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    if total_secs == 0 {
        return "-:--".to_string();
    }
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{}:{:02}", minutes, seconds)
}

pub fn format_duration_option(duration: Option<Duration>) -> String {
    match duration {
        Some(d) if d.as_secs() > 0 => format_duration(d),
        _ => "-:--".to_string(),
    }
}

pub fn truncate_text(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}
