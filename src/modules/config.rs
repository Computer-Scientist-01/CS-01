use anyhow::{Result, bail};
use serde_json::Value;

/// This function takes a JSON object (like a dictionary) and converts it into a string format
/// that looks like a Git configuration file (INI format).
///
/// For example, if you have this JSON:
/// {
///     "core": {
///         "bare": false
///     }
/// }
///
/// It will convert it to:
/// [core]
///   bare = false
pub fn obj_to_str(config_obj: &Value) -> Result<String> {
    // We try to interpret the input as a helper object (dictionary).
    // If it's not a valid object, we return an error.
    let obj = config_obj
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Invalid configObj: Must be a non-empty object."))?;

    // If the object is empty, we also return an error because a config file shouldn't be empty.
    if obj.is_empty() {
        bail!("Invalid configObj: Must be a non-empty object.");
    };

    let mut output = String::new();

    // Loop through each main section of the config (e.g., "core", "remote").
    for (section_name, section_val) in obj {
        // Each section must contain another object (subsections or settings).
        let subsections = section_val.as_object().ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid section '{}': Must contain subsection objects.",
                section_name
            )
        })?;

        // Loop through the subsections inside the main section.
        for (subsection_name, settings_val) in subsections {
            // The settings inside must also be an object (key-value pairs).
            let settings = settings_val.as_object().ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid settings for [{}]: Must be an object.",
                    section_name
                )
            })?;

            // Create the header for this section.
            // If there's a subsection name (like 'origin' in [remote "origin"]), include it.
            // Otherwise, just use the section name (like [core]).
            let quoted_subsection = if subsection_name.is_empty() {
                "".to_string()
            } else {
                format!(" \"{}\"", subsection_name)
            };

            output.push_str(&format!("[{}{}]\n", section_name, quoted_subsection));

            // Write each setting as "key = value"
            for (key, val) in settings {
                // Convert the value to a string.
                // If it's a complicated object, we turn it into a JSON string.
                // If it's a simple string, we just use it.
                // Otherwise (numbers, booleans), we standard conversion.
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
