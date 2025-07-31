use neksisc::cli::CLI;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CLI::new();
    
    match cli.run() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
} 