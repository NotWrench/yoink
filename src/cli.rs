use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yoink")]
#[command(version, about = "Yoink your codebase into clipboard", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// The path to the codebase
    #[arg(default_value = ".")]
    pub path: String,

    /// Specific directories or files to exclude
    #[arg(short, long, value_delimiter = ',')]
    pub exclude: Vec<String>,

    /// Only include files matching these glob patterns
    #[arg(short = 'i', long, value_delimiter = ',')]
    pub include_only: Vec<String>,

    /// Hidden files or directories to explicitly include
    #[arg(short = 'H', long, value_delimiter = ',')]
    pub include_hidden: Vec<String>,

    /// Maximum file size in bytes (default: 10485760 / 10MB)
    #[arg(long)]
    pub max_file_size: Option<usize>,

    /// Maximum directory traversal depth
    #[arg(long)]
    pub depth: Option<usize>,

    /// Output to a file instead of the clipboard
    #[arg(short, long)]
    pub out: Option<String>,

    /// Force use a specific profile, ignoring directory bindings
    #[arg(short = 'p', long)]
    pub profile: Option<String>,

    /// Save the current flags (--exclude, --include-only, etc.) as a new profile
    #[arg(long)]
    pub save_profile: Option<String>,

    /// Bind the current directory to a profile so it runs automatically in the future
    #[arg(long)]
    pub bind_profile: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Open the TUI to manage profiles and directory bindings
    Manage {
        /// The path to manage bindings for
        #[arg(default_value = ".")]
        path: String,
    },
}

#[derive(Clone, Default)]
pub struct YoinkOptions {
    pub path: String,
    pub exclude: Vec<String>,
    pub include_only: Vec<String>,
    pub include_hidden: Vec<String>,
    pub max_file_size: usize,
    pub depth: Option<usize>,
    pub out: Option<String>,
}
