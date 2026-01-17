// render.rs - Fixed with helper function and simplified styles
use crate::types::AdmonitionMeta;

/// Render title with icon
fn render_title(meta: &AdmonitionMeta) -> String {
    let icon = meta.icon.as_ref().map_or(String::new(), |i| {
        format!(
            r#"<span class="admonition-icon" style="background-image: url('{}');"></span>"#,
            i
        )
    });

    format!(
        r#"{icon}<span class="admonition-title-text">{}</span>
    <a href="#{}" class="admonition-anchor" aria-hidden="true">#</a>"#,
        meta.title,
        meta.id.as_deref().unwrap_or("")
    )
}

/// Render admonition as HTML
pub fn render_admonition(meta: &AdmonitionMeta, content: &str) -> String {
    let body = extract_admonition_body(content);
    let id = meta.id.as_deref().unwrap_or("admonition");

    // Build CSS classes
    let mut classes = vec![
        "admonition".to_string(),
        format!("admonition-{}", meta.directive),
    ];
    classes.extend(meta.classnames.iter().cloned());

    // Build style attributes (only CSS custom property)
    let style = meta
        .color
        .as_ref()
        .map(|color| format!("--admonition-color: {};", color))
        .unwrap_or_default();

    // Build title HTML
    let title_html = if !meta.title.is_empty() {
        format!(
            r#"<div class="admonition-title">{}</div>"#,
            render_title(meta)
        )
    } else {
        String::new()
    };

    // Determine wrapper element for collapsible
    if meta.collapsible {
        format!(
            r#"<details id="{}" class="{}" style="{}">
<summary class="admonition-title">{}</summary>
<div class="admonition-content">

{}

</div>
</details>"#,
            id,
            classes.join(" "),
            style,
            meta.title,
            body.trim()
        )
    } else {
        format!(
            r#"<div id="{}" class="{}" style="{}">
{}
<div class="admonition-content">

{}

</div>
</div>"#,
            id,
            classes.join(" "),
            style,
            title_html,
            body.trim()
        )
    }
}

/// Generate CSS for admonitions
pub fn generate_custom_css(config: &crate::types::AdmonishConfig) -> String {
    let mut css = String::from(
        r#"/* Admonition CSS - Auto-generated */
:root {
    --admonition-border-color: rgba(0, 0, 0, 0.1);
    --admonition-background: rgba(0, 0, 0, 0.02);
}

.admonition {
    border: 1px solid var(--admonition-border-color);
    border-left: 4px solid var(--admonition-color, #448aff);
    border-radius: 4px;
    margin: 1.5rem 0;
    background: var(--admonition-background);
    overflow: hidden;
}

.admonition-title {
    padding: 0.75rem 1rem;
    font-weight: 600;
    border-bottom: 1px solid var(--admonition-border-color);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: rgba(0, 0, 0, 0.03);
}

.admonition-title-text {
    flex: 1;
}

.admonition-icon {
    width: 1.2em;
    height: 1.2em;
    background-size: contain;
    background-repeat: no-repeat;
    background-position: center;
    opacity: 0.8;
}

.admonition-content {
    padding: 1rem;
}

.admonition-content > *:first-child {
    margin-top: 0;
}

.admonition-content > *:last-child {
    margin-bottom: 0;
}

.admonition-anchor {
    opacity: 0;
    text-decoration: none;
    color: inherit;
    margin-left: 0.5rem;
    font-size: 0.9em;
}

.admonition:hover .admonition-anchor {
    opacity: 0.5;
}

.admonition:hover .admonition-anchor:hover {
    opacity: 1;
}

/* Collapsible styles */
details.admonition > summary.admonition-title {
    cursor: pointer;
    list-style: none;
    position: relative;
    padding-right: 2.5rem;
}

details.admonition > summary.admonition-title::-webkit-details-marker {
    display: none;
}

details.admonition > summary.admonition-title::after {
    content: "▼";
    position: absolute;
    right: 1rem;
    top: 50%;
    transform: translateY(-50%) rotate(0deg);
    transition: transform 0.2s;
    font-size: 0.8em;
    opacity: 0.6;
}

details.admonition[open] > summary.admonition-title::after {
    transform: translateY(-50%) rotate(180deg);
}
"#,
    );

    // Add custom directive styles
    for (name, directive) in &config.custom {
        if let (Some(color), Some(icon)) = (&directive.color, &directive.icon) {
            let bg_color = format!("{}20", color);
            css.push_str(&format!(
                r#".admonition-{} {{
    --admonition-color: {};
}}

.admonition-{} .admonition-title {{
    background: {};
}}

.admonition-{} .admonition-icon {{
    background-image: url('{}');
}}
"#,
                name, color, name, bg_color, name, icon
            ));
        }
    }

    css
}
