# Neksis 

A modern, statically-typed programming language designed for safety, performance, and developer productivity. Neksis combines the best features of languages like Rust, Go, and TypeScript to create a powerful yet accessible programming experience.

## Features

- **Static Type Checking** with type inference
- **Memory Safety** with built-in borrow checker
- **Comprehensive Standard Library** with I/O, math, crypto, networking, and more
- **Modern Development Tools** including formatter, linter, and package manager
- **IDE Support** with Language Server Protocol (LSP)
- **Interactive REPL** for rapid prototyping
- **Cross-platform** compilation and execution

## Installation

### Prerequisites
- Rust 1.70+ (for building from source)
- Git
- A modern operating system (Windows, macOS, Linux)

### Build from Source (Recommended)
```bash
git clone https://github.com/idkcallme/Neksis.git
cd neksis
cargo build --release
cargo install --path .
```

### Verify Installation
```bash
neksis --version
```

## Quick Start

### Hello World
Create a file named `hello.nx`:
```nx
fn main() -> Int {
    println("Hello, World!");
    return 0;
}
```

Run the program:
```bash
neksis run hello.nx
```

### Interactive REPL
```bash
neksis repl
```

## Documentation

- **[Complete Guide](NEKSIS_COMPLETE_GUIDE.txt)** - Comprehensive documentation covering tutorials, reference manual, and API documentation
- **[Implementation Summary](IMPLEMENTATION_SUMMARY.md)** - Technical implementation details
- **[Production Readiness](PRODUCTION_READINESS.md)** - Production deployment information

## Project Structure

```
neksis/
├── neksisc/                 # Main compiler implementation
│   ├── src/
│   │   ├── lexer.rs         # Tokenization
│   │   ├── parser.rs        # AST generation
│   │   ├── semantic.rs      # Type checking
│   │   ├── bytecode_compiler.rs # Code generation
│   │   ├── vm.rs            # Virtual machine
│   │   ├── stdlib/          # Standard library modules
│   │   └── ...
│   └── Cargo.toml
├── docs/                    # Documentation
├── examples/                # Example programs
├── tests/                   # Test files
└── README.md
```

## Testing

Run the test suite:
```bash
cargo test
```

Run specific test files:
```bash
neksis run tests/basic_tests.nx
```

## Development Tools

### Code Formatting
```bash
neksis format program.nx
```

### Linting
```bash
neksis lint program.nx
```

### Package Management
```bash
neksis init my-project
neksis install package-name
```

## Language Features

### Basic Types
- `Int`: 64-bit integers
- `Float`: 64-bit floating-point numbers
- `Bool`: Boolean values
- `String`: UTF-8 encoded strings

### Control Flow
```nx
// If expressions
let result = if x > 40 {
    "large"
} else {
    "small"
};

// Block expressions
let block_result = {
    let temp = 100;
    let temp2 = 50;
    temp + temp2
};
```

### Functions
```nx
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

fn greet(name: String) -> String {
    return "Hello, " + name;
}
```

### Standard Library
- **I/O**: `print()`, `println()`, `read_line()`
- **String**: `len()`, `substring()`, `concat()`
- **Math**: `abs()`, `sqrt()`, `sin()`, `cos()`
- **Crypto**: `hash_sha256()`, `aes_encrypt_128()`
- **Networking**: `http_get()`, `tcp_connect()`
- **Collections**: HashMap, HashSet, Vector, LinkedList

## Advanced Features

- **Type Inference**: Automatic type deduction
- **Memory Safety**: Built-in borrow checker
- **Memory Profiling**: Automatic memory usage tracking
- **Code Formatting**: Automatic code formatting
- **Linting**: Static analysis and style checking
- **Package Management**: Dependency resolution
- **IDE Integration**: LSP support for code completion

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Use Cases

- **Educational Programming**: Computer science courses and learning
- **Rapid Prototyping**: Quick application development
- **System Utilities**: Automation and system tools
- **Web Development**: Server-side applications
- **Embedded Systems**: Resource-constrained environments
- **Application Development**: Full-featured applications

## Performance

- **Fast Compilation**: Incremental compilation with caching
- **Efficient Runtime**: Optimized bytecode execution
- **Memory Optimized**: Automatic memory management with profiling
- **Scalable**: Supports large codebases with modular architecture


## Acknowledgments

Thanks to all contributors and the Rust community for inspiration and tools that made this project possible.

---

**Happy coding with Neksis!**

For more information, see the [Complete Documentation Guide](NEKSIS_COMPLETE_GUIDE.txt). 
