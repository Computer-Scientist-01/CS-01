use anyhow::{Result, bail};
use serde_json::Value;

/// Converts a nested config object to an `INI-like string` suitable for Git config.
pub fn obj_to_str(config_obj: &Value) -> Result<String> {
    let obj = config_obj
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Invalid configObj: Must be a non-empty object."))?;

    if obj.is_empty() {
        bail!("Invalid configObj: Must be a non-empty object.");
    };

    let mut output = String::new();

    // Iterate over sections
    for (section_name, section_val) in obj {
        let subsections = section_val.as_object().ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid section '{}': Must contain subsection objects.",
                section_name
            )
        })?;

        // Iterate over subsections
        for (subsection_name, settings_val) in subsections {
            let settings = settings_val.as_object().ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid settings for [{}]: Must be an object.",
                    section_name
                )
            })?;

            // Compose section header
            let quoted_subsection = if subsection_name.is_empty() {
                "".to_string()
            } else {
                format!(" \"{}\"", subsection_name)
            };

            output.push_str(&format!("[{}{}]\n", section_name, quoted_subsection));

            // Generate key=value lines
            for (key, val) in settings {
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
