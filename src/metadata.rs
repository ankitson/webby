use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{Result, err};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub properties: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetadataOverrides {
    pub title: Option<String>,
    pub description: Option<String>,
    pub properties: BTreeMap<String, Value>,
}

impl MetadataOverrides {
    pub fn is_empty(&self) -> bool {
        self.title.is_none() && self.description.is_none() && self.properties.is_empty()
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawMetadata {
    title: Option<String>,
    description: Option<String>,
    properties: BTreeMap<String, Value>,
    #[serde(flatten)]
    extra_properties: BTreeMap<String, Value>,
}

pub fn read_app_metadata(path: &Path, is_dir: bool) -> Result<AppMetadata> {
    let html_path = if is_dir {
        path.join("index.html")
    } else {
        path.to_path_buf()
    };
    if !html_path.exists() {
        return Ok(AppMetadata::default());
    }

    let html = fs::read_to_string(&html_path)
        .map_err(|e| err(format!("failed to read {}: {e}", html_path.display())))?;
    parse_metadata(&html).map_err(|e| {
        err(format!(
            "failed to parse webby metadata in {}: {e}",
            html_path.display()
        ))
    })
}

pub fn apply_app_metadata_overrides(
    path: &Path,
    is_dir: bool,
    overrides: &MetadataOverrides,
) -> Result<()> {
    if overrides.is_empty() {
        return Ok(());
    }

    let html_path = if is_dir {
        path.join("index.html")
    } else {
        path.to_path_buf()
    };
    if !html_path.exists() {
        return Err(err(format!(
            "cannot write webby metadata because {} does not exist",
            html_path.display()
        )));
    }

    let html = fs::read_to_string(&html_path)
        .map_err(|e| err(format!("failed to read {}: {e}", html_path.display())))?;
    let mut metadata = parse_metadata(&html).map_err(|e| {
        err(format!(
            "failed to parse webby metadata in {}: {e}",
            html_path.display()
        ))
    })?;

    if let Some(title) = &overrides.title {
        metadata.title = clean(Some(title.clone()));
    }
    if let Some(description) = &overrides.description {
        metadata.description = clean(Some(description.clone()));
    }
    for (key, value) in &overrides.properties {
        metadata.properties.insert(key.clone(), value.clone());
    }

    let block = render_metadata_block(&metadata)?;
    let updated = if let Some((start, end)) = find_webby_script_range(&html) {
        format!("{}{}{}", &html[..start], block, &html[end..])
    } else if let Some(head_close) = html.to_ascii_lowercase().find("</head>") {
        format!("{}{}\n{}", &html[..head_close], block, &html[head_close..])
    } else {
        format!("{block}\n{html}")
    };

    fs::write(&html_path, updated)
        .map_err(|e| err(format!("failed to write {}: {e}", html_path.display())))
}

fn parse_metadata(html: &str) -> std::result::Result<AppMetadata, serde_json::Error> {
    let mut metadata = if let Some(json) = extract_webby_json(html) {
        let raw: RawMetadata = serde_json::from_str(json.trim())?;
        let mut properties = raw.properties;
        for (key, value) in raw.extra_properties {
            if !value.is_null() {
                properties.entry(key).or_insert(value);
            }
        }
        AppMetadata {
            title: clean(raw.title),
            description: clean(raw.description),
            properties,
        }
    } else {
        AppMetadata::default()
    };

    if metadata.title.is_none() {
        metadata.title = extract_title(html);
    }
    if metadata.description.is_none() {
        metadata.description = extract_meta_description(html);
    }

    Ok(metadata)
}

fn extract_webby_json(html: &str) -> Option<&str> {
    let (_, open_end, close_start, _) = find_webby_script_parts(html)?;
    Some(&html[open_end + 1..close_start])
}

fn find_webby_script_range(html: &str) -> Option<(usize, usize)> {
    let (start, _, _, close_end) = find_webby_script_parts(html)?;
    Some((start, close_end))
}

fn find_webby_script_parts(html: &str) -> Option<(usize, usize, usize, usize)> {
    let lower = html.to_ascii_lowercase();
    let mut offset = 0;
    while let Some(relative_start) = lower[offset..].find("<script") {
        let start = offset + relative_start;
        let open_end = lower[start..].find('>')? + start;
        let tag = &html[start..=open_end];
        if attr_value(tag, "type")
            .as_deref()
            .map(|value| value.eq_ignore_ascii_case("application/webby+json"))
            .unwrap_or(false)
        {
            let content_start = open_end + 1;
            let close = lower[content_start..].find("</script>")? + content_start;
            let close_end = close + "</script>".len();
            return Some((start, open_end, close, close_end));
        }
        offset = open_end + 1;
    }
    None
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let open_start = lower.find("<title")?;
    let content_start = lower[open_start..].find('>')? + open_start + 1;
    let close = lower[content_start..].find("</title>")? + content_start;
    clean(Some(html_unescape(&html[content_start..close])))
}

fn extract_meta_description(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut offset = 0;
    while let Some(relative_start) = lower[offset..].find("<meta") {
        let start = offset + relative_start;
        let Some(open_end) = lower[start..].find('>').map(|end| end + start) else {
            return None;
        };
        let tag = &html[start..=open_end];
        let is_description = attr_value(tag, "name")
            .as_deref()
            .map(|value| value.eq_ignore_ascii_case("description"))
            .unwrap_or(false);
        if is_description {
            return attr_value(tag, "content").and_then(|value| clean(Some(html_unescape(&value))));
        }
        offset = open_end + 1;
    }
    None
}

fn attr_value(tag: &str, attr_name: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        while index < bytes.len() && !is_attr_start(bytes[index]) {
            index += 1;
        }
        let name_start = index;
        while index < bytes.len() && is_attr_name(bytes[index]) {
            index += 1;
        }
        if name_start == index {
            break;
        }
        let name = &tag[name_start..index];
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] != b'=' {
            continue;
        }
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        let value = if bytes[index] == b'"' || bytes[index] == b'\'' {
            let quote = bytes[index];
            index += 1;
            let value_start = index;
            while index < bytes.len() && bytes[index] != quote {
                index += 1;
            }
            let value = tag[value_start..index].to_string();
            index += usize::from(index < bytes.len());
            value
        } else {
            let value_start = index;
            while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'>'
            {
                index += 1;
            }
            tag[value_start..index].to_string()
        };
        if name.eq_ignore_ascii_case(attr_name) {
            return Some(html_unescape(&value));
        }
    }
    None
}

fn is_attr_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b':' || byte == b'-'
}

fn is_attr_name(byte: u8) -> bool {
    is_attr_start(byte) || byte.is_ascii_digit()
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn render_metadata_block(metadata: &AppMetadata) -> Result<String> {
    let mut object = Map::new();
    if let Some(title) = &metadata.title {
        object.insert("title".to_string(), Value::String(title.clone()));
    }
    if let Some(description) = &metadata.description {
        object.insert(
            "description".to_string(),
            Value::String(description.clone()),
        );
    }
    object.insert(
        "properties".to_string(),
        Value::Object(
            metadata
                .properties
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        ),
    );
    let json = serde_json::to_string_pretty(&Value::Object(object))
        .map_err(|e| err(format!("failed to serialize webby metadata: {e}")))?;
    Ok(format!(
        "<script type=\"application/webby+json\">\n{json}\n</script>"
    ))
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_webby_json_metadata_and_fallbacks() {
        let html = r#"<!doctype html>
<html>
<head>
  <title>Fallback title</title>
  <meta name="description" content="Fallback description">
  <script type="application/webby+json">
  {
    "title": "Custom title",
    "properties": {
      "category": "Documents",
      "rank": 2
    },
    "audience": "internal"
  }
  </script>
</head>
</html>"#;

        let metadata = parse_metadata(html).unwrap();

        assert_eq!(metadata.title.as_deref(), Some("Custom title"));
        assert_eq!(
            metadata.description.as_deref(),
            Some("Fallback description")
        );
        assert_eq!(
            metadata.properties.get("category").and_then(Value::as_str),
            Some("Documents")
        );
        assert_eq!(
            metadata.properties.get("rank").and_then(Value::as_i64),
            Some(2)
        );
        assert_eq!(
            metadata.properties.get("audience").and_then(Value::as_str),
            Some("internal")
        );
    }

    #[test]
    fn falls_back_to_standard_html_metadata() {
        let html = r#"<title>Docs &amp; Notes</title><meta content="A &quot;short&quot; note" name="description">"#;

        let metadata = parse_metadata(html).unwrap();

        assert_eq!(metadata.title.as_deref(), Some("Docs & Notes"));
        assert_eq!(metadata.description.as_deref(), Some("A \"short\" note"));
        assert!(metadata.properties.is_empty());
    }

    #[test]
    fn replaces_existing_metadata_block() {
        let html = r#"<html><head><script type="application/webby+json">
{"properties":{"category":"Old"}}
</script></head></html>"#;
        let mut metadata = parse_metadata(html).unwrap();
        metadata.properties.insert(
            "category".to_string(),
            Value::String("Documents".to_string()),
        );
        metadata.title = Some("Updated".to_string());
        let block = render_metadata_block(&metadata).unwrap();
        let (start, end) = find_webby_script_range(html).unwrap();
        let updated = format!("{}{}{}", &html[..start], block, &html[end..]);

        assert_eq!(
            parse_metadata(&updated)
                .unwrap()
                .properties
                .get("category")
                .and_then(Value::as_str),
            Some("Documents")
        );
        assert_eq!(
            parse_metadata(&updated).unwrap().title.as_deref(),
            Some("Updated")
        );
        assert_eq!(updated.matches("application/webby+json").count(), 1);
    }
}
