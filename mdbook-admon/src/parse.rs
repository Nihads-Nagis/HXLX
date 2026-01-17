// parse.rs - Fixed with proper body extraction
use anyhow::{anyhow, Result};

use crate::types::{
    AdmonishConfig, AdmonitionMeta, BuiltinDirective, CustomDirective, DirectiveMap,
};

/// Parse admonition from raw code block (including fences)
pub fn parse_admonition(
    info_string: &str,
    raw_block: &str,
    config: &AdmonishConfig,
    directive_map: &DirectiveMap,
) -> Option<Result<AdmonitionMeta>> {
    // Extract just the directive part from info string
    let directive_part = if info_string == "admonish" {
        ""
    } else if info_string.starts_with("admonish ") {
        &info_string[9..] // Skip "admonish "
    } else {
        // Not an admonition
        return None;
    };

    // Parse configuration
    let parsed = match parse_directive_config(directive_part) {
        Ok(p) => p,
        Err(e) => return Some(Err(e)),
    };

    // Extract body content
    let body = extract_admonition_body(raw_block);

    // Resolve metadata
    Some(resolve_metadata(parsed, &body, config, directive_map))
}

/// Extract body from code block (removes fences)
fn extract_admonition_body(content: &str) -> String {
    let mut lines = content.lines();

    // Drop opening fence line
    lines.next();

    // Collect remaining lines, removing closing fence if present
    let mut body_lines: Vec<&str> = lines.collect();
    if let Some(last) = body_lines.last() {
        let trimmed = last.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            body_lines.pop();
        }
    }

    body_lines.join("\n")
}

/// Parse directive configuration from string
fn parse_directive_config(info_str: &str) -> Result<ParsedConfig> {
    let mut config = ParsedConfig::default();

    if info_str.is_empty() {
        return Ok(config);
    }

    // Simple parsing: support both v2 and v3 style
    let parts: Vec<&str> = info_str.split_whitespace().collect();
    let mut i = 0;

    // First part could be directive without key=
    if !parts.is_empty() && !parts[0].contains('=') {
        config.directive = Some(parts[0].to_string());
        i += 1;
    }

    // Parse key=value pairs
    while i < parts.len() {
        let part = parts[i];
        if let Some((key, value)) = part.split_once('=') {
            let value = value.trim_matches('"');

            match key {
                "title" => config.title = Some(value.to_string()),
                "id" => config.id = Some(value.to_string()),
                "type" => config.directive = Some(value.to_string()),
                "collapsible" => config.collapsible = Some(value == "true"),
                "class" => {
                    config
                        .classnames
                        .extend(value.split_whitespace().map(|s| s.to_string()));
                }
                _ => {}
            }
        } else if part.starts_with('.') {
            // Class name in v1 style
            config.classnames.push(part[1..].to_string());
        }
        i += 1;
    }

    Ok(config)
}

#[derive(Debug, Default)]
struct ParsedConfig {
    directive: Option<String>,
    title: Option<String>,
    id: Option<String>,
    classnames: Vec<String>,
    collapsible: Option<bool>,
}

/// Resolve final admonition metadata
fn resolve_metadata(
    parsed: ParsedConfig,
    content: &str,
    config: &AdmonishConfig,
    directive_map: &DirectiveMap,
) -> Result<AdmonitionMeta> {
    let directive_name = parsed.directive.unwrap_or_else(|| "note".to_string());

    // Determine if directive is built-in or custom
    let (directive, color, icon, custom_collapsible) =
        if let Ok(builtin) = BuiltinDirective::from_str(&directive_name) {
            // Built-in directive - get color from BUILTIN_DIRECTIVES constant
            let color = crate::types::BUILTIN_DIRECTIVES
                .iter()
                .find(|(name, _)| *name == builtin.to_string())
                .map(|(_, color)| color.to_string())
                .unwrap_or_else(|| "#448aff".to_string());

            (builtin.to_string(), Some(color), None, None)
        } else if let Some(custom) = directive_map.get(&directive_name) {
            // Custom directive
            (
                directive_name,
                custom.color.clone(),
                custom.icon.clone(),
                custom.collapsible,
            )
        } else {
            // Unknown directive, default to note
            ("note".to_string(), Some("#448aff".to_string()), None, None)
        };

    // Determine title
    let title = parsed
        .title
        .or_else(|| directive_map.get(&directive).and_then(|c| c.title.clone()))
        .or_else(|| config.default_title.clone())
        .unwrap_or_else(|| {
            // Capitalize directive name
            let mut chars = directive.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        });

    // Determine collapsible state
    let collapsible = parsed
        .collapsible
        .or(custom_collapsible)
        .unwrap_or(config.default_collapsible);

    // Generate ID
    let id = parsed.id.or_else(|| {
        let slug = title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
            .replace(' ', "-");
        Some(format!("{}{}", config.css_id_prefix, slug))
    });

    Ok(AdmonitionMeta {
        directive,
        title,
        id,
        classnames: parsed.classnames,
        collapsible,
        color,
        icon,
    })
}
