// Simple test of the modern Neksis 2025 parser
use neksisc::modern_lexer::Lexer;
use neksisc::modern_parser::Parser;

fn main() {
    println!("=== Testing Modern Neksis 2025 Parser ===");
    
    // Test cases that should NOT crash the parser (fixing fragility)
    let test_cases = vec![
        "2 + 3 * 4",                    // Basic arithmetic
        "let x = 42;",                  // Variable declaration
        "if x > 0 { print(x); }",       // Conditional
        "fn test() { return 5; }",      // Function
        "x.method(1, 2, 3)",           // Method call
        "arr[0] = value",              // Array access
        "{ let a = 1; let b = 2; }",   // Block
        "x && y || z",                 // Logical operations
        "1..10",                       // Range
        "\"hello\" + \" world\"",       // String concatenation
    ];
    
    for (i, test_case) in test_cases.iter().enumerate() {
        println!("\n--- Test Case {}: {} ---", i + 1, test_case);
        
        match test_parser(test_case) {
            Ok(()) => println!("✅ PASSED: Parser handled successfully"),
            Err(e) => println!("❌ FAILED: {}", e),
        }
    }
    
    println!("\n=== Testing complete! ===");
}

fn test_parser(input: &str) -> Result<(), String> {
    // Create lexer
    let mut lexer = Lexer::new(input);
    
    // Tokenize
    let tokens = lexer.tokenize();
    println!("Tokens: {:?}", tokens.iter().take(5).collect::<Vec<_>>());
    
    // Create parser
    let mut parser = Parser::new(tokens);
    
    // Parse
    match parser.parse() {
        Ok(program) => {
            println!("Parsed {} statements successfully", program.statements.len());
            Ok(())
        },
        Err(e) => Err(format!("Parsing failed: {:?}", e)),
    }
}
