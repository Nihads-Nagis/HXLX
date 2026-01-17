// markdown.rs - Fixed with proper range handling
use mdbook::{
    book::Book,
    preprocess::{Preprocessor, PreprocessorContext},
    utils::fs::write_file,
};
use pulldown_cmark::{CodeBlockKind::*, Event, OffsetIter, Options, Parser, Tag};
use std::ops::Range;
use std::path::Path;

use crate::{
    parse::parse_admonition,
    render::{generate_custom_css, render_admonition},
    types::{AdmonishConfig, DirectiveMap},
};

pub struct AdmonishPreprocessor;

impl Preprocessor for AdmonishPreprocessor {
    fn name(&self) -> &str {
        "admonish"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> mdbook::errors::Result<Book> {
        if ctx.renderer != "html" {
            return Ok(book);
        }

        let config = crate::config::parse_config(ctx)?;
        let directive_map = DirectiveMap::new(config.custom.clone());

        // Generate CSS
        if let Some(dest) = &ctx.config.build.build_dir {
            let css_path = Path::new(dest).join("admonish.css");
            let css = generate_custom_css(&config);
            write_file(&css_path, &css)?;
        }

        book.for_each_mut(|item| {
            if let mdbook::book::BookItem::Chapter(chapter) = item {
                if let Ok(processed) = process_chapter(&chapter.content, &config, &directive_map) {
                    chapter.content = processed;
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

/// Process a chapter using range-based replacement
fn process_chapter(
    content: &str,
    config: &AdmonishConfig,
    directive_map: &DirectiveMap,
) -> anyhow::Result<String> {
    let mut replacements = Vec::new();
    let parser = Parser::new_ext(content, Options::all());
    let mut offset_iter = parser.into_offset_iter();

    let mut admonition_start = 0;
    let mut info_string = String::new();
    let mut in_admonition = false;
    let mut depth = 0;

    while let Some((event, range)) = offset_iter.next() {
        match event {
            Event::Start(Tag::CodeBlock(Fenced(info))) if info.starts_with("admonish") => {
                in_admonition = true;
                depth = 1;
                admonition_start = range.start;
                info_string = info.to_string();
            }
            Event::End(Tag::CodeBlock(_)) if in_admonition => {
                depth -= 1;
                if depth == 0 {
                    in_admonition = false;
                    let admonition_end = range.end;
                    let raw_block = &content[admonition_start..admonition_end];

                    // Parse the entire raw block
                    if let Some(Ok(meta)) =
                        parse_admonition(&info_string, raw_block, config, directive_map)
                    {
                        let replacement = render_admonition(&meta, raw_block);

                        replacements.push(Replacement {
                            range: admonition_start..admonition_end,
                            replacement,
                        });
                    }
                }
            }
            Event::Start(Tag::CodeBlock(_)) if in_admonition => {
                depth += 1;
            }
            _ => {}
        }
    }

    apply_replacements(content, replacements)
}

#[derive(Debug)]
struct Replacement {
    range: Range<usize>,
    replacement: String,
}

/// Apply replacements to original content
fn apply_replacements(content: &str, mut replacements: Vec<Replacement>) -> anyhow::Result<String> {
    // Sort by start position (end-to-start would be better for in-place,
    // but we're building a new string anyway)
    replacements.sort_by_key(|r| r.range.start);

    let mut result = String::with_capacity(content.len() * 2);
    let mut last_pos = 0;

    for replacement in replacements {
        // Add content before this replacement
        result.push_str(&content[last_pos..replacement.range.start]);
        // Add replacement
        result.push_str(&replacement.replacement);
        last_pos = replacement.range.end;
    }

    // Add remaining content
    result.push_str(&content[last_pos..]);

    Ok(result)
}
