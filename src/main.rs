use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;

mod cli;
mod config;
mod language;
mod processor;
mod tui;

use cli::{Cli, Command, YoinkOptions};

fn main() -> Result<()> {
    let cli_args = Cli::parse();

    // Handle TUI Subcommand
    if let Some(Command::Manage { path }) = cli_args.command {
        tui::run(path).context("TUI encountered an error")?;
        return Ok(());
    }

    // Fail fast if config exists but is invalid
    let mut config_data = config::load_config()?;

    let abs_path = dunce::canonicalize(Path::new(&cli_args.path))
        .unwrap_or_else(|_| Path::new(&cli_args.path).to_path_buf())
        .to_string_lossy()
        .to_string();

    // Handle --save-profile command
    if let Some(profile_name) = &cli_args.save_profile {
        let new_profile = config::Profile {
            exclude: cli_args.exclude.clone(),
            include_only: cli_args.include_only.clone(),
            include_hidden: cli_args.include_hidden.clone(),
            max_file_size: cli_args.max_file_size,
            depth: cli_args.depth,
        };
        let proj = config_data.projects.entry(abs_path.clone()).or_default();
        proj.profiles.insert(profile_name.clone(), new_profile);
        config::save_config(&config_data).context("Failed to save config")?;
        eprintln!(
            "Saved profile '{}' for project '{}'",
            profile_name, abs_path
        );

        if cli_args.bind_profile.is_none() {
            return Ok(());
        }
    }

    // Handle --bind-profile command
    if let Some(profile_name) = &cli_args.bind_profile {
        let proj = config_data.projects.entry(abs_path.clone()).or_default();
        if !proj.profiles.contains_key(profile_name) {
            eprintln!(
                "Error: Profile '{}' does not exist in this project.",
                profile_name
            );
            return Ok(());
        }
        proj.bound_profile = Some(profile_name.clone());
        config::save_config(&config_data).context("Failed to save config")?;
        eprintln!(
            "Bound directory '{}' to profile '{}'",
            abs_path, profile_name
        );
        return Ok(()); // Exit after binding
    }

    // Figure out which profile to use
    let project_config = config_data.projects.get(&abs_path);
    let active_profile_name = if let Some(p) = &cli_args.profile {
        Some(p.clone()) // Explicit flag wins
    } else {
        project_config.and_then(|pc| pc.bound_profile.clone()) // Fallback to directory binding
    };

    let profile = if let Some(name) = active_profile_name {
        if let Some(p) = project_config.and_then(|pc| pc.profiles.get(&name)) {
            eprintln!("(Using profile: {})", name);
            p.clone()
        } else {
            eprintln!("Warning: Profile '{}' not found for this project.", name);
            config::Profile::default()
        }
    } else {
        config::Profile::default()
    };

    let default_max_file_size = 10 * 1024 * 1024; // 10MB
    let resolved_max_file_size = cli_args
        .max_file_size
        .or(profile.max_file_size)
        .unwrap_or(default_max_file_size);

    let resolved_depth = cli_args.depth.or(profile.depth);

    // Merge Profile settings with CLI ad-hoc flags
    let final_options = YoinkOptions {
        path: cli_args.path,
        exclude: merge_vecs(profile.exclude, cli_args.exclude),
        include_only: merge_vecs(profile.include_only, cli_args.include_only),
        include_hidden: merge_vecs(profile.include_hidden, cli_args.include_hidden),
        max_file_size: resolved_max_file_size,
        depth: resolved_depth,
        out: cli_args.out,
    };

    processor::process_codebase(final_options);

    Ok(())
}

fn merge_vecs(mut a: Vec<String>, b: Vec<String>) -> Vec<String> {
    a.extend(b);
    a.sort();
    a.dedup();
    a
}
