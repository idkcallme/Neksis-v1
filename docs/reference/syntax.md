# neksis Language Reference Manual

This is the authoritative reference for the neksis programming language. It provides a complete description of the language's syntax, semantics, and core components.

## 📋 Table of Contents

1. [Lexical Structure](#lexical-structure)
2. [Program Structure](#program-structure)
3. [Declarations](#declarations)
4. [Expressions](#expressions)
5. [Statements](#statements)
6. [Types](#types)
7. [Functions](#functions)
8. [Control Flow](#control-flow)
9. [Error Handling](#error-handling)
10. [Memory Management](#memory-management)

## 🔤 Lexical Structure

### Identifiers

Identifiers are used for variable names, function names, and type names.

**Syntax:**
```
identifier ::= letter (letter | digit | '_')*
letter ::= 'a'..'z' | 'A'..'Z' | '_'
digit ::= '0'..'9'
```

**Examples:**
```nx
let myVariable = 42;
let _private = "secret";
let user_name = "john";
```

### Keywords

neksis reserves the following keywords:

```
fn        let       return    if        else      while     for
in        struct    enum      class     module    import    export
as        type      const     mut       ref       move      drop
try       catch     throw     break     continue  match     case
default   where     impl      trait     pub       priv      static
```

### Literals

#### Integer Literals

```nx
let decimal = 42;
let hex = 0x2A;
let binary = 0b101010;
let octal = 0o52;
```

#### Float Literals

```nx
let pi = 3.14159;
let scientific = 1.23e-4;
```

#### String Literals

```nx
let simple = "Hello, World!";
let multiline = "This is a
multi-line string";
let escaped = "Line 1\nLine 2\tTabbed";
```

#### Boolean Literals

```nx
let true_val = true;
let false_val = false;
```

## 📦 Program Structure

A neksis program consists of a sequence of declarations at the top level.

**Syntax:**
```
program ::= declaration*
declaration ::= function_declaration
              | struct_declaration
              | enum_declaration
              | module_declaration
              | import_declaration
```

**Example:**
```nx
// Import statements
import std.io;
import std.math;

// Type definitions
struct Point {
    x: Float,
    y: Float,
}

// Function definitions
fn main() -> Int {
    let point = Point { x: 1.0, y: 2.0 };
    println("Point: (" + point.x + ", " + point.y + ")");
    return 0;
}
```

## 📝 Declarations

### Function Declarations

**Syntax:**
```
function_declaration ::= 'fn' identifier '(' parameter_list? ')' return_type? function_body
parameter_list ::= parameter (',' parameter)*
parameter ::= identifier ':' type
return_type ::= '->' type
function_body ::= '{' statement* '}'
```

**Examples:**
```nx
// Simple function
fn greet(name: String) {
    println("Hello, " + name + "!");
}

// Function with return type
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

// Function with multiple parameters
fn calculate(x: Float, y: Float, operation: String) -> Float {
    if operation == "add" {
        return x + y;
    } else if operation == "multiply" {
        return x * y;
    }
    return 0.0;
}
```

### Variable Declarations

**Syntax:**
```
variable_declaration ::= 'let' identifier type_annotation? '=' expression ';'
type_annotation ::= ':' type
```

**Examples:**
```nx
// Type inference
let message = "Hello, World!";

// Explicit type annotation
let count: Int = 42;
let pi: Float = 3.14159;
let is_valid: Bool = true;

// Mutable variable
let mut counter = 0;
counter = counter + 1;
```

### Struct Declarations

**Syntax:**
```
struct_declaration ::= 'struct' identifier '{' field_list '}'
field_list ::= field (',' field)*
field ::= identifier ':' type
```

**Examples:**
```nx
struct Person {
    name: String,
    age: Int,
    email: String,
}

struct Point {
    x: Float,
    y: Float,
}

// Using structs
fn main() {
    let person = Person {
        name: "Alice",
        age: 30,
        email: "alice@example.com",
    };
    
    let point = Point { x: 1.0, y: 2.0 };
}
```

### Enum Declarations

**Syntax:**
```
enum_declaration ::= 'enum' identifier '{' variant_list '}'
variant_list ::= variant (',' variant)*
variant ::= identifier ('(' type_list ')')?
type_list ::= type (',' type)*
```

**Examples:**
```nx
enum Color {
    Red,
    Green,
    Blue,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

enum Option<T> {
    Some(T),
    None,
}
```

## 🔢 Expressions

### Primary Expressions

**Syntax:**
```
primary_expression ::= literal
                    | identifier
                    | '(' expression ')'
                    | function_call
                    | struct_construction
                    | field_access
```

**Examples:**
```nx
// Literals
42
"Hello"
true

// Identifiers
x
my_function

// Parenthesized expressions
(2 + 3) * 4

// Function calls
add(5, 3)
println("Hello")

// Struct construction
Point { x: 1.0, y: 2.0 }

// Field access
point.x
person.name
```

### Binary Expressions

**Syntax:**
```
binary_expression ::= primary_expression
                   | binary_expression operator binary_expression
operator ::= '+' | '-' | '*' | '/' | '%'
           | '==' | '!=' | '<' | '<=' | '>' | '>='
           | '&&' | '||'
```

**Examples:**
```nx
// Arithmetic
let sum = a + b;
let product = x * y;
let quotient = dividend / divisor;

// Comparison
let is_equal = a == b;
let is_greater = x > y;

// Logical
let both_true = condition1 && condition2;
let either_true = condition1 || condition2;
```

### Assignment Expressions

**Syntax:**
```
assignment_expression ::= identifier '=' expression
```

**Examples:**
```nx
let mut x = 10;
x = 20;
x = x + 5;
```

## 📜 Statements

### Expression Statements

**Syntax:**
```
expression_statement ::= expression ';'
```

**Examples:**
```nx
println("Hello, World!");
add(5, 3);
x = 42;
```

### Declaration Statements

**Syntax:**
```
declaration_statement ::= variable_declaration
```

**Examples:**
```nx
let x = 42;
let message: String = "Hello";
let mut counter = 0;
```

### Control Flow Statements

#### If Statements

**Syntax:**
```
if_statement ::= 'if' '(' expression ')' statement ('else' statement)?
```

**Examples:**
```nx
if x > 0 {
    println("Positive");
} else {
    println("Non-positive");
}

if age >= 18 {
    println("Adult");
} else if age >= 13 {
    println("Teenager");
} else {
    println("Child");
}
```

#### While Loops

**Syntax:**
```
while_statement ::= 'while' '(' expression ')' statement
```

**Examples:**
```nx
let mut i = 0;
while i < 10 {
    println("Count: " + i);
    i = i + 1;
}

while !queue.is_empty() {
    let item = queue.pop();
    process(item);
}
```

#### For Loops

**Syntax:**
```
for_statement ::= 'for' '(' identifier 'in' expression ')' statement
```

**Examples:**
```nx
for item in items {
    println(item);
}

for i in range(0, 10) {
    println("Index: " + i);
}
```

### Return Statements

**Syntax:**
```
return_statement ::= 'return' expression? ';'
```

**Examples:**
```nx
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

fn greet(name: String) {
    println("Hello, " + name + "!");
    return; // Optional explicit return
}
```

## 🏷️ Types

### Basic Types

neksis provides the following basic types:

- **`Int`** - 64-bit signed integer
- **`Float`** - 64-bit floating-point number
- **`String`** - UTF-8 string
- **`Bool`** - Boolean value (`true` or `false`)
- **`Void`** - Unit type (no value)

### Type Annotations

**Syntax:**
```
type ::= basic_type
       | identifier
       | '[' type ']'  // Array type
       | '(' type_list ')' '->' type  // Function type
       | type '?'  // Optional type
```

**Examples:**
```nx
let x: Int = 42;
let name: String = "Alice";
let numbers: [Int] = [1, 2, 3, 4, 5];
let handler: (String) -> Void = fn(msg: String) { println(msg); };
let maybe_value: Int? = null;
```

### Type Inference

neksis supports type inference, allowing you to omit type annotations when the compiler can determine the type:

```nx
let x = 42;        // Inferred as Int
let name = "Alice"; // Inferred as String
let is_valid = true; // Inferred as Bool
```

## 🔧 Functions

### Function Definitions

**Syntax:**
```
function_definition ::= 'fn' identifier '(' parameter_list? ')' return_type? '{' statement* '}'
```

**Examples:**
```nx
// Simple function
fn greet() {
    println("Hello, World!");
}

// Function with parameters
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

// Function with multiple parameters
fn calculate(x: Float, y: Float, operation: String) -> Float {
    if operation == "add" {
        return x + y;
    } else if operation == "multiply" {
        return x * y;
    }
    return 0.0;
}
```

### Function Calls

**Syntax:**
```
function_call ::= identifier '(' argument_list? ')'
argument_list ::= expression (',' expression)*
```

**Examples:**
```nx
greet();
let result = add(5, 3);
let area = calculate(10.0, 5.0, "multiply");
```

### Higher-Order Functions

neksis supports higher-order functions:

```nx
fn apply(f: (Int) -> Int, x: Int) -> Int {
    return f(x);
}

fn square(x: Int) -> Int {
    return x * x;
}

fn main() {
    let result = apply(square, 5); // result = 25
}
```

## 🔄 Control Flow

### Conditional Expressions

**Syntax:**
```
conditional_expression ::= 'if' expression 'then' expression 'else' expression
```

**Examples:**
```nx
let max = if a > b then a else b;
let message = if age >= 18 then "Adult" else "Minor";
```

### Pattern Matching

**Syntax:**
```
match_statement ::= 'match' expression '{' match_arm* '}'
match_arm ::= pattern '=>' expression ';'
pattern ::= literal | identifier | '_'
```

**Examples:**
```nx
fn describe_number(x: Int) -> String {
    match x {
        0 => "Zero",
        1 => "One",
        2 => "Two",
        _ => "Other",
    }
}

enum Option<T> {
    Some(T),
    None,
}

fn unwrap_or_default<T>(option: Option<T>, default: T) -> T {
    match option {
        Some(value) => value,
        None => default,
    }
}
```

## ⚠️ Error Handling

### Try-Catch Blocks

**Syntax:**
```
try_catch_statement ::= 'try' '{' statement* '}' 'catch' '(' identifier ')' '{' statement* '}'
```

**Examples:**
```nx
try {
    let result = divide(a, b);
    println("Result: " + result);
} catch (error) {
    println("Error: " + error);
}
```

### Error Propagation

```nx
fn read_file(path: String) -> Result<String, String> {
    // Implementation that may fail
    if file_exists(path) {
        return Ok(read_contents(path));
    } else {
        return Err("File not found");
    }
}

fn process_file(path: String) -> Result<String, String> {
    let content = read_file(path)?; // Propagate errors
    return process_content(content);
}
```

## 💾 Memory Management

### Ownership

neksis uses an ownership system similar to Rust:

```nx
fn main() {
    let s1 = String::new("Hello");
    let s2 = s1; // s1 is moved to s2, s1 is no longer valid
    
    // This would cause an error:
    // println(s1); // Error: s1 has been moved
    
    println(s2); // This works
}
```

### Borrowing

```nx
fn print_string(s: &String) {
    println(s);
}

fn main() {
    let s = String::new("Hello");
    print_string(&s); // Borrow s
    println(s); // s is still valid
}
```

### Lifetimes

```nx
fn longest<'a>(x: &'a String, y: &'a String) -> &'a String {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

## 📚 Standard Library

neksis provides a rich standard library with modules for common tasks:

```nx
import std.io;
import std.math;
import std.string;
import std.collections;

fn main() {
    // I/O operations
    let input = std.io.read_line();
    std.io.println("You entered: " + input);
    
    // Mathematical operations
    let sqrt_result = std.math.sqrt(16.0);
    let max_value = std.math.max(5, 10);
    
    // String operations
    let upper = std.string.to_upper("hello");
    let length = std.string.length("neksis");
    
    // Collections
    let mut list = std.collections.List::new();
    list.push(1);
    list.push(2);
    list.push(3);
}
```

## 🔍 Type System

### Type Checking

neksis performs static type checking at compile time:

```nx
fn add(a: Int, b: Int) -> Int {
    return a + b;
}

fn main() {
    let result = add(5, 3); // OK
    // let result = add("5", 3); // Error: type mismatch
}
```

### Type Coercion

neksis supports limited type coercion:

```nx
fn main() {
    let x: Int = 42;
    let y: Float = x; // Int to Float coercion
    
    let a: Float = 3.14;
    // let b: Int = a; // Error: Float to Int requires explicit conversion
}
```

### Generic Types

```nx
struct Container<T> {
    value: T,
}

fn create_container<T>(value: T) -> Container<T> {
    return Container { value: value };
}

fn main() {
    let int_container = create_container(42);
    let string_container = create_container("Hello");
}
```

---

This reference manual provides the complete specification for the neksis programming language. For more detailed information about specific features, see the individual reference sections. 