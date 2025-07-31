use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::{Write, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use crate::error::CompilerError;
use crate::ast::{Expression, Program, Statement, Literal};

#[derive(Debug, Clone)]
pub struct PythonBridge {
    python_path: String,
    installed_packages: Arc<Mutex<HashMap<String, bool>>>,
    package_cache: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl PythonBridge {
    pub fn new() -> Self {
        Self {
            python_path: "python".to_string(),
            installed_packages: Arc::new(Mutex::new(HashMap::new())),
            package_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_python_path(mut self, path: String) -> Self {
        self.python_path = path;
        self
    }

    pub fn check_python_available(&self) -> Result<bool, CompilerError> {
        let output = Command::new(&self.python_path)
            .arg("--version")
            .output()
            .map_err(|e| CompilerError::syntax_error(&format!("Python not found: {}", e)))?;

        Ok(output.status.success())
    }

    pub fn install_package(&self, package_name: &str) -> Result<(), CompilerError> {
        println!("Installing Python package: {}", package_name);
        
        let output = Command::new(&self.python_path)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg(package_name)
            .output()
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to install package: {}", e)))?;

        if output.status.success() {
            let mut packages = self.installed_packages.lock().unwrap();
            packages.insert(package_name.to_string(), true);
            println!("Successfully installed: {}", package_name);
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(CompilerError::syntax_error(&format!("Failed to install {}: {}", package_name, error)))
        }
    }

    pub fn is_package_installed(&self, package_name: &str) -> bool {
        let packages = self.installed_packages.lock().unwrap();
        packages.get(package_name).copied().unwrap_or(false)
    }

    pub fn list_installed_packages(&self) -> Result<Vec<String>, CompilerError> {
        let output = Command::new(&self.python_path)
            .arg("-m")
            .arg("pip")
            .arg("list")
            .output()
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to list packages: {}", e)))?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let packages: Vec<String> = output_str
                .lines()
                .skip(2) // Skip header
                .filter_map(|line| {
                    line.split_whitespace().next().map(|s| s.to_string())
                })
                .collect();
            Ok(packages)
        } else {
            Err(CompilerError::syntax_error("Failed to list installed packages"))
        }
    }

    pub fn execute_python_code(&self, code: &str) -> Result<String, CompilerError> {
        let mut child = Command::new(&self.python_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to start Python: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(code.as_bytes())
                .map_err(|e| CompilerError::syntax_error(&format!("Failed to write to Python stdin: {}", e)))?;
        }

        let output = child.wait_with_output()
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to get Python output: {}", e)))?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout);
            Ok(result.trim().to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(CompilerError::syntax_error(&format!("Python execution failed: {}", error)))
        }
    }

    pub fn call_python_function(&self, module: &str, function: &str, args: &[String]) -> Result<String, CompilerError> {
        let args_str = args.join(", ");
        let code = format!(
            "import {}\nresult = {}.{}({})\nprint(result)",
            module, module, function, args_str
        );
        
        self.execute_python_code(&code)
    }

    pub fn get_package_functions(&self, package_name: &str) -> Result<Vec<String>, CompilerError> {
        let cache = self.package_cache.lock().unwrap();
        if let Some(functions) = cache.get(package_name) {
            return Ok(functions.clone());
        }
        drop(cache);

        let code = format!(
            "import {}\nimport inspect\nfunctions = [name for name, obj in inspect.getmembers({}) if inspect.isfunction(obj)]\nprint('\\n'.join(functions))",
            package_name, package_name
        );

        let result = self.execute_python_code(&code)?;
        let functions: Vec<String> = result.lines().map(|s| s.to_string()).collect();

        let mut cache = self.package_cache.lock().unwrap();
        cache.insert(package_name.to_string(), functions.clone());

        Ok(functions)
    }

    pub fn convert_neksis_to_python(&self, neksis_code: &str) -> Result<String, CompilerError> {
        // Simple conversion from Neksis to Python
        // This is a basic implementation - in practice, you'd want a more sophisticated converter
        
        let mut python_code = String::new();
        python_code.push_str("# Generated Python code from Neksis\n");
        python_code.push_str("import sys\n");
        python_code.push_str("import os\n\n");

        // Convert basic Neksis syntax to Python
        let lines: Vec<&str> = neksis_code.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            if trimmed.starts_with("let ") {
                // Convert let declaration to Python assignment
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let var_name = parts[0].replace("let ", "").trim().to_string();
                    let value = parts[1].trim().replace(';', "");
                    python_code.push_str(&format!("{} = {}\n", var_name, value));
                }
            } else if trimmed.starts_with("fn ") {
                // Convert function declaration
                let func_def = trimmed.replace("fn ", "def ").replace(" -> ", " -> ");
                python_code.push_str(&format!("{}\n", func_def));
            } else if trimmed.starts_with("print(") {
                // Convert print statement
                python_code.push_str(&format!("{}\n", trimmed));
            } else if trimmed.contains('=') {
                // Convert assignment
                python_code.push_str(&format!("{}\n", trimmed.replace(';', "")));
            }
        }

        Ok(python_code)
    }

    pub fn convert_python_to_neksis(&self, python_code: &str) -> Result<String, CompilerError> {
        // Simple conversion from Python to Neksis
        let mut neksis_code = String::new();
        neksis_code.push_str("// Generated Neksis code from Python\n\n");

        let lines: Vec<&str> = python_code.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("#") {
                continue;
            }

            if trimmed.starts_with("def ") {
                // Convert function definition
                let func_def = trimmed.replace("def ", "fn ").replace(" -> ", " -> ");
                neksis_code.push_str(&format!("{}\n", func_def));
            } else if trimmed.starts_with("print(") {
                // Convert print statement
                neksis_code.push_str(&format!("{};\n", trimmed));
            } else if trimmed.contains('=') && !trimmed.contains("==") {
                // Convert assignment
                neksis_code.push_str(&format!("let {};\n", trimmed));
            } else {
                // Keep other lines as-is
                neksis_code.push_str(&format!("{}\n", trimmed));
            }
        }

        Ok(neksis_code)
    }

    pub fn create_python_wrapper(&self, neksis_function: &str) -> Result<String, CompilerError> {
        // Create a Python wrapper for a Neksis function
        let wrapper_code = format!(
            r#"
import subprocess
import sys
import json

def call_neksis_function(function_name, *args):
    # Create a temporary Neksis file
    neksis_code = f'''
{neksis_function}
let result = {function_name}({", ".join([str(arg) for arg in args])});
print(result);
'''
    
    # Write to temporary file
    with open('temp_neksis.nx', 'w') as f:
        f.write(neksis_code)
    
    # Execute Neksis code
    try:
        result = subprocess.run(['neksisc', 'temp_neksis.nx'], 
                              capture_output=True, text=True, check=True)
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        return f"Error: {{e.stderr}}"
    finally:
        # Clean up
        import os
        if os.path.exists('temp_neksis.nx'):
            os.remove('temp_neksis.nx')

if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1:
        function_name = sys.argv[1]
        args = sys.argv[2:]
        result = call_neksis_function(function_name, *args)
        print(result)
"#
        );

        Ok(wrapper_code)
    }

    pub fn generate_python_bindings(&self, neksis_file: &str) -> Result<String, CompilerError> {
        // Generate Python bindings for a Neksis file
        let neksis_content = std::fs::read_to_string(neksis_file)
            .map_err(|e| CompilerError::syntax_error(&format!("Failed to read file: {}", e)))?;

        let mut bindings = String::new();
        bindings.push_str("# Python bindings for Neksis functions\n");
        bindings.push_str("import subprocess\n");
        bindings.push_str("import json\n\n");

        // Parse Neksis file to find functions
        let lines: Vec<&str> = neksis_content.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") {
                // Extract function name
                let func_parts: Vec<&str> = trimmed.split('(').collect();
                if let Some(func_name) = func_parts.get(0) {
                    let name = func_name.replace("fn ", "").trim().to_string();
                    let params = func_parts.get(1).map(|s| s.trim()).unwrap_or("");
                    let body = trimmed.split('{').nth(1).map(|s| s.trim()).unwrap_or("");
                    
                    bindings.push_str(&format!(
                        r#"
def {}({}):\n    """Call Neksis function {}"""
    neksis_code = f'''
fn {}({}):\n    # Function implementation would go here\n    pass

let result = {}({{", ".join([str(arg) for arg in args])}}});
print(result);
'''
    
    with open('temp_call.nx', 'w') as f:
        f.write(neksis_code)
    
    try:
        result = subprocess.run(['neksisc', 'temp_call.nx'], 
                              capture_output=True, text=True, check=True)
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        return f"Error: {{e.stderr}}"
    finally:
        import os
        if os.path.exists('temp_call.nx'):
            os.remove('temp_call.nx')

"#,
                        name, params, name, name, params, name, name, name, name
                    ));
                }
            }
        }

        Ok(bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_bridge_creation() {
        let bridge = PythonBridge::new();
        assert_eq!(bridge.python_path, "python");
    }

    #[test]
    fn test_neksis_to_python_conversion() {
        let bridge = PythonBridge::new();
        let neksis_code = "let x = 42;\nfn add(a, b) -> Int { a + b }\nprint(x);";
        let python_code = bridge.convert_neksis_to_python(neksis_code).unwrap();
        assert!(python_code.contains("x = 42"));
        assert!(python_code.contains("def add"));
    }
} 