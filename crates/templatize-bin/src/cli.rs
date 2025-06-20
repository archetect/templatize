use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "templatize")]
#[command(version)]
#[command(about = "Convert existing projects into Jinja2 templates")]
#[command(long_about = "A CLI tool that transforms existing projects into reusable Jinja2 templates by performing selective word replacements and generating appropriate template syntax.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Replace exact token with exact Jinja2 syntax")]
    Exact {
        #[arg(help = "Exact token to replace")]
        token: String,

        #[arg(help = "Exact Jinja2 syntax to replace it with")]
        replacement: String,

        #[arg(short, long, help = "Templatize file and directory paths")]
        path: bool,

        #[arg(short, long, help = "Templatize file contents")]
        contents: bool,

        #[arg(help = "Target directory (defaults to current directory)")]
        target: Option<PathBuf>,

        #[arg(long, help = "Perform a dry run without making changes")]
        dry_run: bool,

        #[arg(short, long, help = "Interactive mode - prompt for each change")]
        interactive: bool,
    },

    #[command(about = "Replace compound words with case shape variants")]
    Shapes {
        #[arg(help = "Compound word token to replace (e.g., 'example-name')")]
        token: String,

        #[arg(help = "Compound word Jinja2 replacement (e.g., '{{ project-name }}')")]
        replacement: String,

        #[arg(short, long, help = "Templatize file and directory paths")]
        path: bool,

        #[arg(short, long, help = "Templatize file contents")]
        contents: bool,

        #[arg(help = "Target directory (defaults to current directory)")]
        target: Option<PathBuf>,

        #[arg(long, help = "Perform a dry run without making changes")]
        dry_run: bool,

        #[arg(short, long, help = "Interactive mode - prompt for each change")]
        interactive: bool,
    },

    #[command(about = "Escape Jinja2 syntax in file contents")]
    Escape {
        #[arg(help = "Target file or directory (defaults to current directory)")]
        target: Option<PathBuf>,

        #[arg(long, help = "Perform a dry run without making changes")]
        dry_run: bool,

        #[arg(short, long, help = "Interactive mode - prompt for each change")]
        interactive: bool,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_exact_command() {
        let args = vec![
            "templatize",
            "exact",
            "example-name",
            "{{ project-name }}",
            "--path",
            "--contents",
        ];
        
        let cli = Cli::try_parse_from(args).unwrap();
        
        match cli.command {
            Commands::Exact { token, replacement, path, contents, .. } => {
                assert_eq!(token, "example-name");
                assert_eq!(replacement, "{{ project-name }}");
                assert!(path);
                assert!(contents);
            }
            _ => panic!("Expected Exact command"),
        }
    }

    #[test]
    fn test_shapes_command() {
        let args = vec![
            "templatize",
            "shapes",
            "example-name",
            "{{ project-name }}",
            "--path",
            "--contents",
        ];
        
        let cli = Cli::try_parse_from(args).unwrap();
        
        match cli.command {
            Commands::Shapes { token, replacement, path, contents, .. } => {
                assert_eq!(token, "example-name");
                assert_eq!(replacement, "{{ project-name }}");
                assert!(path);
                assert!(contents);
            }
            _ => panic!("Expected Shapes command"),
        }
    }

    #[test]
    fn test_escape_command() {
        let args = vec![
            "templatize",
            "escape",
            "/path/to/file.txt",
            "--interactive",
        ];
        
        let cli = Cli::try_parse_from(args).unwrap();
        
        match cli.command {
            Commands::Escape { target, interactive, .. } => {
                assert_eq!(target, Some(PathBuf::from("/path/to/file.txt")));
                assert!(interactive);
            }
            _ => panic!("Expected Escape command"),
        }
    }
}