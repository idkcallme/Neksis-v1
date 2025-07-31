use neksisc::{FastCompiler, CompilerOptions};

fn main() {
    println!("=== Quick Compiler Test ===");
    
    let source = "let x = 42;";
    let mut compiler = FastCompiler::new(CompilerOptions::default());
    
    match compiler.compile(source) {
        Ok(_) => println!("✅ Compiler test successful!"),
        Err(e) => println!("❌ Compiler test failed: {}", e),
    }
    
    println!("=== Test Complete ===");
} 