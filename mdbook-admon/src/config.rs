// config.rs - Enhanced configuration parsing
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{AdmonishConfig, CustomDirective};

/// Parse configuration from mdBook context
pub fn parse_config(ctx: &mdbook::preprocess::PreprocessorContext) -> Result<AdmonishConfig> {
    let config: AdmonishConfig = ctx
        .config
        .get_preprocessor("admonish")
        .map(|v| v.clone().try_into())
        .unwrap_or(Ok(AdmonishConfig::default()))?;

    Ok(config)
}

/// Parse directive configuration from info string
#[derive(Debug, Default)]
pub struct DirectiveConfig {
    pub directive: String,
    pub title: Option<String>,
    pub id: Option<String>,
    pub classnames: Vec<String>,
    pub collapsible: Option<bool>,
}

/// Parse TOML-like key=value pairs from info string
pub fn parse_directive_string(info_string: &str) -> Result<DirectiveConfig> {
    let mut config = DirectiveConfig::default();

    // Split into parts
    let mut parts = info_string.split_whitespace();

    // First part is the directive (required)
    if let Some(directive) = parts.next() {
        config.directive = directive.to_string();
    }

    // Parse key=value pairs
    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            let value = value.trim_matches('"');

            match key {
                "title" => config.title = Some(value.to_string()),
                "id" => config.id = Some(value.to_string()),
                "class" => {
                    config.classnames = value.split_whitespace().map(|s| s.to_string()).collect();
                }
                "collapsible" => {
                    config.collapsible = Some(value == "true");
                }
                "type" => {
                    config.directive = value.to_string();
                }
                _ => {} // Ignore unknown keys
            }
        } else if !part.contains('=') {
            // If no equals sign, treat as class name
            config.classnames.push(part.to_string());
        }
    }

    Ok(config)
}
