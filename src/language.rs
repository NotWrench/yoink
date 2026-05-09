use std::path::Path;

pub fn get_language_tag(path: &Path) -> &str {
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
        "py" => "python",
        "cpp" | "cc" | "cxx" | "h" | "hpp" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "kt" | "kts" => "kotlin",
        "sh" | "bash" | "zsh" => "bash",
        "bat" | "cmd" => "batch",
        "ps1" => "powershell",
        "html" | "htm" => "html",
        "scss" | "sass" => "scss",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        "graphql" | "gql" => "graphql",
        "el" => "lisp",
        "clj" => "clojure",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        _ => ext, // Fallback to raw extension
    }
}
