use crate::cli::YoinkOptions;
use content_inspector::{inspect, ContentType};
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use std::fmt::Write;
use std::fs;
use std::path::Path;

pub fn process_codebase(options: YoinkOptions) {
    let base_path = Path::new(&options.path);
    let mut builder = WalkBuilder::new(base_path);

    let mut overrides = OverrideBuilder::new(base_path);
    let mut has_overrides = false;

    // Process Exclusions
    for ex in &options.exclude {
        let pattern = if ex.starts_with('!') {
            ex.clone()
        } else {
            format!("!{}", ex)
        };
        if let Err(e) = overrides.add(&pattern) {
            eprintln!("Warning: Invalid exclude pattern '{}': {}", ex, e);
        } else {
            has_overrides = true;
        }
    }

    if has_overrides {
        if let Ok(override_matcher) = overrides.build() {
            builder.overrides(override_matcher);
        }
    }

    if !options.include_hidden.is_empty() {
        builder.hidden(false);
        let include_hidden: Vec<String> = options
            .include_hidden
            .iter()
            .map(|s| s.replace('\\', "/").trim_end_matches('/').to_string())
            .collect();
        let closure_base_path = base_path.to_path_buf();

        builder.filter_entry(move |entry| {
            let rel_path = entry
                .path()
                .strip_prefix(&closure_base_path)
                .unwrap_or(entry.path());

            let rel_str = rel_path.to_string_lossy().replace('\\', "/");
            let rel_str = rel_str.trim_matches('/');

            if rel_str.is_empty() {
                return true;
            } // Allow root

            let is_hidden = rel_path.components().any(|comp| {
                let name = comp.as_os_str().to_string_lossy();
                name.starts_with('.') && name != "." && name != ".."
            });

            if !is_hidden {
                return true;
            } // Allow standard visible files

            include_hidden.iter().any(|h| {
                rel_str == *h
                    || rel_str.starts_with(&format!("{}/", h))
                    || h.starts_with(&format!("{}/", rel_str))
            })
        });
    } else {
        builder.hidden(true);
    }

    let mut has_includes = false;
    let mut inc_builder = OverrideBuilder::new(base_path);
    for inc in &options.include_only {
        if let Err(e) = inc_builder.add(inc) {
            eprintln!("Warning: Invalid include pattern '{}': {}", inc, e);
        } else {
            has_includes = true;
            let _ = inc_builder.add(&format!("{}/**", inc.trim_end_matches('/')));
        }
    }
    let inc_matcher = if has_includes {
        inc_builder.build().ok()
    } else {
        None
    };

    let mut output_buffer = String::new();
    let mut file_count = 0;

    // Processing Output
    for result in builder.build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("Warning: Failed to read entry - {}", err);
                continue;
            }
        };

        let path = entry.path();
        if path.is_file() {
            if let Some(ref matcher) = inc_matcher {
                if !matches!(matcher.matched(path, false), ignore::Match::Whitelist(_)) {
                    continue;
                }
            }

            let buffer = match fs::read(path) {
                Ok(b) => b,
                Err(_) => continue,
            };

            if inspect(&buffer) == ContentType::BINARY {
                continue;
            }

            let content = String::from_utf8_lossy(&buffer);
            let display_path = path.strip_prefix(base_path).unwrap_or(path);

            let path_str = if display_path.as_os_str().is_empty() {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                display_path.to_string_lossy().replace('\\', "/")
            };

            let lang_tag = crate::language::get_language_tag(path);

            // Accumulate into buffer
            let _ = writeln!(output_buffer, "### {}", path_str);
            let _ = writeln!(output_buffer, "```{}", lang_tag);
            let _ = writeln!(output_buffer, "{}", content);
            let _ = writeln!(output_buffer, "```\n");

            file_count += 1;
        }
    }

    let char_count = output_buffer.chars().count();
    let approx_tokens = char_count as f32 / 3.35;

    // Handle Output Destination
    if let Some(out_path) = options.out {
        if let Err(e) = fs::write(&out_path, &output_buffer) {
            eprintln!("Error writing to file {}: {}", out_path, e);
        } else {
            eprintln!("Successfully saved to {}", out_path);
        }
    } else {
        // Default to Clipboard
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(output_buffer) {
                    eprintln!("Error setting clipboard: {}", e);
                    eprintln!("Fallback: use --out <FILE> to save to a file instead.");
                } else {
                    eprintln!("Successfully copied to clipboard!");
                }
            }
            Err(e) => {
                eprintln!("Failed to initialize clipboard: {}", e);
                eprintln!("Fallback: run again with --out <FILE>");
            }
        }
    }

    // Print Stats Summary
    eprintln!(
        "{} files yoinked. ~{:.0} tokens ({} chars)",
        file_count, approx_tokens, char_count
    );
}
