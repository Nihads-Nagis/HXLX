// lib.rs
//! Lightweight mdBook admonition preprocessor with custom styling

mod config;
mod markdown;
mod parse;
mod render;
mod types;

pub use markdown::AdmonishPreprocessor;
