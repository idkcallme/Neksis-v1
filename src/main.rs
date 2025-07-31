use std::env;
use std::fs;
use std::path::Path;
use neksisc::FastCompiler;
use neksisc::compiler::CompilerOptions;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: neksis <file.nx> [--emit-asm]");
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    let emit_asm = args.iter().any(|arg| arg == "--emit-asm");
    
    if !Path::new(file_path).exists() {
        eprintln!("Error: File '{}' not found", file_path);
        std::process::exit(1);
    }
    
    if !file_path.ends_with(".nx") {
        eprintln!("Error: File must have .nx extension");
        std::process::exit(1);
    }
    
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };
    
    if emit_asm {
        // Set the environment variable for this process
        env::set_var("NEKSIS_EMIT_ASM", "1");
    }
    
    let mut compiler = FastCompiler::new(CompilerOptions::default());
    
    if let Err(e) = compiler.compile(&source) {
        eprintln!("Execution failed: {}", e);
        std::process::exit(1);
    }
} 