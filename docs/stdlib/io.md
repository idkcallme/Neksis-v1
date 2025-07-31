# Standard Library - I/O Operations

The neksis standard library provides comprehensive I/O (Input/Output) operations for file handling, console I/O, and data streams.

## 📋 Table of Contents

1. [Console I/O](#console-io)
2. [File Operations](#file-operations)
3. [Directory Operations](#directory-operations)
4. [Error Handling](#error-handling)
5. [Examples](#examples)

## 🖥️ Console I/O

### Basic Output

#### `println(value: String) -> Void`

Prints a string to standard output with a newline.

```nx
import std.io;

fn main() {
    std.io.println("Hello, World!");
    std.io.println("This is on a new line");
}
```

**Output:**
```
Hello, World!
This is on a new line
```

#### `print(value: String) -> Void`

Prints a string to standard output without a newline.

```nx
import std.io;

fn main() {
    std.io.print("Hello");
    std.io.print(" ");
    std.io.print("World");
    std.io.println("!");
}
```

**Output:**
```
Hello World!
```

#### `print_to_stdout(value: String) -> Void`

Explicitly prints to standard output.

```nx
import std.io;

fn main() {
    std.io.print_to_stdout("Standard output message");
}
```

#### `print_to_stderr(value: String) -> Void`

Prints a string to standard error.

```nx
import std.io;

fn main() {
    std.io.print_to_stderr("Error message");
}
```

### Basic Input

#### `read_from_stdin() -> String`

Reads a line from standard input.

```nx
import std.io;

fn main() {
    std.io.println("Enter your name:");
    let name = std.io.read_from_stdin();
    std.io.println("Hello, " + name + "!");
}
```

#### `read_line() -> String`

Reads a line from standard input (alias for `read_from_stdin`).

```nx
import std.io;

fn main() {
    std.io.println("Enter a number:");
    let input = std.io.read_line();
    let number = std.string.to_int(input);
    std.io.println("You entered: " + number);
}
```

## 📁 File Operations

### Reading Files

#### `read_file(path: String) -> Result<String, String>`

Reads the entire contents of a file as a string.

```nx
import std.io;

fn main() {
    let result = std.io.read_file("config.txt");
    match result {
        Ok(content) => {
            std.io.println("File contents:");
            std.io.println(content);
        },
        Err(error) => {
            std.io.print_to_stderr("Error reading file: " + error);
        }
    }
}
```

#### `read_file_bytes(path: String) -> Result<[Int], String>`

Reads a file as raw bytes.

```nx
import std.io;

fn main() {
    let result = std.io.read_file_bytes("image.png");
    match result {
        Ok(bytes) => {
            std.io.println("File size: " + std.collections.length(bytes) + " bytes");
        },
        Err(error) => {
            std.io.print_to_stderr("Error reading file: " + error);
        }
    }
}
```

### Writing Files

#### `write_file(path: String, content: String) -> Result<Void, String>`

Writes a string to a file, overwriting any existing content.

```nx
import std.io;

fn main() {
    let content = "Hello, this is a test file.\nThis is the second line.";
    let result = std.io.write_file("output.txt", content);
    
    match result {
        Ok(_) => {
            std.io.println("File written successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error writing file: " + error);
        }
    }
}
```

#### `write_file_bytes(path: String, bytes: [Int]) -> Result<Void, String>`

Writes raw bytes to a file.

```nx
import std.io;

fn main() {
    let bytes = [72, 101, 108, 108, 111]; // "Hello" in ASCII
    let result = std.io.write_file_bytes("binary.txt", bytes);
    
    match result {
        Ok(_) => {
            std.io.println("Binary file written successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error writing file: " + error);
        }
    }
}
```

#### `append_file(path: String, content: String) -> Result<Void, String>`

Appends a string to the end of a file.

```nx
import std.io;

fn main() {
    let new_line = "\nThis line was appended.";
    let result = std.io.append_file("log.txt", new_line);
    
    match result {
        Ok(_) => {
            std.io.println("Content appended successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error appending to file: " + error);
        }
    }
}
```

### File Management

#### `file_exists(path: String) -> Bool`

Checks if a file exists.

```nx
import std.io;

fn main() {
    let exists = std.io.file_exists("config.txt");
    if exists {
        std.io.println("File exists");
    } else {
        std.io.println("File does not exist");
    }
}
```

#### `delete_file(path: String) -> Result<Void, String>`

Deletes a file.

```nx
import std.io;

fn main() {
    let result = std.io.delete_file("temp.txt");
    match result {
        Ok(_) => {
            std.io.println("File deleted successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error deleting file: " + error);
        }
    }
}
```

#### `copy_file(source: String, destination: String) -> Result<Void, String>`

Copies a file from source to destination.

```nx
import std.io;

fn main() {
    let result = std.io.copy_file("original.txt", "backup.txt");
    match result {
        Ok(_) => {
            std.io.println("File copied successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error copying file: " + error);
        }
    }
}
```

#### `move_file(source: String, destination: String) -> Result<Void, String>`

Moves a file from source to destination.

```nx
import std.io;

fn main() {
    let result = std.io.move_file("old_name.txt", "new_name.txt");
    match result {
        Ok(_) => {
            std.io.println("File moved successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error moving file: " + error);
        }
    }
}
```

## 📂 Directory Operations

### Directory Management

#### `create_directory(path: String) -> Result<Void, String>`

Creates a new directory.

```nx
import std.io;

fn main() {
    let result = std.io.create_directory("new_folder");
    match result {
        Ok(_) => {
            std.io.println("Directory created successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error creating directory: " + error);
        }
    }
}
```

#### `create_directories(path: String) -> Result<Void, String>`

Creates a directory and all its parent directories.

```nx
import std.io;

fn main() {
    let result = std.io.create_directories("parent/child/grandchild");
    match result {
        Ok(_) => {
            std.io.println("Directory structure created successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error creating directories: " + error);
        }
    }
}
```

#### `list_directory(path: String) -> Result<[String], String>`

Lists the contents of a directory.

```nx
import std.io;

fn main() {
    let result = std.io.list_directory(".");
    match result {
        Ok(entries) => {
            std.io.println("Directory contents:");
            for entry in entries {
                std.io.println("  " + entry);
            }
        },
        Err(error) => {
            std.io.print_to_stderr("Error listing directory: " + error);
        }
    }
}
```

#### `delete_directory(path: String) -> Result<Void, String>`

Deletes an empty directory.

```nx
import std.io;

fn main() {
    let result = std.io.delete_directory("empty_folder");
    match result {
        Ok(_) => {
            std.io.println("Directory deleted successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error deleting directory: " + error);
        }
    }
}
```

#### `delete_directory_recursive(path: String) -> Result<Void, String>`

Deletes a directory and all its contents.

```nx
import std.io;

fn main() {
    let result = std.io.delete_directory_recursive("folder_with_contents");
    match result {
        Ok(_) => {
            std.io.println("Directory and contents deleted successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Error deleting directory: " + error);
        }
    }
}
```

### File Information

#### `get_file_size(path: String) -> Result<Int, String>`

Gets the size of a file in bytes.

```nx
import std.io;

fn main() {
    let result = std.io.get_file_size("large_file.txt");
    match result {
        Ok(size) => {
            std.io.println("File size: " + size + " bytes");
        },
        Err(error) => {
            std.io.print_to_stderr("Error getting file size: " + error);
        }
    }
}
```

#### `is_file(path: String) -> Bool`

Checks if a path refers to a file.

```nx
import std.io;

fn main() {
    let is_file = std.io.is_file("document.txt");
    if is_file {
        std.io.println("Path is a file");
    } else {
        std.io.println("Path is not a file");
    }
}
```

#### `is_directory(path: String) -> Bool`

Checks if a path refers to a directory.

```nx
import std.io;

fn main() {
    let is_dir = std.io.is_directory("folder");
    if is_dir {
        std.io.println("Path is a directory");
    } else {
        std.io.println("Path is not a directory");
    }
}
```

## ⚠️ Error Handling

The I/O module uses the `Result<T, E>` type for error handling. All functions that can fail return a `Result` type.

### Error Types

Common error messages include:

- `"File not found"` - The specified file doesn't exist
- `"Permission denied"` - Insufficient permissions to access the file
- `"Directory not empty"` - Attempting to delete a non-empty directory
- `"File already exists"` - Attempting to create a file that already exists
- `"Invalid path"` - The provided path is malformed

### Error Handling Patterns

#### Basic Error Handling

```nx
import std.io;

fn main() {
    let result = std.io.read_file("config.txt");
    match result {
        Ok(content) => {
            std.io.println("Success: " + content);
        },
        Err(error) => {
            std.io.print_to_stderr("Error: " + error);
        }
    }
}
```

#### Error Propagation

```nx
import std.io;

fn process_file(path: String) -> Result<String, String> {
    let content = std.io.read_file(path)?;
    return std.string.to_upper(content);
}

fn main() {
    let result = process_file("input.txt");
    match result {
        Ok(upper_content) => {
            std.io.println("Processed: " + upper_content);
        },
        Err(error) => {
            std.io.print_to_stderr("Processing failed: " + error);
        }
    }
}
```

## 📖 Examples

### Complete File Processing Example

```nx
import std.io;
import std.string;

fn process_log_file(path: String) -> Result<Int, String> {
    // Check if file exists
    if !std.io.file_exists(path) {
        return Err("Log file not found");
    }
    
    // Read file content
    let content = std.io.read_file(path)?;
    
    // Split into lines
    let lines = std.string.split(content, "\n");
    let mut error_count = 0;
    
    // Process each line
    for line in lines {
        if std.string.contains(line, "ERROR") {
            error_count = error_count + 1;
        }
    }
    
    // Write summary to output file
    let summary = "Total lines: " + std.collections.length(lines) + "\n";
    let summary = summary + "Error count: " + error_count + "\n";
    
    std.io.write_file("summary.txt", summary)?;
    
    return Ok(error_count);
}

fn main() {
    let result = process_log_file("application.log");
    match result {
        Ok(error_count) => {
            std.io.println("Processing complete. Found " + error_count + " errors.");
        },
        Err(error) => {
            std.io.print_to_stderr("Processing failed: " + error);
        }
    }
}
```

### Interactive File Editor

```nx
import std.io;

fn edit_file(path: String) -> Result<Void, String> {
    std.io.println("Simple file editor");
    std.io.println("Enter lines (empty line to finish):");
    
    let mut lines = [];
    let mut line_number = 1;
    
    while true {
        std.io.print("Line " + line_number + ": ");
        let input = std.io.read_line();
        
        if std.string.length(input) == 0 {
            break;
        }
        
        lines = std.collections.push(lines, input);
        line_number = line_number + 1;
    }
    
    // Join lines with newlines
    let content = std.string.join(lines, "\n");
    
    // Write to file
    std.io.write_file(path, content)?;
    
    std.io.println("File saved successfully");
    return Ok();
}

fn main() {
    let result = edit_file("notes.txt");
    match result {
        Ok(_) => {
            std.io.println("File editing completed");
        },
        Err(error) => {
            std.io.print_to_stderr("Error: " + error);
        }
    }
}
```

### Directory Backup Utility

```nx
import std.io;
import std.string;

fn backup_directory(source: String, backup_name: String) -> Result<Void, String> {
    // Create backup directory
    let backup_path = "backups/" + backup_name;
    std.io.create_directories(backup_path)?;
    
    // List source directory
    let entries = std.io.list_directory(source)?;
    
    for entry in entries {
        let source_path = source + "/" + entry;
        let dest_path = backup_path + "/" + entry;
        
        if std.io.is_file(source_path) {
            // Copy file
            std.io.copy_file(source_path, dest_path)?;
            std.io.println("Copied: " + entry);
        } else if std.io.is_directory(source_path) {
            // Recursively backup subdirectory
            backup_directory(source_path, backup_name + "/" + entry)?;
        }
    }
    
    return Ok();
}

fn main() {
    let result = backup_directory("documents", "backup_2024");
    match result {
        Ok(_) => {
            std.io.println("Backup completed successfully");
        },
        Err(error) => {
            std.io.print_to_stderr("Backup failed: " + error);
        }
    }
}
```

---

This reference covers all the I/O operations available in the neksis standard library. For more information about other standard library modules, see the [Standard Library Overview](../stdlib/README.md). 