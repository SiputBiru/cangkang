use crate::error::CangkangError;
use crate::models::PageMetadata;

pub fn parse(content: &str) -> Result<(PageMetadata, &str), CangkangError> {
    let mut metadata = PageMetadata {
        title: "Untitled".to_string(),
        date: "".to_string(),
        description: "".to_string(),
        keywords: "".to_string(),
        pinned: false,
        draft: true,
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
            if let Some(desc) = extract_json_value(json_str, "description") {
                metadata.description = desc;
            }
            if let Some(k) = extract_json_value(json_str, "keywords") {
                metadata.keywords = k;
            }

            metadata.pinned = extract_boolean_value(json_str, "pinned").unwrap_or(false);
            metadata.draft = extract_boolean_value(json_str, "draft").unwrap_or(true);

            return Ok((metadata, remaining_content.trim_start()));
        } else {
            // If they forgot the closing dashes!
            return Err(CangkangError::Frontmatter(
                "Found opening '---' for frontmatter, but no closing '---' found.".to_string(),
            ));
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

fn extract_boolean_value(json_str: &str, key: &str) -> Option<bool> {
    let quoted_key = format!("\"{}\"", key);

    json_str
        .find(&quoted_key)
        .and_then(|k_idx| json_str[k_idx..].find(':').map(|c_idx| k_idx + c_idx + 1))
        .map(|start| {
            let search_area = &json_str[start..];
            let end_idx = search_area
                .find(',')
                .or_else(|| search_area.find('}'))
                .unwrap_or(search_area.len());
            search_area[..end_idx].trim() == "true"
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_draft() {
        let content = "---\n{\"title\": \"Test\"}\n---\nContent";
        let (metadata, _) = parse(content).unwrap();
        assert!(metadata.draft);
    }

    #[test]
    fn test_parse_explicit_false_draft() {
        let content = "---\n{\"title\": \"Test\", \"draft\": false}\n---\nContent";
        let (metadata, _) = parse(content).unwrap();
        assert!(!metadata.draft);
    }

    #[test]
    fn test_parse_explicit_true_draft() {
        let content = "---\n{\"title\": \"Test\", \"draft\": true}\n---\nContent";
        let (metadata, _) = parse(content).unwrap();
        assert!(metadata.draft);
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "Just content";
        let (metadata, _) = parse(content).unwrap();
        assert!(metadata.draft); // Default metadata has draft: true
        assert_eq!(metadata.title, "Untitled");
    }
}
