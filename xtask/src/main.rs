use std::process;

use anyhow::Result;
use clap::{ArgMatches, Command};

const SERVICE_NAME: &str = "templatize";

fn main() -> Result<()> {
    let args = clap::command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("install").about("Install templatize binary locally"))
        .subcommand(
            Command::new("run")
                .about("Build and run templatize with arguments")
                .trailing_var_arg(true)
                .allow_hyphen_values(true)
                .arg(clap::Arg::new("args")
                    .help("Arguments to pass to templatize")
                    .action(clap::ArgAction::Append)
                    .num_args(0..))
        )
        .subcommand(
            Command::new("test")
                .about("Test Operations")
                .subcommand(Command::new("all").about("Run all tests for the entire project"))
                .subcommand(Command::new("core").about("Run tests for templatize-core"))
                .subcommand(Command::new("bin").about("Run tests for templatize-bin"))
                .subcommand(Command::new("integration").about("Run integration tests"))
        )
        .subcommand(
            Command::new("docker")
                .about("Docker Operations")
                .subcommand(Command::new("build").about("Builds an application Docker image."))
                .subcommand(Command::new("rmi").about("Removes the application Docker image.")),
        )
        .get_matches();

    match args.subcommand() {
        Some(("docker", args)) => handle_docker_commands(args),
        Some(("install", args)) => handle_install_command(args),
        Some(("run", args)) => handle_run_command(args),
        Some(("test", args)) => handle_test_commands(args),
        Some((command, _)) => anyhow::bail!("Unexpected command: {command}"),
        None => anyhow::bail!("Expected subcommand"),
    }
}

fn handle_install_command(_args: &ArgMatches) -> Result<()> {
    println!("Installing templatize...");
    let status = process::Command::new("cargo")
        .args(["install", "--path", "crates/templatize-bin"])
        .status()?;

    if status.success() {
        println!("✓ templatize installed successfully");
    } else {
        anyhow::bail!("Failed to install templatize");
    }

    Ok(())
}

fn handle_run_command(args: &ArgMatches) -> Result<()> {
    println!("Building and running templatize...");
    
    // Get any additional arguments passed to run command
    let run_args: Vec<String> = args.get_many::<String>("args")
        .map_or(Vec::new(), |vals| vals.cloned().collect());

    let mut command = process::Command::new("cargo");
    command.args(["run", "--bin", "templatize", "--"]);
    
    if !run_args.is_empty() {
        command.args(&run_args);
    }

    let status = command.status()?;

    if !status.success() {
        anyhow::bail!("Failed to run templatize");
    }

    Ok(())
}

fn handle_docker_commands(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("build", _args)) => docker_build(),
        Some(("rmi", _args)) => docker_rmi(),
        _ => Ok(()),
    }
}

fn docker_build() -> Result<()> {
    process::Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(SERVICE_NAME)
        .arg(".")
        .spawn()?
        .wait()?;

    Ok(())
}

fn docker_rmi() -> Result<()> {
    process::Command::new("docker")
        .arg("rmi")
        .arg(SERVICE_NAME)
        .spawn()?
        .wait()?;

    Ok(())
}

fn handle_test_commands(args: &ArgMatches) -> Result<()> {
    match args.subcommand() {
        Some(("all", _args)) => test_all(),
        Some(("core", _args)) => test_core(),
        Some(("bin", _args)) => test_bin(),
        Some(("integration", _args)) => test_integration(),
        _ => {
            println!("Available test commands:");
            println!("  all          - Run all tests for the entire project");
            println!("  core         - Run tests for templatize-core");
            println!("  bin          - Run tests for templatize-bin");
            println!("  integration  - Run integration tests");
            Ok(())
        }
    }
}

fn test_all() -> Result<()> {
    println!("🧪 Running all tests for the templatize project...\n");
    
    let mut all_passed = true;
    
    // Test 1: Core library tests
    println!("📚 Running templatize-core tests...");
    let core_result = test_core_internal();
    if core_result.is_err() {
        all_passed = false;
        println!("❌ Core tests failed: {:?}", core_result.err());
    } else {
        println!("✅ Core tests passed");
    }
    println!();
    
    // Test 2: Binary crate tests
    println!("🔧 Running templatize-bin tests...");
    let bin_result = test_bin_internal();
    if bin_result.is_err() {
        all_passed = false;
        println!("❌ Binary tests failed: {:?}", bin_result.err());
    } else {
        println!("✅ Binary tests passed");
    }
    println!();
    
    // Test 3: Workspace-wide tests
    println!("🏗️  Running workspace tests...");
    let workspace_result = test_workspace_internal();
    if workspace_result.is_err() {
        all_passed = false;
        println!("❌ Workspace tests failed: {:?}", workspace_result.err());
    } else {
        println!("✅ Workspace tests passed");
    }
    println!();
    
    // Test 4: Doc tests
    println!("📖 Running documentation tests...");
    let doc_result = test_docs_internal();
    if doc_result.is_err() {
        all_passed = false;
        println!("❌ Documentation tests failed: {:?}", doc_result.err());
    } else {
        println!("✅ Documentation tests passed");
    }
    println!();
    
    // Test 5: Integration tests
    println!("🔗 Running integration tests...");
    let integration_result = test_integration_internal();
    if integration_result.is_err() {
        all_passed = false;
        println!("❌ Integration tests failed: {:?}", integration_result.err());
    } else {
        println!("✅ Integration tests passed");
    }
    println!();
    
    // Test 6: CLI validation
    println!("⚙️  Running CLI validation tests...");
    let cli_result = test_cli_internal();
    if cli_result.is_err() {
        all_passed = false;
        println!("❌ CLI tests failed: {:?}", cli_result.err());
    } else {
        println!("✅ CLI tests passed");
    }
    println!();
    
    // Summary
    if all_passed {
        println!("🎉 All tests passed successfully!");
        println!("✨ The templatize project is ready for use.");
    } else {
        println!("💥 Some tests failed. Please check the output above.");
        anyhow::bail!("Test suite failed");
    }
    
    Ok(())
}

fn test_core() -> Result<()> {
    println!("🧪 Running templatize-core tests...");
    test_core_internal()
}

fn test_bin() -> Result<()> {
    println!("🧪 Running templatize-bin tests...");
    test_bin_internal()
}

fn test_integration() -> Result<()> {
    println!("🧪 Running integration tests...");
    test_integration_internal()
}

// Internal test functions that return Results
fn test_core_internal() -> Result<()> {
    let status = process::Command::new("cargo")
        .args(["test", "--package", "templatize-core"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Core tests failed");
    }
    Ok(())
}

fn test_bin_internal() -> Result<()> {
    let status = process::Command::new("cargo")
        .args(["test", "--package", "templatize-bin"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Binary tests failed");
    }
    Ok(())
}

fn test_workspace_internal() -> Result<()> {
    let status = process::Command::new("cargo")
        .args(["test", "--workspace"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Workspace tests failed");
    }
    Ok(())
}

fn test_docs_internal() -> Result<()> {
    // Only run doc tests for crates that have library targets
    let status = process::Command::new("cargo")
        .args(["test", "--doc", "--package", "templatize-core"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Documentation tests failed");
    }
    Ok(())
}

fn test_integration_internal() -> Result<()> {
    // Build the binary first
    let build_status = process::Command::new("cargo")
        .args(["build", "--bin", "templatize"])
        .status()?;

    if !build_status.success() {
        anyhow::bail!("Failed to build templatize binary");
    }

    // Test basic CLI functionality
    let help_status = process::Command::new("cargo")
        .args(["run", "--bin", "templatize", "--", "--help"])
        .status()?;

    if !help_status.success() {
        anyhow::bail!("CLI help command failed");
    }

    // Test subcommand help
    let exact_help_status = process::Command::new("cargo")
        .args(["run", "--bin", "templatize", "--", "exact", "--help"])
        .status()?;

    if !exact_help_status.success() {
        anyhow::bail!("CLI exact help command failed");
    }

    Ok(())
}

fn test_cli_internal() -> Result<()> {
    // Test CLI structure validation
    let status = process::Command::new("cargo")
        .args(["run", "--bin", "templatize", "--", "--version"])
        .status()?;

    if !status.success() {
        anyhow::bail!("CLI version command failed");
    }

    Ok(())
}