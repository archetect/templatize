mod cli;
mod diff;

use anyhow::Result;
use cli::{Cli, Commands};
use std::path::PathBuf;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

fn main() -> Result<()> {
    let cli = Cli::parse_args();
    
    setup_logging(&cli)?;
    
    info!("Starting templatize");
    
    match cli.command {
        Commands::Exact { 
            token, 
            replacement, 
            path, 
            contents, 
            target, 
            dry_run,
            interactive 
        } => {
            // Validate that at least one of -p or -c is specified
            if !path && !contents {
                use inquire::Confirm;
                
                let enable_path = Confirm::new("Enable path templating (-p)?")
                    .with_default(true)
                    .prompt()?;
                    
                let enable_contents = Confirm::new("Enable contents templating (-c)?")
                    .with_default(true)
                    .prompt()?;
                
                if !enable_path && !enable_contents {
                    anyhow::bail!("At least one of --path (-p) or --contents (-c) must be enabled");
                }
                
                return handle_exact_command(
                    token, 
                    replacement, 
                    enable_path, 
                    enable_contents, 
                    target, 
                    dry_run,
                    interactive
                );
            }
            
            handle_exact_command(token, replacement, path, contents, target, dry_run, interactive)?;
        }
        Commands::Shapes { 
            token, 
            replacement, 
            path, 
            contents, 
            target, 
            dry_run,
            interactive 
        } => {
            // Validate that at least one of -p or -c is specified
            if !path && !contents {
                use inquire::Confirm;
                
                let enable_path = Confirm::new("Enable path templating (-p)?")
                    .with_default(true)
                    .prompt()?;
                    
                let enable_contents = Confirm::new("Enable contents templating (-c)?")
                    .with_default(true)
                    .prompt()?;
                
                if !enable_path && !enable_contents {
                    anyhow::bail!("At least one of --path (-p) or --contents (-c) must be enabled");
                }
                
                return handle_shapes_command(
                    token, 
                    replacement, 
                    enable_path, 
                    enable_contents, 
                    target, 
                    dry_run,
                    interactive
                );
            }
            
            handle_shapes_command(token, replacement, path, contents, target, dry_run, interactive)?;
        }
        Commands::Escape { target, dry_run, interactive } => {
            handle_escape_command(target, dry_run, interactive)?;
        }
    }
    
    info!("Templatize completed successfully");
    Ok(())
}

fn handle_exact_command(
    token: String,
    replacement: String,
    path: bool,
    contents: bool,
    target: Option<PathBuf>,
    dry_run: bool,
    interactive: bool,
) -> Result<()> {
    let target_dir = target.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    info!("Exact replacement: '{}' -> '{}'", token, replacement);
    info!("Target directory: {:?}", target_dir);
    info!("Path templating: {}", path);
    info!("Contents templating: {}", contents);
    info!("Interactive mode: {}", interactive);
    
    if dry_run {
        warn!("Dry run mode - no changes will be made");
    }
    
    if !target_dir.exists() {
        anyhow::bail!("Target directory does not exist: {:?}", target_dir);
    }
    
    if !target_dir.is_dir() {
        anyhow::bail!("Target must be a directory: {:?}", target_dir);
    }
    
    // Use the core templating functionality
    let result = if interactive {
        let content_callback = |file_path: &std::path::Path, old_content: &str, new_content: &str, description: &str| {
            diff::show_diff_and_confirm(file_path, old_content, new_content, description)
        };
        
        let path_callback = |old_path: &std::path::Path, new_path: &std::path::Path, change_type: &str| {
            diff::show_path_change_and_confirm(old_path, new_path, change_type)
        };
        
        templatize_core::process_directory_interactive(
            &target_dir,
            &token,
            &replacement,
            path,
            contents,
            dry_run,
            content_callback,
            path_callback,
        )?
    } else {
        templatize_core::process_directory(
            &target_dir,
            &token,
            &replacement,
            path,
            contents,
            dry_run,
        )?
    };
    
    println!("Templating complete!");
    println!("  Files processed: {}", result.files_processed);
    println!("  Paths renamed: {}", result.paths_renamed);
    println!("  Content changes: {}", result.content_changes);
    
    Ok(())
}

fn handle_shapes_command(
    token: String,
    replacement: String,
    path: bool,
    contents: bool,
    target: Option<PathBuf>,
    dry_run: bool,
    interactive: bool,
) -> Result<()> {
    let target_dir = target.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    info!("Shapes replacement: '{}' -> '{}'", token, replacement);
    info!("Target directory: {:?}", target_dir);
    info!("Path templating: {}", path);
    info!("Contents templating: {}", contents);
    info!("Interactive mode: {}", interactive);
    
    if dry_run {
        warn!("Dry run mode - no changes will be made");
    }
    
    if !target_dir.exists() {
        anyhow::bail!("Target directory does not exist: {:?}", target_dir);
    }
    
    if !target_dir.is_dir() {
        anyhow::bail!("Target must be a directory: {:?}", target_dir);
    }
    
    // Use the core shapes functionality
    let result = if interactive {
        let content_callback = |file_path: &std::path::Path, old_content: &str, new_content: &str, description: &str| {
            diff::show_diff_and_confirm(file_path, old_content, new_content, description)
        };
        
        let path_callback = |old_path: &std::path::Path, new_path: &std::path::Path, change_type: &str| {
            diff::show_path_change_and_confirm(old_path, new_path, change_type)
        };
        
        templatize_core::process_directory_shapes_interactive(
            &target_dir,
            &token,
            &replacement,
            path,
            contents,
            dry_run,
            content_callback,
            path_callback,
        )?
    } else {
        templatize_core::process_directory_shapes(
            &target_dir,
            &token,
            &replacement,
            path,
            contents,
            dry_run,
        )?
    };
    
    println!("Case shapes templating complete!");
    println!("  Files processed: {}", result.files_processed);
    println!("  Paths renamed: {}", result.paths_renamed);
    println!("  Content changes: {}", result.content_changes);
    
    Ok(())
}

fn handle_escape_command(target: Option<PathBuf>, dry_run: bool, interactive: bool) -> Result<()> {
    let target_path = target.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    info!("Jinja escaping for: {:?}", target_path);
    info!("Interactive mode: {}", interactive);
    
    if dry_run {
        warn!("Dry run mode - no changes will be made");
    }
    
    if !target_path.exists() {
        anyhow::bail!("Target does not exist: {:?}", target_path);
    }
    
    // Use the core escaping functionality
    let result = if interactive {
        let callback = |file_path: &std::path::Path, old_content: &str, new_content: &str, description: &str| {
            diff::show_diff_and_confirm(file_path, old_content, new_content, description)
        };
        
        templatize_core::escape_jinja_syntax_interactive(&target_path, dry_run, callback)?
    } else {
        templatize_core::escape_jinja_syntax(&target_path, dry_run)?
    };
    
    println!("Jinja escaping complete!");
    println!("  Files processed: {}", result.files_processed);
    println!("  Content changes: {}", result.content_changes);
    
    Ok(())
}

fn setup_logging(cli: &Cli) -> Result<()> {
    let filter = if cli.quiet {
        EnvFilter::new("error")
    } else if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .compact()
        )
        .with(filter)
        .init();

    Ok(())
}