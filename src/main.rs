use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::*;

#[derive(Parser)]
#[command(name = "mcp-helper")]
#[command(author = "Your Name <you@example.com>")]
#[command(version = "1.0")]
#[command(about = "MCP Helper - A CLI tool for MCP operations", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, help = "Enable verbose output")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Say hello to someone")]
    Hello {
        #[arg(short, long, default_value = "World")]
        name: String,
        
        #[arg(short, long, help = "Number of times to greet")]
        count: Option<u32>,
    },
    
    #[command(about = "Perform calculations")]
    Calculate {
        #[command(subcommand)]
        operation: Operation,
    },
    
    #[command(about = "Display information")]
    Info {
        #[arg(short, long, help = "Show detailed information")]
        detailed: bool,
    },
}

#[derive(Subcommand)]
enum Operation {
    #[command(about = "Add two numbers")]
    Add {
        #[arg(help = "First number")]
        a: f64,
        #[arg(help = "Second number")]
        b: f64,
    },
    #[command(about = "Subtract two numbers")]
    Subtract {
        #[arg(help = "First number")]
        a: f64,
        #[arg(help = "Second number")]
        b: f64,
    },
    #[command(about = "Multiply two numbers")]
    Multiply {
        #[arg(help = "First number")]
        a: f64,
        #[arg(help = "Second number")]
        b: f64,
    },
    #[command(about = "Divide two numbers")]
    Divide {
        #[arg(help = "Dividend")]
        a: f64,
        #[arg(help = "Divisor")]
        b: f64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("{}", "Verbose mode enabled".dimmed());
    }

    match &cli.command {
        Some(Commands::Hello { name, count }) => {
            let times = count.unwrap_or(1);
            for _ in 0..times {
                println!("{} {}", "Hello,".green().bold(), name.cyan());
            }
        }
        Some(Commands::Calculate { operation }) => {
            let result = match operation {
                Operation::Add { a, b } => {
                    println!("{} + {} = {}", a, b, (a + b).to_string().yellow().bold());
                    a + b
                }
                Operation::Subtract { a, b } => {
                    println!("{} - {} = {}", a, b, (a - b).to_string().yellow().bold());
                    a - b
                }
                Operation::Multiply { a, b } => {
                    println!("{} × {} = {}", a, b, (a * b).to_string().yellow().bold());
                    a * b
                }
                Operation::Divide { a, b } => {
                    if *b == 0.0 {
                        eprintln!("{}", "Error: Division by zero!".red().bold());
                        return Ok(());
                    }
                    println!("{} ÷ {} = {}", a, b, (a / b).to_string().yellow().bold());
                    a / b
                }
            };
            
            if cli.verbose {
                println!("{} {}", "Result:".dimmed(), result.to_string().white());
            }
        }
        Some(Commands::Info { detailed }) => {
            println!("{}", "=== CLI App Information ===".blue().bold());
            println!("Name: {}", "mcp-helper".green());
            println!("Version: {}", "1.0.0".yellow());
            
            if *detailed {
                println!("\n{}", "Features:".underline());
                println!("• {} - Greet someone with customizable repetition", "hello".cyan());
                println!("• {} - Perform basic arithmetic operations", "calculate".cyan());
                println!("• {} - Display application information", "info".cyan());
                println!("\n{}", "Built with:".underline());
                println!("• {} - Command-line argument parsing", "clap".magenta());
                println!("• {} - Error handling", "anyhow".magenta());
                println!("• {} - Colored terminal output", "colored".magenta());
            }
        }
        None => {
            println!("{}", "No command specified. Use --help for usage information.".yellow());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        // Basic test example
        assert_eq!(2.0 + 2.0, 4.0);
    }
}