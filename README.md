# Neksis Programming Language

A modern compiled programming language with clean syntax and type safety.

## Features

- **Clear Syntax** - Easy to read and write
- **Compiled Performance** - Runs as native machine code  
- **Type Safety** - Catches errors at compile time
- **Memory Efficiency** - Automatic memory management
- **Cross-platform** - Works on Windows, macOS, and Linux

## Installation

### Prerequisites
- Rust 1.70+ (for building from source)
- Git
- A modern operating system (Windows, macOS, Linux)

### Build from Source
```bash
git clone https://github.com/idkcallme/Neksis-v1.git
cd Neksis
cargo build --release
```

## Quick Start

1. **Build the compiler:**
   ```bash
   cargo build --release
   ```

2. **Run a program:**
   ```bash
   cargo run --bin neksis -- examples/hello_world.nx
   ```

3. **Learn the language:**
   Read the [Complete Neksis Tutorial](COMPLETE_NEKSIS_TUTORIAL.md) for comprehensive documentation with real examples.

## What Works

✅ **Core Language Features:**
- Variables (`Int`, `String` types)
- Arithmetic operations (`+`, `-`, `*`, `/`)
- Comparison operators (`>`, `<`, `>=`, `<=`)
- Control flow (`if`/`else`, `while` loops)
- Functions (parameters, return values)
- String concatenation

✅ **Real Examples:**
- Calculator programs
- Temperature converters  
- Grade calculators
- Loan calculators
- Text processing

## Current Limitations

- No arrays/lists yet
- No for loops (while loops only)
- No complex data structures
- Single file programs only
- No file I/O yet

## Repository Structure

- `src/` - Compiler source code
- `examples/` - Sample Neksis programs
- `tests/` - Test suite
- `COMPLETE_NEKSIS_TUTORIAL.md` - Comprehensive tutorial with verified examples

## Contributing

This is an active project. All code examples in the documentation have been tested and verified to work correctly.

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

See [LICENSE](LICENSE) file for details.
