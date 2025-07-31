use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::{Write, BufRead};
use std::sync::{Arc, Mutex};
use crate::error::CompilerError;
use crate::ast::{Expression, Program, Statement, Literal};

#[derive(Debug, Clone)]
pub struct WasmBridge {
    wasm_pack_path: String,
    python_bridge: crate::python_bridge::PythonBridge,
    wasm_cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl WasmBridge {
    pub fn new() -> Self {
        Self {
            wasm_pack_path: "wasm-pack".to_string(),
            python_bridge: crate::python_bridge::PythonBridge::new(),
            wasm_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_wasm_pack_path(mut self, path: String) -> Self {
        self.wasm_pack_path = path;
        self
    }

    pub fn check_wasm_pack_available(&self) -> Result<bool, CompilerError> {
        let output = Command::new(&self.wasm_pack_path)
            .arg("--version")
            .output()
            .map_err(|e| CompilerError::syntax_error(&format!("wasm-pack not found: {}", e)))?;

        Ok(output.status.success())
    }

    pub fn compile_to_wasm(&self, neksis_code: &str, output_dir: &str) -> Result<(), CompilerError> {
        println!("Compiling Neksis code to WebAssembly...");

        // First, convert Neksis to Rust
        let rust_code = self.convert_neksis_to_rust(neksis_code)?;

        // Create a temporary Rust project
        let project_name = "neksis_wasm";
        self.create_rust_project(project_name, &rust_code)?;

        // Compile to WASM using wasm-pack
        let output = Command::new(&self.wasm_pack_path)
            .arg("build")
            .arg("--target")
            .arg("web")
            .arg("--out-dir")
            .arg(output_dir)
            .current_dir(format!("./{}", project_name))
            .output()
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to compile to WASM: {}", e)))?;

        if output.status.success() {
            println!("Successfully compiled to WebAssembly!");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(CompilerError::syntax_error(&format!("WASM compilation failed: {}", error)))
        }
    }

    pub fn convert_neksis_to_rust(&self, neksis_code: &str) -> Result<String, CompilerError> {
        let mut rust_code = String::new();
        rust_code.push_str("use wasm_bindgen::prelude::*;\n\n");

        // Convert basic Neksis syntax to Rust
        let lines: Vec<&str> = neksis_code.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            if trimmed.starts_with("fn ") {
                // Convert function declaration
                let func_def = self.convert_function_to_rust(trimmed);
                rust_code.push_str(&format!("{}\n", func_def));
            } else if trimmed.starts_with("let ") {
                // Convert let declaration
                let var_def = self.convert_variable_to_rust(trimmed);
                rust_code.push_str(&format!("{}\n", var_def));
            } else if trimmed.starts_with("print(") {
                // Convert print statement
                let print_stmt = self.convert_print_to_rust(trimmed);
                rust_code.push_str(&format!("{}\n", print_stmt));
            }
        }

        Ok(rust_code)
    }

    fn convert_function_to_rust(&self, neksis_func: &str) -> String {
        // Convert "fn add(a: Int, b: Int) -> Int { a + b }" to Rust
        let mut rust_func = neksis_func.to_string();
        
        // Replace fn with pub fn
        rust_func = rust_func.replace("fn ", "pub fn ");
        
        // Replace type annotations
        rust_func = rust_func.replace(": Int", ": i32");
        rust_func = rust_func.replace(": Float", ": f64");
        rust_func = rust_func.replace(": Bool", ": bool");
        rust_func = rust_func.replace(": String", ": String");
        rust_func = rust_func.replace(" -> Int", " -> i32");
        rust_func = rust_func.replace(" -> Float", " -> f64");
        rust_func = rust_func.replace(" -> Bool", " -> bool");
        rust_func = rust_func.replace(" -> String", " -> String");
        
        // Add #[wasm_bindgen] attribute
        let func_name = rust_func.split('(').next().unwrap_or("").replace("pub fn ", "");
        format!("#[wasm_bindgen]\n{}", rust_func)
    }

    fn convert_variable_to_rust(&self, neksis_var: &str) -> String {
        // Convert "let x = 42;" to "let x = 42;"
        neksis_var.replace("let ", "let ")
    }

    fn convert_print_to_rust(&self, neksis_print: &str) -> String {
        // Convert print statement to web_sys::console::log_1
        let content = neksis_print.replace("print(", "").replace(");", "");
        format!("web_sys::console::log_1(&wasm_bindgen::JsValue::from_str({}));", content)
    }

    fn create_rust_project(&self, project_name: &str, rust_code: &str) -> Result<(), CompilerError> {
        // Create Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = {{ version = "0.3", features = ["console"] }}
"#,
            project_name
        );

        // Create directory structure
        std::fs::create_dir_all(format!("{}/src", project_name))
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to create project directory: {}", e)))?;

        // Write Cargo.toml
        std::fs::write(format!("{}/Cargo.toml", project_name), cargo_toml)
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to write Cargo.toml: {}", e)))?;

        // Write lib.rs
        std::fs::write(format!("{}/src/lib.rs", project_name), rust_code)
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to write lib.rs: {}", e)))?;

        Ok(())
    }

    pub fn create_python_wasm_bridge(&self, wasm_file: &str) -> Result<String, CompilerError> {
        // Create Python code that can load and use the WASM module
        let python_code = format!(
            r#"
import asyncio
import aiohttp
import json
from pathlib import Path

class NeksisWasmBridge:
    def __init__(self, wasm_file_path):
        self.wasm_file_path = wasm_file_path
        self.wasm_instance = None
        
    async def load_wasm(self):
        """Load the WASM module"""
        try:
            # In a real implementation, you would use a WASM runtime like wasmer
            # For now, we'll create a mock implementation
            print(f"Loading WASM module from {{self.wasm_file_path}}")
            self.wasm_instance = {{"loaded": True}}
            return True
        except Exception as e:
            print(f"Failed to load WASM module: {{e}}")
            return False
    
    async def call_wasm_function(self, function_name, *args):
        """Call a function in the WASM module"""
        if not self.wasm_instance:
            await self.load_wasm()
        
        if self.wasm_instance:
            # Mock function call - in reality, you'd call the actual WASM function
            print(f"Calling WASM function {{function_name}} with args {{args}}")
            return f"WASM result from {{function_name}}"
        else:
            raise Exception("WASM module not loaded")
    
    def create_python_wrapper(self, function_name):
        """Create a Python wrapper for a WASM function"""
        wrapper_code = f'''
import asyncio
from neksis_wasm_bridge import NeksisWasmBridge

async def {function_name}_wrapper(*args):
    bridge = NeksisWasmBridge("{wasm_file}")
    return await bridge.call_wasm_function("{function_name}", *args)

def {function_name}(*args):
    """Synchronous wrapper for {function_name}"""
    return asyncio.run({function_name}_wrapper(*args))
'''
        return wrapper_code

# Example usage
async def main():
    bridge = NeksisWasmBridge("{wasm_file}")
    await bridge.load_wasm()
    
    # Call a function
    result = await bridge.call_wasm_function("add", 1, 2)
    print(f"Result: {{result}}")

if __name__ == "__main__":
    asyncio.run(main())
"#,
            wasm_file
        );

        Ok(python_code)
    }

    pub fn generate_wasm_bindings(&self, neksis_file: &str) -> Result<String, CompilerError> {
        // Generate WASM bindings for a Neksis file
        let neksis_content = std::fs::read_to_string(neksis_file)
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to read file: {}", e)))?;

        let mut bindings = String::new();
        bindings.push_str("use wasm_bindgen::prelude::*;\n");
        bindings.push_str("use web_sys::console;\n\n");

        // Parse Neksis file to find functions
        let lines: Vec<&str> = neksis_content.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") {
                // Extract function name and convert to Rust
                let rust_func = self.convert_function_to_rust(trimmed);
                bindings.push_str(&format!("{}\n", rust_func));
            }
        }

        Ok(bindings)
    }

    pub fn create_web_interface(&self, wasm_file: &str) -> Result<String, CompilerError> {
        // Create a simple HTML interface for the WASM module
        let html_code = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Neksis WASM Interface</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .container {{ max-width: 800px; margin: 0 auto; }}
        .function-call {{ margin: 20px 0; padding: 15px; border: 1px solid #ccc; }}
        input, button {{ margin: 5px; padding: 5px; }}
        #output {{ margin-top: 20px; padding: 10px; background: #f0f0f0; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Neksis WASM Interface</h1>
        <p>This interface allows you to call functions from your compiled Neksis code.</p>
        
        <div class="function-call">
            <h3>Function Call</h3>
            <input type="text" id="functionName" placeholder="Function name" value="add">
            <input type="text" id="args" placeholder="Arguments (comma-separated)" value="1,2">
            <button onclick="callFunction()">Call Function</button>
        </div>
        
        <div id="output"></div>
    </div>

    <script type="module">
        import init, {{ add }} from './pkg/{}';
        
        let wasmModule;
        
        async function initWasm() {{
            try {{
                wasmModule = await init();
                console.log('WASM module loaded successfully');
            }} catch (e) {{
                console.error('Failed to load WASM module:', e);
                document.getElementById('output').innerHTML = 'Error: Failed to load WASM module';
            }}
        }}
        
        window.callFunction = function() {{
            const functionName = document.getElementById('functionName').value;
            const argsStr = document.getElementById('args').value;
            const args = argsStr.split(',').map(arg => parseInt(arg.trim()) || arg.trim());
            
            try {{
                let result;
                switch(functionName) {{
                    case 'add':
                        result = add(args[0], args[1]);
                        break;
                    default:
                        result = 'Function not found';
                }}
                
                document.getElementById('output').innerHTML = `Result: ${{result}}`;
            }} catch (e) {{
                document.getElementById('output').innerHTML = `Error: ${{e}}`;
            }}
        }};
        
        // Initialize WASM when page loads
        initWasm();
    </script>
</body>
</html>
"#,
            wasm_file.replace(".wasm", "")
        );

        Ok(html_code)
    }

    pub fn compile_with_python_bridge(&self, neksis_code: &str, output_dir: &str) -> Result<(), CompilerError> {
        // Compile Neksis to WASM and create Python bridge
        self.compile_to_wasm(neksis_code, output_dir)?;

        // Create Python bridge
        let python_bridge = self.create_python_wasm_bridge(&format!("{}/pkg/neksis_wasm_bg.wasm", output_dir))?;
        
        // Write Python bridge to file
        std::fs::write(format!("{}/neksis_wasm_bridge.py", output_dir), python_bridge)
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to write Python bridge: {}", e)))?;

        // Create web interface
        let web_interface = self.create_web_interface("neksis_wasm")?;
        std::fs::write(format!("{}/index.html", output_dir), web_interface)
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to write web interface: {}", e)))?;

        println!("WASM compilation with Python bridge completed!");
        println!("Files created:");
        println!("  - {}/pkg/ (WASM files)", output_dir);
        println!("  - {}/neksis_wasm_bridge.py (Python bridge)", output_dir);
        println!("  - {}/index.html (Web interface)", output_dir);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_bridge_creation() {
        let bridge = WasmBridge::new();
        assert_eq!(bridge.wasm_pack_path, "wasm-pack");
    }

    #[test]
    fn test_neksis_to_rust_conversion() {
        let bridge = WasmBridge::new();
        let neksis_code = "fn add(a: Int, b: Int) -> Int { a + b }";
        let rust_code = bridge.convert_neksis_to_rust(neksis_code).unwrap();
        assert!(rust_code.contains("pub fn add"));
        assert!(rust_code.contains("i32"));
    }
} 