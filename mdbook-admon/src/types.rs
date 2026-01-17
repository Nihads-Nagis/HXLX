// types.rs - Enhanced with custom styling
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Book-wide configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdmonishConfig {
    #[serde(default)]
    pub default_title: Option<String>,

    #[serde(default)]
    pub default_collapsible: bool,

    #[serde(default)]
    pub css_id_prefix: String,

    #[serde(default)]
    pub custom: HashMap<String, CustomDirective>,
}

/// Custom directive with icon and color
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomDirective {
    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub icon: Option<String>,

    #[serde(default)]
    pub color: Option<String>,

    #[serde(default)]
    pub aliases: Vec<String>,

    #[serde(default)]
    pub collapsible: Option<bool>,
}

/// Supported built-in directives
#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum BuiltinDirective {
    Note,
    Info,
    Tip,
    Warning,
    Danger,
    Success,
    Question,
    Bug,
    Example,
}
pub const BUILTIN_DIRECTIVES: &[(&str, &str)] = &[
    ("note", "#448aff"),
    ("info", "#00b0ff"),
    ("tip", "#00bfa5"),
    ("warning", "#ff9100"),
    ("danger", "#ff5252"),
    ("success", "#00c853"),
    ("question", "#7c4dff"),
    ("bug", "#ff4081"),
    ("example", "#651fff"),
];
impl std::str::FromStr for BuiltinDirective {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "note" => Ok(Self::Note),
            "info" | "todo" => Ok(Self::Info),
            "tip" | "hint" => Ok(Self::Tip),
            "warning" | "caution" => Ok(Self::Warning),
            "danger" | "error" => Ok(Self::Danger),
            "success" | "check" => Ok(Self::Success),
            "question" | "help" => Ok(Self::Question),
            "bug" => Ok(Self::Bug),
            "example" => Ok(Self::Example),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for BuiltinDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Note => "note",
                Self::Info => "info",
                Self::Tip => "tip",
                Self::Warning => "warning",
                Self::Danger => "danger",
                Self::Success => "success",
                Self::Question => "question",
                Self::Bug => "bug",
                Self::Example => "example",
            }
        )
    }
}

/// Admonition metadata for rendering
#[derive(Debug)]
pub struct AdmonitionMeta {
    pub directive: String,
    pub title: String,
    pub id: Option<String>,
    pub classnames: Vec<String>,
    pub collapsible: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// Directive lookup map for custom directives
#[derive(Debug, Clone, Default)]
pub struct DirectiveMap {
    inner: HashMap<String, CustomDirective>,
}

impl DirectiveMap {
    pub fn new(custom: HashMap<String, CustomDirective>) -> Self {
        let mut inner = HashMap::new();

        for (name, config) in custom {
            // Add primary name
            inner.insert(name.clone(), config.clone());

            // Add aliases
            for alias in &config.aliases {
                inner.insert(alias.clone(), config.clone());
            }
        }

        Self { inner }
    }

    pub fn get(&self, key: &str) -> Option<&CustomDirective> {
        self.inner.get(key)
    }
}
