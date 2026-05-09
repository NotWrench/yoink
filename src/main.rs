use clap::Parser;
use std::path::Path;

mod cli;
mod config;
mod processor;

use cli::{Cli, YoinkOptions};

fn main() {
    let cli_args = Cli::parse();
    let mut config_data = config::load_config();

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
        };
        config_data
            .profiles
            .insert(profile_name.clone(), new_profile);
        config::save_config(&config_data);
        eprintln!("Saved profile '{}'", profile_name);

        if cli_args.bind_profile.is_none() {
            return;
        }
    }

    // Handle --bind-profile command
    if let Some(profile_name) = &cli_args.bind_profile {
        if !config_data.profiles.contains_key(profile_name) {
            eprintln!("Error: Profile '{}' does not exist.", profile_name);
            return;
        }
        config_data
            .bindings
            .insert(abs_path.clone(), profile_name.clone());
        config::save_config(&config_data);
        eprintln!(
            "Bound directory '{}' to profile '{}'",
            abs_path, profile_name
        );
        return; // Exit after binding
    }

    // Figure out which profile to use
    let active_profile_name = if let Some(p) = &cli_args.profile {
        Some(p.clone()) // Explicit flag wins
    } else {
        config_data.bindings.get(&abs_path).cloned() // Fallback to directory binding
    };

    let profile = if let Some(name) = active_profile_name {
        if let Some(p) = config_data.profiles.get(&name) {
            eprintln!("(Using profile: {})", name);
            p.clone()
        } else {
            eprintln!("Warning: Bound profile '{}' not found in config.", name);
            config::Profile::default()
        }
    } else {
        config::Profile::default()
    };

    // Merge Profile settings with CLI ad-hoc flags
    let final_options = YoinkOptions {
        path: cli_args.path,
        exclude: merge_vecs(profile.exclude, cli_args.exclude),
        include_only: merge_vecs(profile.include_only, cli_args.include_only),
        include_hidden: merge_vecs(profile.include_hidden, cli_args.include_hidden),
        out: cli_args.out,
    };

    processor::process_codebase(final_options);
}

fn merge_vecs(mut a: Vec<String>, b: Vec<String>) -> Vec<String> {
    a.extend(b);
    a.sort();
    a.dedup();
    a
}
