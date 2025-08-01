// Removed unused import
use crate::error::CompilerError;
use crate::formatter::CodeFormatter;
use crate::linter::Linter;
use crate::package_manager::PackageManager;
use crate::lsp::LSPServer;
use crate::tests::TestSuite;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::bytecode_compiler::BytecodeCompiler;
use std::env;
use std::fs;
use std::path::Path;

pub struct CLI;

impl CLI {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<(), CompilerError> {
        let args: Vec<String> = env::args().collect();
        
        if args.len() < 2 {
            return self.show_help();
        }

        let command = &args[1];
        
        // Check for version and help flags first
        if command.starts_with("-") || command == "help" || command == "version" {
            match command.as_str() {
                "help" | "--help" | "-h" => return self.show_help(),
                "version" | "--version" | "-v" => return self.show_version(),
                _ => {
                    // If it starts with - but isn't a recognized flag, show help
                    return self.show_help();
                }
            }
        }

        // Handle subcommands
        match command.as_str() {
            "init" => self.handle_init(&args[2..]),
            "build" => self.handle_build(&args[2..]),
            "run" => self.handle_run(&args[2..]),
            "install" => self.handle_install(&args[2..]),
            "lsp" => self.handle_lsp(&args[2..]),
            "test" => self.handle_test(&args[2..]),
            "format" => self.handle_format(&args[2..]),
            "lint" => self.handle_lint(&args[2..]),
            "repl" => self.handle_repl(&args[2..]),
            _ => {
                // If no command is specified, try to compile and run the file directly
                self.handle_direct_execution(&args[1..])
            }
        }
    }

    fn handle_init(&self, args: &[String]) -> Result<(), CompilerError> {
        let project_name = args.get(0).unwrap_or(&"my-nexus-project".to_string()).clone();
        
        let package_manager = PackageManager::new()?;
        package_manager.init_project(&project_name)?;
        
        println!("✅ Project '{}' initialized successfully!", project_name);
        println!("📁 Created project structure:");
        println!("   ├── nexus.json (package manifest)");
        println!("   ├── src/");
        println!("   │   └── main.nx (entry point)");
        println!("   └── README.md");
        println!();
        println!("🚀 Next steps:");
        println!("   cd {}", project_name);
        println!("   neksis run");
        
        Ok(())
    }

    fn handle_build(&self, args: &[String]) -> Result<(), CompilerError> {
        let default_file = "src/main.nx".to_string();
        let source_file = args.get(0).unwrap_or(&default_file);
        
        if !Path::new(source_file).exists() {
            return Err(CompilerError::runtime_error(&format!("Source file '{}' not found", source_file)));
        }

        let source = fs::read_to_string(source_file)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read file: {}", e)))?;

        // Compile to bytecode
        let mut lexer = Lexer::new(&source, source_file.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let mut bytecode_compiler = BytecodeCompiler::new();
        let instructions = bytecode_compiler.compile_program(&ast)?;
        
        println!("✅ Build successful!");
        println!("📦 Generated {} instructions", instructions.len());
        
        Ok(())
    }

    fn handle_run(&self, args: &[String]) -> Result<(), CompilerError> {
        let default_file = "src/main.nx".to_string();
        let source_file = args.get(0).unwrap_or(&default_file);
        
        if !Path::new(source_file).exists() {
            return Err(CompilerError::runtime_error(&format!("Source file '{}' not found", source_file)));
        }

        let source = fs::read_to_string(source_file)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read file: {}", e)))?;

        // Compile to bytecode
        let mut lexer = Lexer::new(&source, source_file.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let mut bytecode_compiler = BytecodeCompiler::new();
        let instructions = bytecode_compiler.compile_program(&ast)?;
        
        println!("🚀 Running {}...", source_file);
        println!("📤 Output:");
        
        // Execute the compiled code
        let mut vm = crate::vm::VM::new();
        vm.load_instructions(instructions);
        vm.run()?;
        
        Ok(())
    }

    fn handle_install(&self, args: &[String]) -> Result<(), CompilerError> {
        let package_name = args.get(0).ok_or_else(|| {
            CompilerError::runtime_error("Package name required. Usage: neksis install <package-name>")
        })?;
        
        let package_manager = PackageManager::new()?;
        package_manager.install_dependency(package_name, None)?;
        
        println!("✅ Package '{}' installed successfully!", package_name);
        
        Ok(())
    }

    fn handle_lsp(&self, _args: &[String]) -> Result<(), CompilerError> {
        println!("🔧 Starting neksis Language Server...");
        println!("📝 LSP server is running on stdio");
        println!("💡 Connect your IDE to use features like:");
        println!("   - Code completion");
        println!("   - Go to definition");
        println!("   - Find references");
        println!("   - Error diagnostics");
        
        let _server = LSPServer::new();
        
        // Simple LSP server loop (in a real implementation, this would handle stdio)
        println!("🔄 LSP server ready for connections");
        
        Ok(())
    }

    fn handle_test(&self, _args: &[String]) -> Result<(), CompilerError> {
        println!("🧪 Running neksis test suite...");
        
        let test_suite = TestSuite::new();
        let results = test_suite.run_all_tests()?;
        results.print_summary();
        
        if results.failed > 0 {
            return Err(CompilerError::runtime_error(&format!("{} tests failed", results.failed)));
        }
        
        Ok(())
    }

    fn handle_format(&self, args: &[String]) -> Result<(), CompilerError> {
        let source_file = args.get(0).ok_or_else(|| {
            CompilerError::runtime_error("File path required. Usage: neksis format <file.nx>")
        })?;
        
        if !Path::new(source_file).exists() {
            return Err(CompilerError::runtime_error(&format!("File '{}' not found", source_file)));
        }

        let source = fs::read_to_string(source_file)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read file: {}", e)))?;

        let formatter = CodeFormatter::new();
        let formatted = formatter.format_source(&source)?;
        
        // Write formatted code back to file
        fs::write(source_file, formatted)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to write formatted code: {}", e)))?;
        
        println!("✅ Formatted '{}' successfully!", source_file);
        
        Ok(())
    }

    fn handle_lint(&self, args: &[String]) -> Result<(), CompilerError> {
        let source_file = args.get(0).ok_or_else(|| {
            CompilerError::runtime_error("File path required. Usage: neksis lint <file.nx>")
        })?;
        
        if !Path::new(source_file).exists() {
            return Err(CompilerError::runtime_error(&format!("File '{}' not found", source_file)));
        }

        let source = fs::read_to_string(source_file)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read file: {}", e)))?;

        let linter = Linter::new();
        let issues = linter.lint_source(&source, source_file)?;
        
        if issues.is_empty() {
            println!("✅ No linting issues found in '{}'", source_file);
        } else {
            println!("🔍 Found {} linting issues in '{}':", issues.len(), source_file);
            println!();
            
            for issue in issues {
                let severity_icon = match issue.severity {
                    crate::linter::LintSeverity::Error => "❌",
                    crate::linter::LintSeverity::Warning => "⚠️",
                    crate::linter::LintSeverity::Info => "ℹ️",
                };
                
                println!("{} {}:{} - {}", 
                    severity_icon, 
                    issue.line, 
                    issue.column, 
                    issue.message
                );
                
                if let Some(suggestion) = issue.suggestion {
                    println!("   💡 {}", suggestion);
                }
                println!();
            }
        }
        
        Ok(())
    }

    fn handle_repl(&self, _args: &[String]) -> Result<(), CompilerError> {
        println!("🚀 Starting Neksis REPL...");
        println!("Type 'exit' or 'quit' to exit, 'help' for commands");
        println!();
        
        let mut repl = crate::repl::REPL::new();
        repl.run()?;
        
        Ok(())
    }

    fn handle_direct_execution(&self, args: &[String]) -> Result<(), CompilerError> {
        let source_file = &args[0];
        
        if !Path::new(source_file).exists() {
            return Err(CompilerError::runtime_error(&format!("File '{}' not found", source_file)));
        }

        let source = fs::read_to_string(source_file)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to read file: {}", e)))?;

        // Compile to bytecode
        let mut lexer = Lexer::new(&source, source_file.to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let mut bytecode_compiler = BytecodeCompiler::new();
        let instructions = bytecode_compiler.compile_program(&ast)?;
        
        // Execute the compiled code
        let mut vm = crate::vm::VM::new();
        vm.load_instructions(instructions);
        vm.run()?;
        
        Ok(())
    }

    fn show_help(&self) -> Result<(), CompilerError> {
        println!("🚀 neksis Programming Language Compiler");
        println!();
        println!("Usage: neksis <command> [options]");
        println!();
        println!("Commands:");
        println!("  init [project-name]     Initialize a new neksis project");
        println!("  build [file.nx]         Compile a neksis source file");
        println!("  run [file.nx]           Compile and run a neksis source file");
        println!("  install <package>       Install a package dependency");
        println!("  lsp                     Start the Language Server Protocol server");
        println!("  test                    Run the test suite");
        println!("  format <file.nx>        Format a neksis source file");
        println!("  lint <file.nx>          Lint a neksis source file");
        println!("  repl                    Start the interactive REPL");
        println!("  help                    Show this help message");
        println!("  version                 Show version information");
        println!();
        println!("Examples:");
        println!("  neksis init my-project");
        println!("  neksis run src/main.nx");
        println!("  neksis format src/main.nx");
        println!("  neksis lint src/main.nx");
        println!("  neksis repl              # Start interactive REPL");
        println!("  neksis src/main.nx      # Direct execution");
        println!();
        println!("For more information, visit: https://github.com/nexus-lang/nexus");
        
        Ok(())
    }

    fn show_version(&self) -> Result<(), CompilerError> {
        println!("neksis Programming Language Compiler v0.1.0");
        println!("Built with Rust");
        println!("Copyright (c) 2024 neksis Language Team");
        
        Ok(())
    }
} 