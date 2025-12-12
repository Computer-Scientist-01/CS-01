use anyhow::{Result, bail};
use serde_json::Value;

/// Converts a JSON Object into a Git-compatible INI string.
///
/// NOTE: This implementation specifically handles the 3-level hierarchy of Git config:
/// Section -> Subsection -> Setting.
/// - Top-level keys are Sections.
/// - Second-level keys are Subsections. If the key is empty string "", it represents the Section itself (no subsection).
/// - Third-level keys are the actual Settings (Key-Value pairs).
pub fn obj_to_str(config_obj: &Value) -> Result<String> {
    let obj = config_obj
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Invalid configObj: Must be a non-empty object."))?;

    if obj.is_empty() {
        bail!("Invalid configObj: Must be a non-empty object.");
    };

    let mut output = String::new();

    for (section_name, section_val) in obj {
        let subsections = section_val.as_object().ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid section '{}': Must contain subsection objects.",
                section_name
            )
        })?;

        for (subsection_name, settings_val) in subsections {
            let settings = settings_val.as_object().ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid settings for [{}]: Must be an object.",
                    section_name
                )
            })?;

            // Note: Git config format uses [section "subsection"] syntax.
            // If subsection_name is empty, it formats as [section]
            let quoted_subsection = if subsection_name.is_empty() {
                "".to_string()
            } else {
                format!(" \"{}\"", subsection_name)
            };

            output.push_str(&format!("[{}{}]\n", section_name, quoted_subsection));

            for (key, val) in settings {
                // Critical: We must handle different JSON types to match Git's string expectation.
                // - Objects/Arrays are serialized to JSON strings.
                // - Primitives are converted directly.
                let string_value = if val.is_object() {
                    serde_json::to_string(val)?
                } else if val.is_string() {
                    val.as_str().unwrap().to_string()
                } else {
                    val.to_string()
                };

                output.push_str(&format!("  {} = {}\n", key, string_value));
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_obj_to_str_basic() {
        let config = json!({
            "core": {
                "": {
                    "bare": false
                }
            }
        });
        let result = obj_to_str(&config).unwrap();
        assert!(result.contains("[core]"));
        assert!(result.contains("bare = false"));
    }

    #[test]
    fn test_obj_to_str_nested() {
        let config = json!({
            "remote": {
                "origin": {
                    "url": "https://example.com"
                }
            }
        });
        let result = obj_to_str(&config).unwrap();
        assert!(result.contains("[remote \"origin\"]"));
        assert!(result.contains("url = https://example.com"));
    }

    #[test]
    fn test_obj_to_str_complex_types() {
        let config = json!({
            "user": {
                "": {
                    "id": 123,
                    "active": true
                }
            }
        });
        let result = obj_to_str(&config).unwrap();
        assert!(result.contains("id = 123"));
        assert!(result.contains("active = true"));
    }

    #[test]
    fn test_obj_to_str_invalid_input() {
        // Not an object
        let config = json!([]);
        assert!(obj_to_str(&config).is_err());

        // Empty object
        let config = json!({});
        assert!(obj_to_str(&config).is_err());
    }
}
