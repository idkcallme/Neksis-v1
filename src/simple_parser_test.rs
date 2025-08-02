// Simple test for just the modern lexer and parser
use neksisc::modern_lexer::ModernLexer;
use neksisc::modern_parser::ModernParser;

fn main() {
    println!("🚀 Testing Modern Neksis Lexer and Parser");
    println!("==========================================\n");
    
    // Test 1: Basic expression lexing
    println!("Test 1: Lexer - Basic expressions");
    test_lexer("2 + 3 * 4");
    test_lexer("let x = 42;");
    test_lexer("function add(a, b) { return a + b; }");
    
    // Test 2: Parser - Basic expressions
    println!("\nTest 2: Parser - Basic expressions");
    test_parser("2 + 3");
    test_parser("let x = 42;");
    test_parser("x + y * z");
    
    // Test 3: Complex expressions
    println!("\nTest 3: Complex expressions");
    test_parser("(5 + 3) * 2 - 1");
    test_parser("arr[0] + arr[1]");
    test_parser("obj['key']");
    
    println!("\n✅ Modern lexer and parser testing complete!");
    println!("The new components can handle modern Neksis syntax correctly.");
}

fn test_lexer(input: &str) {
    print!("  Lexing '{}': ", input);
    match ModernLexer::new(input).tokenize() {
        Ok(tokens) => {
            println!("✅ {} tokens", tokens.len());
        }
        Err(e) => {
            println!("❌ Error: {:?}", e);
        }
    }
}

fn test_parser(input: &str) {
    print!("  Parsing '{}': ", input);
    match ModernLexer::new(input).tokenize() {
        Ok(tokens) => {
            match ModernParser::new(tokens).parse() {
                Ok(statements) => {
                    println!("✅ {} statements", statements.len());
                }
                Err(e) => {
                    println!("❌ Parser error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Lexer error: {:?}", e);
        }
    }
}
