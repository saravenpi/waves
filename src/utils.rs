use std::time::Duration;

/// Formats a Duration as a human-readable time string.
///
/// # Arguments
/// * `duration` - Duration to format
/// # Returns
/// String in format "M:SS" (e.g., "3:45")
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{}:{:02}", minutes, seconds)
}

/// Truncates text to a maximum character count and adds ellipsis.
///
/// Properly handles multi-byte UTF-8 characters by counting and slicing at character boundaries.
/// # Arguments
/// * `text` - Text to truncate
/// * `max_chars` - Maximum number of characters before truncation
/// # Returns
/// Original text if short enough, otherwise truncated text with "..." suffix
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}
