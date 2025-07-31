# Quick Start Guide

Welcome to neksis! This guide will get you up and running with neksis in just 5 minutes. You'll write your first program, learn the basics, and understand how to use the neksis development tools.

## 🚀 Prerequisites

Before you begin, make sure you have:

- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository
- **A text editor** - VS Code, Vim, or any editor you prefer

## 📦 Installation

### Option 1: From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/nexus-lang/nexus.git
cd nexus

# Build the compiler
cargo build --release

# Install globally (optional)
cargo install --path neksisc
```

### Option 2: Using Cargo

```bash
# Install directly from GitHub
cargo install --git https://github.com/nexus-lang/nexus.git neksisc
```

### Verify Installation

```bash
neksis --version
```

You should see: `neksis Programming Language Compiler v0.1.0`

## 🎯 Your First Program

Let's create your first neksis program! Create a file called `hello.nx`:

```nx
fn main() {
    println("Hello, neksis!");
}
```

### Running Your Program

```bash
neksis run hello.nx
```

**Output:**
```
🚀 Running hello.nx...
📤 Output:
Hello, neksis!
```

Congratulations! You've just written and run your first neksis program! 🎉

## 📚 Basic Concepts

### Functions

Functions are the building blocks of neksis programs. Every program starts with a `main` function:

```nx
fn main() {
    // Your code here
}
```

### Variables

Declare variables using `let`:

```nx
fn main() {
    let message = "Hello, World!";
    let number = 42;
    println(message);
}
```

### Types

neksis has a strong type system. You can specify types explicitly:

```nx
fn main() {
    let name: String = "neksis";
    let age: Int = 25;
    let height: Float = 1.75;
    let is_student: Bool = true;
    
    println("Name: " + name);
    println("Age: " + age);
}
```

### Functions with Parameters

```nx
fn greet(name: String) {
    println("Hello, " + name + "!");
}

fn add(a: Int, b: Int) -> Int {
    return a + b;
}

fn main() {
    greet("Alice");
    let result = add(5, 3);
    println("5 + 3 = " + result);
}
```

## 🛠️ Development Workflow

### 1. Create a New Project

```bash
neksis init my-project
cd my-project
```

This creates a project structure:
```
my-project/
├── nexus.json          # Project configuration
├── src/
│   └── main.nx        # Entry point
└── README.md
```

### 2. Write Your Code

Edit `src/main.nx`:

```nx
fn fibonacci(n: Int) -> Int {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    let result = fibonacci(10);
    println("Fibonacci(10) = " + result);
}
```

### 3. Run Your Program

```bash
neksis run
```

### 4. Format Your Code

```bash
neksis format src/main.nx
```

### 5. Lint Your Code

```bash
neksis lint src/main.nx
```

## 🔧 Development Tools

### Code Formatting

Nexus includes a built-in formatter for consistent code style:

```bash
neksis format src/main.nx
```

### Linting

Check your code for potential issues:

```bash
neksis lint src/main.nx
```

### Testing

Run the test suite:

```bash
neksis test
```

### Language Server

Start the LSP server for IDE integration:

```bash
neksis lsp
```

## 📖 Next Steps

Now that you have the basics, here's what to explore next:

1. **[Basic Syntax Tutorial](tutorials/basic-syntax.md)** - Learn more about neksis syntax
2. **[Functions and Control Flow](tutorials/functions-control-flow.md)** - Master functions and control structures
3. **[Standard Library](tutorials/standard-library.md)** - Explore built-in functionality
4. **[Package Management](tutorials/package-management.md)** - Learn to manage dependencies

## 🎯 Common Patterns

### Working with Strings

```nx
fn main() {
    let name = "neksis";
    let greeting = "Hello, " + name + "!";
    println(greeting);
}
```

### Basic Math

```nx
fn main() {
    let a = 10;
    let b = 5;
    let sum = a + b;
    let product = a * b;
    let quotient = a / b;
    
    println("Sum: " + sum);
    println("Product: " + product);
    println("Quotient: " + quotient);
}
```

### Conditional Logic

```nx
fn main() {
    let age = 18;
    
    if age >= 18 {
        println("You are an adult");
    } else {
        println("You are a minor");
    }
}
```

## 🐛 Troubleshooting

### Common Issues

**"Command not found: neksis"**
- Make sure you've built the compiler: `cargo build --release`
- Add the target directory to your PATH: `export PATH="$PATH:./target/release"`

**"File not found"**
- Check that your `.nx` file exists in the current directory
- Use the full path: `neksis run /path/to/your/file.nx`

**Compilation errors**
- Check the syntax in your `.nx` file
- Make sure all functions have proper return types
- Verify that all variables are declared before use

### Getting Help

- Run `neksis help` for command-line help
- Check the [Language Reference](reference/syntax.md) for syntax details
- Join our [Discord Community](https://discord.gg/nexus-lang) for support

## 🎉 Congratulations!

You've successfully:
- ✅ Installed neksis
- ✅ Written your first program
- ✅ Learned basic syntax
- ✅ Used development tools
- ✅ Understood the workflow

You're now ready to build amazing things with neksis! Continue your journey with the [tutorials](tutorials/basic-syntax.md) or explore the [language reference](reference/syntax.md).

---

**Happy coding with neksis! 🚀** 