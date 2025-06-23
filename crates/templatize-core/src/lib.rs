use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

pub mod templater;

pub use templater::{ExactTemplater, JinjaEscaper, CaseShapeTemplater, TemplateOptions, CaseShapeMapping};

#[derive(thiserror::Error, Debug)]
pub enum TemplateError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path error: {message}")]
    Path { message: String },
    #[error("Template error: {message}")]
    Template { message: String },
}

pub struct TemplatizeResult {
    pub files_processed: usize,
    pub paths_renamed: usize,
    pub content_changes: usize,
}

pub fn process_directory(
    target: &Path,
    token: &str,
    replacement: &str,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
) -> Result<TemplatizeResult> {
    let templater = ExactTemplater::new(token, replacement);
    let options = TemplateOptions {
        process_paths,
        process_contents,
        dry_run,
    };
    
    info!("Starting directory processing: {:?}", target);
    
    let mut result = TemplatizeResult {
        files_processed: 0,
        paths_renamed: 0,
        content_changes: 0,
    };
    
    if target.is_file() {
        process_file(target, &templater, &options, &mut result)?;
    } else if target.is_dir() {
        // First, process all contents inside the target directory
        process_directory_contents_recursive(target, &templater, &options, &mut result)?;
        
        // Finally, rename the target directory itself if needed
        if options.process_paths {
            rename_target_directory(target, &templater, &options, &mut result)?;
        }
    } else {
        anyhow::bail!("Target does not exist or is not a file or directory: {:?}", target);
    }
    
    info!(
        "Processing complete: {} files processed, {} paths renamed, {} content changes",
        result.files_processed, result.paths_renamed, result.content_changes
    );
    
    Ok(result)
}

pub fn escape_jinja_syntax(
    target: &Path,
    dry_run: bool,
) -> Result<TemplatizeResult> {
    let escaper = JinjaEscaper::new()
        .map_err(|e| anyhow::anyhow!("Failed to create Jinja escaper: {}", e))?;
    
    info!("Starting Jinja escaping for: {:?}", target);
    
    let mut result = TemplatizeResult {
        files_processed: 0,
        paths_renamed: 0,
        content_changes: 0,
    };
    
    if target.is_file() {
        escape_file(target, &escaper, dry_run, &mut result)?;
    } else if target.is_dir() {
        escape_directory_recursive(target, &escaper, dry_run, &mut result)?;
    } else {
        anyhow::bail!("Target does not exist or is not a file or directory: {:?}", target);
    }
    
    info!(
        "Jinja escaping complete: {} files processed, {} content changes",
        result.files_processed, result.content_changes
    );
    
    Ok(result)
}

pub fn process_directory_interactive<F, G>(
    target: &Path,
    token: &str,
    replacement: &str,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    content_callback: F,
    path_callback: G,
) -> Result<TemplatizeResult>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
    G: Fn(&Path, &Path, &str) -> Result<bool>,
{
    let templater = ExactTemplater::new(token, replacement);
    
    info!("Starting interactive directory processing: {:?}", target);
    
    let mut result = TemplatizeResult {
        files_processed: 0,
        paths_renamed: 0,
        content_changes: 0,
    };
    
    if target.is_file() {
        process_file_interactive(target, &templater, process_paths, process_contents, dry_run, &content_callback, &path_callback, &mut result)?;
    } else if target.is_dir() {
        // First, process all contents inside the target directory
        process_directory_contents_recursive_interactive(target, &templater, process_paths, process_contents, dry_run, &content_callback, &path_callback, &mut result)?;
        
        // Finally, rename the target directory itself if needed
        if process_paths {
            rename_target_directory_interactive(target, &templater, process_paths, process_contents, dry_run, &content_callback, &path_callback, &mut result)?;
        }
    } else {
        anyhow::bail!("Target does not exist or is not a file or directory: {:?}", target);
    }
    
    info!(
        "Interactive processing complete: {} files processed, {} paths renamed, {} content changes",
        result.files_processed, result.paths_renamed, result.content_changes
    );
    
    Ok(result)
}

pub fn escape_jinja_syntax_interactive<F>(
    target: &Path,
    dry_run: bool,
    callback: F,
) -> Result<TemplatizeResult>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
{
    let escaper = JinjaEscaper::new()
        .map_err(|e| anyhow::anyhow!("Failed to create Jinja escaper: {}", e))?;
    
    info!("Starting interactive Jinja escaping for: {:?}", target);
    
    let mut result = TemplatizeResult {
        files_processed: 0,
        paths_renamed: 0,
        content_changes: 0,
    };
    
    if target.is_file() {
        escape_file_interactive(target, &escaper, dry_run, &callback, &mut result)?;
    } else if target.is_dir() {
        escape_directory_recursive_interactive(target, &escaper, dry_run, &callback, &mut result)?;
    } else {
        anyhow::bail!("Target does not exist or is not a file or directory: {:?}", target);
    }
    
    info!(
        "Interactive Jinja escaping complete: {} files processed, {} content changes",
        result.files_processed, result.content_changes
    );
    
    Ok(result)
}

fn process_directory_contents_recursive(
    dir: &Path,
    templater: &ExactTemplater,
    options: &TemplateOptions,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Processing directory contents: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Collect directories and files separately for depth-first processing
    let mut directories = Vec::new();
    let mut files = Vec::new();
    
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }
    
    // First, recursively process subdirectories' CONTENTS (depth-first)
    for dir_path in &directories {
        process_directory_contents_recursive(dir_path, templater, options, result)?;
    }
    
    // Then process files in current directory (while paths are still valid)
    for file_path in &files {
        process_file(file_path, templater, options, result)?;
    }
    
    // Finally, rename subdirectories in reverse order (deepest paths first)
    // This happens after all files in this directory are processed
    if options.process_paths {
        directories.reverse(); // Process in reverse order for safety
        for dir_path in &directories {
            // Only use component replacement to rename directory within its current parent
            if let Some(new_name) = templater.process_path_component(dir_path) {
                let new_path = dir_path.parent().unwrap().join(&new_name);
                
                if options.dry_run {
                    info!("Would rename directory: {:?} -> {:?}", dir_path, new_path);
                } else {
                    info!("Renaming directory: {:?} -> {:?}", dir_path, new_path);
                    fs::rename(dir_path, &new_path)?;
                }
                result.paths_renamed += 1;
            }
        }
    }
    
    Ok(())
}

fn process_directory_recursive(
    dir: &Path,
    templater: &ExactTemplater,
    options: &TemplateOptions,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Processing directory: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Collect directories and files separately for depth-first processing
    let mut directories = Vec::new();
    let mut files = Vec::new();
    
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }
    
    // First, recursively process all subdirectories' CONTENTS (but don't rename the subdirectories yet)
    for dir_path in &directories {
        process_directory_contents_recursive(dir_path, templater, options, result)?;
    }
    
    // Then process files in current directory
    for file_path in &files {
        process_file(file_path, templater, options, result)?;
    }
    
    // Finally, rename subdirectories in reverse order (deepest paths first)
    // This ensures we rename child directories only after all their contents are processed
    if options.process_paths {
        directories.reverse(); // Process in reverse order for safety
        for dir_path in &directories {
            // Only use component replacement to rename directory within its current parent
            if let Some(new_name) = templater.process_path_component(dir_path) {
                let new_path = dir_path.parent().unwrap().join(&new_name);
                
                if options.dry_run {
                    info!("Would rename directory: {:?} -> {:?}", dir_path, new_path);
                } else {
                    info!("Renaming directory: {:?} -> {:?}", dir_path, new_path);
                    fs::rename(dir_path, &new_path)?;
                }
                result.paths_renamed += 1;
            }
        }
    }
    
    Ok(())
}

fn rename_target_directory(
    target: &Path,
    templater: &ExactTemplater,
    options: &TemplateOptions,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Checking if target directory needs renaming: {:?}", target);
    
    // Only use component replacement to rename target directory within its current parent
    if let Some(new_name) = templater.process_path_component(target) {
        let new_path = target.parent().unwrap().join(&new_name);
        
        if options.dry_run {
            info!("Would rename target directory: {:?} -> {:?}", target, new_path);
        } else {
            info!("Renaming target directory: {:?} -> {:?}", target, new_path);
            fs::rename(target, &new_path)?;
        }
        result.paths_renamed += 1;
    }
    
    Ok(())
}

fn process_file(
    file_path: &Path,
    templater: &ExactTemplater,
    options: &TemplateOptions,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Processing file: {:?}", file_path);
    result.files_processed += 1;
    
    let mut content_changed = false;
    let mut path_changed = false;
    
    // Process file contents
    if options.process_contents {
        if let Ok(content) = fs::read_to_string(file_path) {
            if let Some(new_content) = templater.process_content(&content) {
                if options.dry_run {
                    info!("Would update contents of: {:?}", file_path);
                } else {
                    info!("Updating contents of: {:?}", file_path);
                    fs::write(file_path, new_content)?;
                }
                content_changed = true;
            }
        } else {
            debug!("Skipping binary file: {:?}", file_path);
        }
    }
    
    // Process file path (only rename within current directory)
    if options.process_paths {
        if let Some(new_name) = templater.process_path_component(file_path) {
            let new_path = file_path.parent().unwrap().join(&new_name);
            
            if options.dry_run {
                info!("Would rename file: {:?} -> {:?}", file_path, new_path);
            } else {
                info!("Renaming file: {:?} -> {:?}", file_path, new_path);
                fs::rename(file_path, &new_path)?;
            }
            path_changed = true;
        }
    }
    
    if content_changed {
        result.content_changes += 1;
    }
    if path_changed {
        result.paths_renamed += 1;
    }
    
    Ok(())
}

fn escape_directory_recursive(
    dir: &Path,
    escaper: &JinjaEscaper,
    dry_run: bool,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Escaping directory: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Process files
    for entry in &entries {
        let path = entry.path();
        if path.is_file() {
            escape_file(&path, escaper, dry_run, result)?;
        }
    }
    
    // Process subdirectories recursively
    for entry in &entries {
        let path = entry.path();
        if path.is_dir() {
            escape_directory_recursive(&path, escaper, dry_run, result)?;
        }
    }
    
    Ok(())
}

fn escape_file(
    file_path: &Path,
    escaper: &JinjaEscaper,
    dry_run: bool,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Escaping file: {:?}", file_path);
    result.files_processed += 1;
    
    if let Ok(content) = fs::read_to_string(file_path) {
        if let Some(escaped_content) = escaper.escape_content(&content) {
            if dry_run {
                info!("Would escape Jinja syntax in: {:?}", file_path);
            } else {
                info!("Escaping Jinja syntax in: {:?}", file_path);
                fs::write(file_path, escaped_content)?;
            }
            result.content_changes += 1;
        }
    } else {
        debug!("Skipping binary file: {:?}", file_path);
    }
    
    Ok(())
}

fn process_directory_contents_recursive_interactive<F, G>(
    dir: &Path,
    templater: &ExactTemplater,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    content_callback: &F,
    path_callback: &G,
    result: &mut TemplatizeResult,
) -> Result<()>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
    G: Fn(&Path, &Path, &str) -> Result<bool>,
{
    debug!("Processing interactive directory contents: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Collect directories and files separately for depth-first processing
    let mut directories = Vec::new();
    let mut files = Vec::new();
    
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }
    
    // First, recursively process subdirectories' CONTENTS (depth-first)
    for dir_path in &directories {
        process_directory_contents_recursive_interactive(dir_path, templater, process_paths, process_contents, dry_run, content_callback, path_callback, result)?;
    }
    
    // Then rename subdirectories in reverse order (deepest paths first)
    // This establishes the new directory structure before processing files
    if process_paths {
        directories.reverse(); // Process in reverse order for safety
        for dir_path in &directories {
            // Only use component replacement to rename directory within its current parent
            if let Some(new_name) = templater.process_path_component(dir_path) {
                let new_path = dir_path.parent().unwrap().join(&new_name);
                
                if path_callback(dir_path, &new_path, "Directory")? {
                    if dry_run {
                        info!("Would rename directory: {:?} -> {:?}", dir_path, new_path);
                    } else {
                        info!("Renaming directory: {:?} -> {:?}", dir_path, new_path);
                        fs::rename(dir_path, &new_path)?;
                    }
                    result.paths_renamed += 1;
                }
            }
        }
    }
    
    // Finally, process files in current directory after directories are renamed
    for file_path in &files {
        process_file_interactive(file_path, templater, process_paths, process_contents, dry_run, content_callback, path_callback, result)?;
    }
    
    Ok(())
}

fn rename_target_directory_interactive<F, G>(
    target: &Path,
    templater: &ExactTemplater,
    _process_paths: bool,
    _process_contents: bool,
    dry_run: bool,
    _content_callback: &F,
    path_callback: &G,
    result: &mut TemplatizeResult,
) -> Result<()>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
    G: Fn(&Path, &Path, &str) -> Result<bool>,
{
    debug!("Checking if target directory needs interactive renaming: {:?}", target);
    
    // Only use component replacement to rename target directory within its current parent
    if let Some(new_name) = templater.process_path_component(target) {
        let new_path = target.parent().unwrap().join(&new_name);
        
        if path_callback(target, &new_path, "Target Directory")? {
            if dry_run {
                info!("Would rename target directory: {:?} -> {:?}", target, new_path);
            } else {
                info!("Renaming target directory: {:?} -> {:?}", target, new_path);
                fs::rename(target, &new_path)?;
            }
            result.paths_renamed += 1;
        }
    }
    
    Ok(())
}

fn process_directory_recursive_interactive<F, G>(
    dir: &Path,
    templater: &ExactTemplater,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    content_callback: &F,
    path_callback: &G,
    result: &mut TemplatizeResult,
) -> Result<()>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
    G: Fn(&Path, &Path, &str) -> Result<bool>,
{
    debug!("Processing interactive directory: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Collect directories and files separately for depth-first processing
    let mut directories = Vec::new();
    let mut files = Vec::new();
    
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }
    
    // First, recursively process all subdirectories' CONTENTS (but don't rename the subdirectories yet)
    for dir_path in &directories {
        process_directory_contents_recursive_interactive(dir_path, templater, process_paths, process_contents, dry_run, content_callback, path_callback, result)?;
    }
    
    // Then process files in current directory
    for file_path in &files {
        process_file_interactive(file_path, templater, process_paths, process_contents, dry_run, content_callback, path_callback, result)?;
    }
    
    // Finally, rename subdirectories in reverse order (deepest paths first)
    // This ensures we rename child directories only after all their contents are processed
    if process_paths {
        directories.reverse(); // Process in reverse order for safety
        for dir_path in &directories {
            // Only use component replacement to rename directory within its current parent
            if let Some(new_name) = templater.process_path_component(dir_path) {
                let new_path = dir_path.parent().unwrap().join(&new_name);
                
                if path_callback(dir_path, &new_path, "Directory")? {
                    if dry_run {
                        info!("Would rename directory: {:?} -> {:?}", dir_path, new_path);
                    } else {
                        info!("Renaming directory: {:?} -> {:?}", dir_path, new_path);
                        fs::rename(dir_path, &new_path)?;
                    }
                    result.paths_renamed += 1;
                }
            }
        }
    }
    
    Ok(())
}

fn process_file_interactive<F, G>(
    file_path: &Path,
    templater: &ExactTemplater,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    content_callback: &F,
    path_callback: &G,
    result: &mut TemplatizeResult,
) -> Result<()>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
    G: Fn(&Path, &Path, &str) -> Result<bool>,
{
    debug!("Processing file: {:?}", file_path);
    result.files_processed += 1;
    
    let mut content_changed = false;
    let mut path_changed = false;
    
    // Process file contents
    if process_contents {
        if let Ok(content) = fs::read_to_string(file_path) {
            if let Some(new_content) = templater.process_content(&content) {
                if content_callback(file_path, &content, &new_content, "Content change")? {
                    if dry_run {
                        info!("Would update contents of: {:?}", file_path);
                    } else {
                        info!("Updating contents of: {:?}", file_path);
                        fs::write(file_path, new_content)?;
                    }
                    content_changed = true;
                }
            }
        } else {
            debug!("Skipping binary file: {:?}", file_path);
        }
    }
    
    // Process file path (only rename within current directory)
    if process_paths {
        if let Some(new_name) = templater.process_path_component(file_path) {
            let new_path = file_path.parent().unwrap().join(&new_name);
            
            if path_callback(file_path, &new_path, "File")? {
                if dry_run {
                    info!("Would rename file: {:?} -> {:?}", file_path, new_path);
                } else {
                    info!("Renaming file: {:?} -> {:?}", file_path, new_path);
                    fs::rename(file_path, &new_path)?;
                }
                path_changed = true;
            }
        }
    }
    
    if content_changed {
        result.content_changes += 1;
    }
    if path_changed {
        result.paths_renamed += 1;
    }
    
    Ok(())
}

fn escape_directory_recursive_interactive<F>(
    dir: &Path,
    escaper: &JinjaEscaper,
    dry_run: bool,
    callback: &F,
    result: &mut TemplatizeResult,
) -> Result<()>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
{
    debug!("Escaping directory: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Process files
    for entry in &entries {
        let path = entry.path();
        if path.is_file() {
            escape_file_interactive(&path, escaper, dry_run, callback, result)?;
        }
    }
    
    // Process subdirectories recursively
    for entry in &entries {
        let path = entry.path();
        if path.is_dir() {
            escape_directory_recursive_interactive(&path, escaper, dry_run, callback, result)?;
        }
    }
    
    Ok(())
}

fn escape_file_interactive<F>(
    file_path: &Path,
    escaper: &JinjaEscaper,
    dry_run: bool,
    callback: &F,
    result: &mut TemplatizeResult,
) -> Result<()>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
{
    debug!("Escaping file: {:?}", file_path);
    result.files_processed += 1;
    
    if let Ok(content) = fs::read_to_string(file_path) {
        if let Some(escaped_content) = escaper.escape_content(&content) {
            if callback(file_path, &content, &escaped_content, "Jinja escaping")? {
                if dry_run {
                    info!("Would escape Jinja syntax in: {:?}", file_path);
                } else {
                    info!("Escaping Jinja syntax in: {:?}", file_path);
                    fs::write(file_path, escaped_content)?;
                }
                result.content_changes += 1;
            }
        }
    } else {
        debug!("Skipping binary file: {:?}", file_path);
    }
    
    Ok(())
}

pub fn process_directory_shapes(
    target: &Path,
    token: &str,
    replacement: &str,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
) -> Result<TemplatizeResult> {
    let templater = CaseShapeTemplater::new(token, replacement)?;
    
    info!("Starting directory shapes processing: {:?}", target);
    
    let mut result = TemplatizeResult {
        files_processed: 0,
        paths_renamed: 0,
        content_changes: 0,
    };
    
    if target.is_file() {
        process_file_shapes(target, &templater, process_paths, process_contents, dry_run, &mut result)?;
    } else if target.is_dir() {
        // First, process all contents inside the target directory
        process_directory_contents_recursive_shapes(target, &templater, process_paths, process_contents, dry_run, &mut result)?;
        
        // Finally, rename the target directory itself if needed
        if process_paths {
            rename_target_directory_shapes(target, &templater, process_paths, process_contents, dry_run, &mut result)?;
        }
    } else {
        anyhow::bail!("Target does not exist or is not a file or directory: {:?}", target);
    }
    
    info!(
        "Shapes processing complete: {} files processed, {} paths renamed, {} content changes",
        result.files_processed, result.paths_renamed, result.content_changes
    );
    
    Ok(result)
}

pub fn process_directory_shapes_interactive<F, G>(
    target: &Path,
    token: &str,
    replacement: &str,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    content_callback: F,
    path_callback: G,
) -> Result<TemplatizeResult>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
    G: Fn(&Path, &Path, &str) -> Result<bool>,
{
    let templater = CaseShapeTemplater::new(token, replacement)?;
    
    info!("Starting interactive shapes processing: {:?}", target);
    
    let mut result = TemplatizeResult {
        files_processed: 0,
        paths_renamed: 0,
        content_changes: 0,
    };
    
    if target.is_file() {
        process_file_shapes_interactive(target, &templater, process_paths, process_contents, dry_run, &content_callback, &path_callback, &mut result)?;
    } else if target.is_dir() {
        process_directory_recursive_shapes_interactive(target, &templater, process_paths, process_contents, dry_run, &content_callback, &path_callback, &mut result)?;
    } else {
        anyhow::bail!("Target does not exist or is not a file or directory: {:?}", target);
    }
    
    info!(
        "Interactive shapes processing complete: {} files processed, {} paths renamed, {} content changes",
        result.files_processed, result.paths_renamed, result.content_changes
    );
    
    Ok(result)
}

fn process_directory_contents_recursive_shapes(
    dir: &Path,
    templater: &CaseShapeTemplater,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Processing shapes directory contents: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Collect directories and files separately for depth-first processing
    let mut directories = Vec::new();
    let mut files = Vec::new();
    
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }
    
    // First, recursively process subdirectories' CONTENTS (depth-first)
    for dir_path in &directories {
        process_directory_contents_recursive_shapes(dir_path, templater, process_paths, process_contents, dry_run, result)?;
    }
    
    // Then rename subdirectories in reverse order (deepest paths first)
    // This establishes the new directory structure before processing files
    if process_paths {
        directories.reverse(); // Process in reverse order for safety
        for dir_path in &directories {
            // Only use component replacement to rename directory within its current parent
            if let Some(new_name) = templater.process_path_component(dir_path) {
                let new_path = dir_path.parent().unwrap().join(&new_name);
                
                if dry_run {
                    info!("Would rename directory: {:?} -> {:?}", dir_path, new_path);
                } else {
                    info!("Renaming directory: {:?} -> {:?}", dir_path, new_path);
                    fs::rename(dir_path, &new_path)?;
                }
                result.paths_renamed += 1;
            }
        }
    }
    
    // Finally, process files in current directory after directories are renamed
    for file_path in &files {
        process_file_shapes(file_path, templater, process_paths, process_contents, dry_run, result)?;
    }
    
    Ok(())
}

fn rename_target_directory_shapes(
    target: &Path,
    templater: &CaseShapeTemplater,
    _process_paths: bool,
    _process_contents: bool,
    dry_run: bool,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Checking if target directory needs shapes renaming: {:?}", target);
    
    // Only use component replacement to rename target directory within its current parent
    if let Some(new_name) = templater.process_path_component(target) {
        let new_path = target.parent().unwrap().join(&new_name);
        
        if dry_run {
            info!("Would rename target directory: {:?} -> {:?}", target, new_path);
        } else {
            info!("Renaming target directory: {:?} -> {:?}", target, new_path);
            fs::rename(target, &new_path)?;
        }
        result.paths_renamed += 1;
    }
    
    Ok(())
}

fn process_directory_recursive_shapes(
    dir: &Path,
    templater: &CaseShapeTemplater,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Processing shapes directory: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Collect directories and files separately for depth-first processing
    let mut directories = Vec::new();
    let mut files = Vec::new();
    
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }
    
    // First, recursively process all subdirectories' CONTENTS (but don't rename the subdirectories yet)
    for dir_path in &directories {
        process_directory_contents_recursive_shapes(dir_path, templater, process_paths, process_contents, dry_run, result)?;
    }
    
    // Then process files in current directory
    for file_path in &files {
        process_file_shapes(file_path, templater, process_paths, process_contents, dry_run, result)?;
    }
    
    // Finally, rename subdirectories in reverse order (deepest paths first)
    // This ensures we rename child directories only after all their contents are processed
    if process_paths {
        directories.reverse(); // Process in reverse order for safety
        for dir_path in &directories {
            // Only use component replacement to rename directory within its current parent
            if let Some(new_name) = templater.process_path_component(dir_path) {
                let new_path = dir_path.parent().unwrap().join(&new_name);
                
                if dry_run {
                    info!("Would rename directory: {:?} -> {:?}", dir_path, new_path);
                } else {
                    info!("Renaming directory: {:?} -> {:?}", dir_path, new_path);
                    fs::rename(dir_path, &new_path)?;
                }
                result.paths_renamed += 1;
            }
        }
    }
    
    Ok(())
}

fn process_file_shapes(
    file_path: &Path,
    templater: &CaseShapeTemplater,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    result: &mut TemplatizeResult,
) -> Result<()> {
    debug!("Processing shapes file: {:?}", file_path);
    result.files_processed += 1;
    
    let mut content_changed = false;
    let mut path_changed = false;
    
    // Process file contents
    if process_contents {
        if let Ok(content) = fs::read_to_string(file_path) {
            if let Some(new_content) = templater.process_content(&content) {
                if dry_run {
                    info!("Would update contents of: {:?}", file_path);
                } else {
                    info!("Updating contents of: {:?}", file_path);
                    fs::write(file_path, new_content)?;
                }
                content_changed = true;
            }
        } else {
            debug!("Skipping binary file: {:?}", file_path);
        }
    }
    
    // Process file path (only rename within current directory)
    if process_paths {
        if let Some(new_name) = templater.process_path_component(file_path) {
            let new_path = file_path.parent().unwrap().join(&new_name);
            
            if dry_run {
                info!("Would rename file: {:?} -> {:?}", file_path, new_path);
            } else {
                info!("Renaming file: {:?} -> {:?}", file_path, new_path);
                fs::rename(file_path, &new_path)?;
            }
            path_changed = true;
        }
    }
    
    if content_changed {
        result.content_changes += 1;
    }
    if path_changed {
        result.paths_renamed += 1;
    }
    
    Ok(())
}

fn process_directory_recursive_shapes_interactive<F, G>(
    dir: &Path,
    templater: &CaseShapeTemplater,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    content_callback: &F,
    path_callback: &G,
    result: &mut TemplatizeResult,
) -> Result<()>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
    G: Fn(&Path, &Path, &str) -> Result<bool>,
{
    debug!("Processing shapes directory: {:?}", dir);
    
    let entries: Vec<_> = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, _>>()?;
    
    // Collect directories and files separately for depth-first processing
    let mut directories = Vec::new();
    let mut files = Vec::new();
    
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            directories.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }
    
    // First, recursively process subdirectories (depth-first)
    for dir_path in &directories {
        process_directory_recursive_shapes_interactive(dir_path, templater, process_paths, process_contents, dry_run, content_callback, path_callback, result)?;
    }
    
    // Then process files in current directory
    for file_path in &files {
        process_file_shapes_interactive(file_path, templater, process_paths, process_contents, dry_run, content_callback, path_callback, result)?;
    }
    
    // Finally, process directory renaming in reverse order (deepest first)
    // This ensures we rename child directories before parent directories
    if process_paths {
        directories.reverse(); // Process in reverse order for safety
        for dir_path in &directories {
            // Only use component replacement to rename directory within its current parent
            if let Some(new_name) = templater.process_path_component(dir_path) {
                let new_path = dir_path.parent().unwrap().join(&new_name);
                
                if path_callback(dir_path, &new_path, "Directory")? {
                    if dry_run {
                        info!("Would rename directory: {:?} -> {:?}", dir_path, new_path);
                    } else {
                        info!("Renaming directory: {:?} -> {:?}", dir_path, new_path);
                        fs::rename(dir_path, &new_path)?;
                    }
                    result.paths_renamed += 1;
                }
            }
        }
    }
    
    Ok(())
}

fn process_file_shapes_interactive<F, G>(
    file_path: &Path,
    templater: &CaseShapeTemplater,
    process_paths: bool,
    process_contents: bool,
    dry_run: bool,
    content_callback: &F,
    path_callback: &G,
    result: &mut TemplatizeResult,
) -> Result<()>
where
    F: Fn(&Path, &str, &str, &str) -> Result<bool>,
    G: Fn(&Path, &Path, &str) -> Result<bool>,
{
    debug!("Processing shapes file: {:?}", file_path);
    result.files_processed += 1;
    
    let mut content_changed = false;
    let mut path_changed = false;
    
    // Process file contents
    if process_contents {
        if let Ok(content) = fs::read_to_string(file_path) {
            if let Some(new_content) = templater.process_content(&content) {
                if content_callback(file_path, &content, &new_content, "Case shape content change")? {
                    if dry_run {
                        info!("Would update contents of: {:?}", file_path);
                    } else {
                        info!("Updating contents of: {:?}", file_path);
                        fs::write(file_path, new_content)?;
                    }
                    content_changed = true;
                }
            }
        } else {
            debug!("Skipping binary file: {:?}", file_path);
        }
    }
    
    // Process file path (only rename within current directory)
    if process_paths {
        if let Some(new_name) = templater.process_path_component(file_path) {
            let new_path = file_path.parent().unwrap().join(&new_name);
            
            if path_callback(file_path, &new_path, "File")? {
                if dry_run {
                    info!("Would rename file: {:?} -> {:?}", file_path, new_path);
                } else {
                    info!("Renaming file: {:?} -> {:?}", file_path, new_path);
                    fs::rename(file_path, &new_path)?;
                }
                path_changed = true;
            }
        }
    }
    
    if content_changed {
        result.content_changes += 1;
    }
    if path_changed {
        result.paths_renamed += 1;
    }
    
    Ok(())
}