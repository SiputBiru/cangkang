use crate::error::CangkangError;

#[derive(Debug, Default)]
pub struct PageMetadata {
    pub title: String,
    pub date: String,
}

pub fn parse(content: &str) -> Result<(PageMetadata, &str), CangkangError> {
    let mut metadata = PageMetadata {
        title: "Untitled".to_string(),
        date: "".to_string(),
    };

    // Check if the file starts with the frontmatter dashes
    if content.starts_with("---\n") || content.starts_with("---\r\n") {
        let end_marker = "\n---";

        // Find where the frontmatter block ends (skipping the first 3 chars)
        if let Some(end_idx) = content[3..].find(end_marker) {
            let json_str = &content[3..end_idx + 3];
            let remaining_content = &content[end_idx + 3 + end_marker.len()..];

            // Extract our specific keys
            if let Some(t) = extract_json_value(json_str, "title") {
                metadata.title = t;
            }
            if let Some(d) = extract_json_value(json_str, "date") {
                metadata.date = d;
            }

            return Ok((metadata, remaining_content.trim_start()));
        } else {
            // If they forgot the closing dashes!
            return Err(CangkangError::Parse {
                message: "Found opening '---' for frontmatter, but no closing '---' found."
                    .to_string(),
                line: 1,
            });
        }
    }

    // If no frontmatter is found, just return defaults and the original text
    Ok((metadata, content))
}

fn extract_json_value(json_str: &str, key: &str) -> Option<String> {
    // Wrap the key in quotes to prevent false positives (e.g., searching for "title")
    let quoted_key = format!("\"{}\"", key);
    let key_idx = json_str.find(&quoted_key)?;

    // Find the colon after the key
    let colon_idx = json_str[key_idx..].find(':')?;
    let search_area = &json_str[key_idx + colon_idx + 1..];

    // Find the opening quote of the value
    let start_quote = search_area.find('"')?;
    let value_area = &search_area[start_quote + 1..];

    // Find the closing quote
    let end_quote = value_area.find('"')?;

    // Slice out everything in between
    Some(value_area[..end_quote].to_string())
}
