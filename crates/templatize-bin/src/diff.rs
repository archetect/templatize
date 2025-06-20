use anyhow::Result;
use inquire::Confirm;
use similar::{ChangeTag, TextDiff};
use std::fmt::Write;

pub fn show_diff_and_confirm(
    file_path: &std::path::Path,
    old_content: &str,
    new_content: &str,
    change_description: &str,
) -> Result<bool> {
    println!("\n📝 {}: {}", change_description, file_path.display());
    
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut output = String::new();
    let mut has_changes = false;
    
    for (i, group) in diff.grouped_ops(3).iter().enumerate() {
        if i > 0 {
            writeln!(output, "{:-^1$}", "", 40)?;
        }
        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("- ", "\x1b[31m"), // Red
                    ChangeTag::Insert => ("+ ", "\x1b[32m"), // Green
                    ChangeTag::Equal => ("  ", "\x1b[0m"),   // Default
                };
                write!(
                    output,
                    "{}{}{}\x1b[0m",
                    style,
                    sign,
                    change.value()
                )?;
                if change.tag() != ChangeTag::Equal {
                    has_changes = true;
                }
            }
        }
    }
    
    if !has_changes {
        println!("No changes detected.");
        return Ok(false);
    }
    
    println!("{}", output);
    
    let apply_change = Confirm::new("Apply this change?")
        .with_default(true)
        .prompt()?;
    
    Ok(apply_change)
}

pub fn show_path_change_and_confirm(
    old_path: &std::path::Path,
    new_path: &std::path::Path,
    change_type: &str,
) -> Result<bool> {
    println!("\n📁 {} rename:", change_type);
    println!("  \x1b[31m- {}\x1b[0m", old_path.display());
    println!("  \x1b[32m+ {}\x1b[0m", new_path.display());
    
    let apply_change = Confirm::new("Apply this rename?")
        .with_default(true)
        .prompt()?;
    
    Ok(apply_change)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_detection() {
        let old_content = "This is old content\nwith multiple lines";
        let new_content = "This is new content\nwith multiple lines";
        
        let diff = TextDiff::from_lines(old_content, new_content);
        let mut has_changes = false;
        
        for group in diff.grouped_ops(3).iter() {
            for op in group {
                for change in diff.iter_changes(op) {
                    if change.tag() != ChangeTag::Equal {
                        has_changes = true;
                        break;
                    }
                }
            }
        }
        
        assert!(has_changes);
    }

    #[test]
    fn test_no_diff_detection() {
        let content = "This is the same content\nwith multiple lines";
        
        let diff = TextDiff::from_lines(content, content);
        let mut has_changes = false;
        
        for group in diff.grouped_ops(3).iter() {
            for op in group {
                for change in diff.iter_changes(op) {
                    if change.tag() != ChangeTag::Equal {
                        has_changes = true;
                        break;
                    }
                }
            }
        }
        
        assert!(!has_changes);
    }
}