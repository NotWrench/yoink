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

    // Process Explicit Hidden Inclusions
    for inc in &options.include_hidden {
        if let Err(e) = overrides.add(inc) {
            eprintln!("Warning: Invalid include pattern '{}': {}", inc, e);
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
            .map(|s| s.replace('\\', "/"))
            .collect();
        let closure_base_path = base_path.to_path_buf();

        builder.filter_entry(move |entry| {
            let rel_path = entry
                .path()
                .strip_prefix(&closure_base_path)
                .unwrap_or(entry.path());
            let rel_str = rel_path.to_string_lossy().replace('\\', "/");

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

            // If it's hidden, allow it if it is a parent OR a child of our target included path
            include_hidden
                .iter()
                .any(|h| rel_str.starts_with(h) || h.starts_with(&rel_str))
        });
    } else {
        builder.hidden(true);
    }

    let include_only = options.include_only.clone();

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
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            if !include_only.is_empty() && !include_only.contains(&ext.to_string()) {
                continue;
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

            let lang_tag = get_language_tag(path);

            // Accumulate into buffer instead of printing to stdout
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

fn get_language_tag(path: &Path) -> &str {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Explicit file name matches
    if file_name == "dockerfile" {
        return "dockerfile";
    }
    if file_name == "makefile" {
        return "makefile";
    }
    if file_name == "cmakelists.txt" {
        return "cmake";
    }
    if file_name.ends_with("ignore") {
        return "ignore";
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Extension matches
    match ext.to_lowercase().as_str() {
        "rs" => "rust",
        "js" | "cjs" | "mjs" => "javascript",
        "ts" | "cts" | "mts" => "typescript",
        "jsx" => "jsx",
        "tsx" => "tsx",
        "py" => "python",
        "go" => "go",
        "c" => "c",
        "cpp" | "cc" | "cxx" | "h" | "hpp" => "cpp",
        "cs" => "csharp",
        "java" => "java",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "sh" | "bash" | "zsh" => "bash",
        "bat" | "cmd" => "batch",
        "ps1" => "powershell",
        "sql" => "sql",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "md" | "markdown" => "markdown",
        "vue" => "vue",
        "svelte" => "svelte",
        "graphql" | "gql" => "graphql",
        "dart" => "dart",
        "lua" => "lua",
        "zig" => "zig",
        "el" => "lisp",
        "clj" => "clojure",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        _ => ext, // Fallback to raw extension
    }
}
