// Module System for Neksis 2025
//
// This module provides import/export functionality, namespace management,
// and module resolution for the Neksis programming language.

use crate::modern_ast::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Module registry for managing loaded modules
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    pub modules: HashMap<String, Module>,
    pub module_paths: HashMap<String, PathBuf>,
    pub current_module: Option<String>,
    pub global_dependencies: Vec<String>,
}

/// Represents a loaded module
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub exports: HashMap<String, ExportedItem>,
    pub imports: HashMap<String, ImportedItem>,
    pub statements: Vec<Statement>,
    pub dependencies: Vec<String>,
}

/// Items that can be exported from a module
#[derive(Debug, Clone)]
pub enum ExportedItem {
    Function {
        name: String,
        params: Vec<Parameter>,
        body: Vec<Statement>,
        return_type: Option<Type>,
    },
    Class {
        name: String,
        definition: Statement,
    },
    Variable {
        name: String,
        value: Expression,
        var_type: Option<Type>,
    },
    Constant {
        name: String,
        value: Expression,
        const_type: Option<Type>,
    },
    Type {
        name: String,
        definition: Type,
    },
}

/// Items imported from other modules
#[derive(Debug, Clone)]
pub struct ImportedItem {
    pub name: String,
    pub original_name: String,
    pub module_name: String,
    pub item_type: ImportType,
}

#[derive(Debug, Clone)]
pub enum ImportType {
    Function,
    Class,
    Variable,
    Constant,
    Type,
    Module, // Wildcard import
}

/// Import resolution strategies
#[derive(Debug, Clone)]
pub enum ImportResolution {
    Relative(String),      // ./module or ../module
    Absolute(String),      // /path/to/module
    Package(String),       // package_name
    Standard(String),      // std::io
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            module_paths: HashMap::new(),
            current_module: None,
            global_dependencies: Vec::new(),
        }
    }

    /// Load a module from file
    pub fn load_module(&mut self, module_name: &str, file_path: &Path) -> Result<(), String> {
        if self.modules.contains_key(module_name) {
            return Ok(()); // Already loaded
        }

        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read module {}: {}", module_name, e))?;

        // Parse the module content
        let mut lexer = crate::modern_lexer::Lexer::new(&content);
        let tokens = lexer.tokenize();
        let tokens = if tokens.is_empty() {
            return Err(format!("Failed to tokenize module {}", module_name));
        } else {
            tokens
        };

        let mut parser = crate::modern_parser::Parser::new(tokens);
        let program = parser.parse()
            .map_err(|e| format!("Failed to parse module {}: {:?}", module_name, e))?;

        // Extract exports and imports
        let exports = HashMap::new();
        let imports = HashMap::new();
        let mut dependencies = Vec::new();
        let mut module_statements = Vec::new();

        for stmt in program.statements {
            match &stmt {
                Statement::Use(use_stmt) => {
                    // Handle use statements for standard library imports
                    dependencies.push(use_stmt.path.clone());
                },
                _ => {
                    module_statements.push(stmt);
                }
            }
        }

        let module = Module {
            name: module_name.to_string(),
            path: file_path.to_path_buf(),
            exports,
            imports,
            statements: module_statements,
            dependencies,
        };

        self.modules.insert(module_name.to_string(), module);
        self.module_paths.insert(module_name.to_string(), file_path.to_path_buf());

        Ok(())
    }

    /// Extract an exported item from a statement
    #[allow(dead_code)]
    fn extract_export(&self, stmt: &Statement) -> Option<ExportedItem> {
        match stmt {
            Statement::Function(func_stmt) => {
                Some(ExportedItem::Function {
                    name: func_stmt.name.clone(),
                    params: func_stmt.parameters.clone(),
                    body: vec![], // TODO: Convert expression body to statements
                    return_type: func_stmt.return_type.clone(),
                })
            },
            Statement::Class(class_stmt) => {
                Some(ExportedItem::Class {
                    name: class_stmt.name.clone(),
                    definition: stmt.clone(),
                })
            },
            Statement::Let(let_stmt) => {
                Some(ExportedItem::Variable {
                    name: let_stmt.name.clone(),
                    value: *let_stmt.value.clone(),
                    var_type: let_stmt.type_annotation.clone(),
                })
            },
            _ => None,
        }
    }

    /// Resolve an import path to a file path
    pub fn resolve_import(&self, import_path: &str, current_module_path: &Path) -> Result<PathBuf, String> {
        let resolution = self.parse_import_path(import_path);
        
        match resolution {
            ImportResolution::Relative(path) => {
                let current_dir = current_module_path.parent()
                    .ok_or("Invalid current module path")?;
                Ok(current_dir.join(format!("{}.nx", path)))
            },
            ImportResolution::Absolute(path) => {
                Ok(PathBuf::from(format!("{}.nx", path)))
            },
            ImportResolution::Package(package) => {
                // Look in package directories
                self.resolve_package(&package)
            },
            ImportResolution::Standard(std_module) => {
                // Look in standard library
                self.resolve_standard_module(&std_module)
            }
        }
    }

    /// Parse import path to determine resolution strategy
    pub fn parse_import_path(&self, path: &str) -> ImportResolution {
        if path.starts_with("./") || path.starts_with("../") {
            ImportResolution::Relative(path.to_string())
        } else if path.starts_with("/") {
            ImportResolution::Absolute(path.to_string())
        } else if path.starts_with("std::") {
            ImportResolution::Standard(path.to_string())
        } else {
            ImportResolution::Package(path.to_string())
        }
    }

    /// Resolve a package import
    fn resolve_package(&self, package: &str) -> Result<PathBuf, String> {
        // Look in common package directories
        let package_dirs = vec![
            "./packages",
            "./modules",
            "./lib",
            "../packages",
        ];

        for dir in package_dirs {
            let package_path = PathBuf::from(dir).join(format!("{}.nx", package));
            if package_path.exists() {
                return Ok(package_path);
            }
        }

        Err(format!("Package {} not found", package))
    }

    /// Resolve a standard library module
    fn resolve_standard_module(&self, std_module: &str) -> Result<PathBuf, String> {
        // Remove std:: prefix
        let module_name = std_module.strip_prefix("std::").unwrap_or(std_module);
        
        // Look in standard library directory
        let std_dirs = vec![
            "./stdlib",
            "../stdlib",
            "./neksisc/src/stdlib",
        ];

        for dir in std_dirs {
            let std_path = PathBuf::from(dir).join(format!("{}.nx", module_name));
            if std_path.exists() {
                return Ok(std_path);
            }
        }

        Err(format!("Standard module {} not found", std_module))
    }

    /// Get an exported item from a module
    pub fn get_export(&self, module_name: &str, item_name: &str) -> Option<&ExportedItem> {
        self.modules.get(module_name)?.exports.get(item_name)
    }

    /// Check if a module is loaded
    pub fn is_loaded(&self, module_name: &str) -> bool {
        self.modules.contains_key(module_name)
    }

    /// Get all dependencies for a module
    pub fn get_dependencies(&self, module_name: &str) -> Vec<String> {
        self.modules.get(module_name)
            .map(|m| m.dependencies.clone())
            .unwrap_or_default()
    }

    /// Load all dependencies for a module recursively
    pub fn load_dependencies(&mut self, module_name: &str) -> Result<(), String> {
        let dependencies = self.get_dependencies(module_name);
        
        for dep in dependencies {
            if !self.is_loaded(&dep) {
                // Try to resolve and load the dependency
                if let Some(current_path) = self.module_paths.get(module_name) {
                    let dep_path = self.resolve_import(&dep, current_path)?;
                    self.load_module(&dep, &dep_path)?;
                    
                    // Recursively load dependencies of the dependency
                    self.load_dependencies(&dep)?;
                }
            }
        }

        Ok(())
    }

    /// Create a module namespace for execution
    pub fn create_namespace(&self, module_name: &str) -> HashMap<String, ExportedItem> {
        let mut namespace = HashMap::new();

        if let Some(module) = self.modules.get(module_name) {
            // Add module's own exports
            for (name, item) in &module.exports {
                namespace.insert(name.clone(), item.clone());
            }

            // Add imported items
            for (alias, import) in &module.imports {
                if let Some(source_module) = self.modules.get(&import.module_name) {
                    if let Some(item) = source_module.exports.get(&import.original_name) {
                        namespace.insert(alias.clone(), item.clone());
                    }
                }
            }
        }

        namespace
    }
}

impl ExportedItem {
    pub fn name(&self) -> &str {
        match self {
            ExportedItem::Function { name, .. } => name,
            ExportedItem::Class { name, .. } => name,
            ExportedItem::Variable { name, .. } => name,
            ExportedItem::Constant { name, .. } => name,
            ExportedItem::Type { name, .. } => name,
        }
    }

    pub fn item_type(&self) -> ImportType {
        match self {
            ExportedItem::Function { .. } => ImportType::Function,
            ExportedItem::Class { .. } => ImportType::Class,
            ExportedItem::Variable { .. } => ImportType::Variable,
            ExportedItem::Constant { .. } => ImportType::Constant,
            ExportedItem::Type { .. } => ImportType::Type,
        }
    }
}

/// Module system executor
pub struct ModuleExecutor {
    pub registry: ModuleRegistry,
    pub current_namespace: HashMap<String, ExportedItem>,
}

impl ModuleExecutor {
    pub fn new() -> Self {
        Self {
            registry: ModuleRegistry::new(),
            current_namespace: HashMap::new(),
        }
    }

    /// Execute an import statement (placeholder for future implementation)
    pub fn execute_import(&mut self, _import_stmt: &Statement) -> Result<(), String> {
        // TODO: Implement proper import handling when AST supports it
        Ok(())
    }

    /// Execute a use statement (for standard library)
    pub fn execute_use(&mut self, use_stmt: &Statement) -> Result<(), String> {
        if let Statement::Use(use_stmt) = use_stmt {
            // Add to dependencies for now
            if !self.registry.global_dependencies.contains(&use_stmt.path) {
                self.registry.global_dependencies.push(use_stmt.path.clone());
            }
            Ok(())
        } else {
            Err("Expected use statement".to_string())
        }
    }

    /// Look up an identifier in the current namespace
    pub fn lookup(&self, name: &str) -> Option<&ExportedItem> {
        self.current_namespace.get(name)
    }

    /// Set the current module context
    pub fn set_current_module(&mut self, module_name: String) {
        self.registry.current_module = Some(module_name.clone());
        self.current_namespace = self.registry.create_namespace(&module_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registry() {
        let mut registry = ModuleRegistry::new();
        assert!(registry.modules.is_empty());
        assert!(!registry.is_loaded("test"));
    }

    #[test]
    fn test_import_path_parsing() {
        let registry = ModuleRegistry::new();
        
        match registry.parse_import_path("./module") {
            ImportResolution::Relative(_) => {},
            _ => panic!("Expected relative import"),
        }
        
        match registry.parse_import_path("std::io") {
            ImportResolution::Standard(_) => {},
            _ => panic!("Expected standard import"),
        }
        
        match registry.parse_import_path("my_package") {
            ImportResolution::Package(_) => {},
            _ => panic!("Expected package import"),
        }
    }

    #[test]
    fn test_exported_item_name() {
        let func_export = ExportedItem::Function {
            name: "test_func".to_string(),
            params: vec![],
            body: vec![],
            return_type: None,
        };
        
        assert_eq!(func_export.name(), "test_func");
        
        match func_export.item_type() {
            ImportType::Function => {},
            _ => panic!("Expected function type"),
        }
    }
}
