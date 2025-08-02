// Object-Oriented Programming Support for Neksis 2025
//
// This module provides class instances, method calls, inheritance,
// polymorphism, and encapsulation features for the Neksis language.

use crate::modern_ast::*;
use std::collections::HashMap;

/// Represents a class instance at runtime
#[derive(Debug, Clone, PartialEq)]
pub struct ClassInstance {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
    pub methods: HashMap<String, Vec<Statement>>,
    pub parent_class: Option<String>,
}

/// Runtime value types for Neksis OOP
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(ClassInstance),
    Function(Vec<String>, Vec<Statement>), // params, body
    Null,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Boolean(_) => "bool",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function(_, _) => "function",
            Value::Null => "null",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(_) => true,
            Value::Function(_, _) => true,
        }
    }
}

/// Class definition registry
#[derive(Debug, Clone)]
pub struct ClassRegistry {
    pub classes: HashMap<String, ClassDefinition>,
}

/// Complete class definition with inheritance support
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDefinition {
    pub name: String,
    pub parent: Option<String>,
    pub fields: Vec<FieldDefinition>,
    pub methods: Vec<MethodDefinition>,
    pub constructors: Vec<MethodDefinition>,
    pub visibility: HashMap<String, Visibility>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: Option<Type>,
    pub default_value: Option<Expression>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDefinition {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_override: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}

impl ClassRegistry {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    /// Register a new class definition
    pub fn register_class(&mut self, class_def: ClassDefinition) -> Result<(), String> {
        // Check for inheritance cycles
        if let Some(parent) = &class_def.parent {
            if self.has_inheritance_cycle(&class_def.name, parent) {
                return Err(format!("Inheritance cycle detected for class {}", class_def.name));
            }
        }

        self.classes.insert(class_def.name.clone(), class_def);
        Ok(())
    }

    /// Check if a class inherits from another (directly or indirectly)
    pub fn inherits_from(&self, child: &str, parent: &str) -> bool {
        if let Some(class) = self.classes.get(child) {
            if let Some(class_parent) = &class.parent {
                if class_parent == parent {
                    return true;
                }
                return self.inherits_from(class_parent, parent);
            }
        }
        false
    }

    /// Detect inheritance cycles
    fn has_inheritance_cycle(&self, class_name: &str, parent_name: &str) -> bool {
        if class_name == parent_name {
            return true;
        }
        if let Some(parent_class) = self.classes.get(parent_name) {
            if let Some(grandparent) = &parent_class.parent {
                return self.has_inheritance_cycle(class_name, grandparent);
            }
        }
        false
    }

    /// Get all methods for a class (including inherited)
    pub fn get_all_methods(&self, class_name: &str) -> Vec<MethodDefinition> {
        let mut methods = Vec::new();
        let mut current_class = class_name;

        loop {
            if let Some(class) = self.classes.get(current_class) {
                // Add this class's methods
                for method in &class.methods {
                    // Check if method is already overridden
                    if !methods.iter().any(|m: &MethodDefinition| m.name == method.name) {
                        methods.push(method.clone());
                    }
                }

                // Move to parent class
                if let Some(parent) = &class.parent {
                    current_class = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        methods
    }

    /// Create a new instance of a class
    pub fn create_instance(&self, class_name: &str, _args: Vec<Value>) -> Result<ClassInstance, String> {
        let class_def = self.classes.get(class_name)
            .ok_or_else(|| format!("Class {} not found", class_name))?;

        let mut instance = ClassInstance {
            class_name: class_name.to_string(),
            fields: HashMap::new(),
            methods: HashMap::new(),
            parent_class: class_def.parent.clone(),
        };

        // Initialize fields with default values
        self.initialize_fields(&mut instance, class_name)?;

        // Call constructor if available
        if let Some(_constructor) = class_def.constructors.first() {
            // TODO: Call constructor with proper parameter matching
        }

        Ok(instance)
    }

    /// Initialize fields for an instance (including inherited fields)
    fn initialize_fields(&self, instance: &mut ClassInstance, class_name: &str) -> Result<(), String> {
        let mut current_class = class_name;

        loop {
            if let Some(class) = self.classes.get(current_class) {
                // Initialize this class's fields
                for field in &class.fields {
                    if !instance.fields.contains_key(&field.name) {
                        let default_value = if let Some(_default) = &field.default_value {
                            // TODO: Evaluate default expression
                            Value::Null
                        } else {
                            Value::Null
                        };
                        instance.fields.insert(field.name.clone(), default_value);
                    }
                }

                // Move to parent class
                if let Some(parent) = &class.parent {
                    current_class = parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

/// Object-oriented operations executor
pub struct OOPExecutor {
    pub class_registry: ClassRegistry,
    pub instances: HashMap<String, ClassInstance>,
    pub instance_counter: usize,
}

impl OOPExecutor {
    pub fn new() -> Self {
        Self {
            class_registry: ClassRegistry::new(),
            instances: HashMap::new(),
            instance_counter: 0,
        }
    }

    /// Execute a method call on an object
    pub fn call_method(
        &mut self,
        instance_id: &str,
        method_name: &str,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        let instance = self.instances.get(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?
            .clone();

        let methods = self.class_registry.get_all_methods(&instance.class_name);
        let method = methods.iter()
            .find(|m| m.name == method_name)
            .ok_or_else(|| format!("Method {} not found in class {}", method_name, instance.class_name))?;

        // Check parameter count
        if method.params.len() != args.len() {
            return Err(format!(
                "Method {} expects {} parameters, got {}",
                method_name,
                method.params.len(),
                args.len()
            ));
        }

        // TODO: Execute method body with proper scope and parameter binding
        Ok(Value::Null)
    }

    /// Access a field on an object
    pub fn get_field(&self, instance_id: &str, field_name: &str) -> Result<Value, String> {
        let instance = self.instances.get(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?;

        instance.fields.get(field_name)
            .cloned()
            .ok_or_else(|| format!("Field {} not found", field_name))
    }

    /// Set a field on an object
    pub fn set_field(&mut self, instance_id: &str, field_name: &str, value: Value) -> Result<(), String> {
        let instance = self.instances.get_mut(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?;

        // TODO: Check field visibility and type compatibility
        instance.fields.insert(field_name.to_string(), value);
        Ok(())
    }

    /// Create a new class instance
    pub fn new_instance(&mut self, class_name: &str, args: Vec<Value>) -> Result<String, String> {
        let instance = self.class_registry.create_instance(class_name, args)?;
        
        self.instance_counter += 1;
        let instance_id = format!("instance_{}", self.instance_counter);
        
        self.instances.insert(instance_id.clone(), instance);
        Ok(instance_id)
    }

    /// Check if an instance is of a specific type (including inheritance)
    pub fn instance_of(&self, instance_id: &str, class_name: &str) -> Result<bool, String> {
        let instance = self.instances.get(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?;

        if instance.class_name == class_name {
            return Ok(true);
        }

        Ok(self.class_registry.inherits_from(&instance.class_name, class_name))
    }
}

/// Convert AST class definition to runtime class definition
pub fn convert_ast_class(ast_class: &Statement) -> Result<ClassDefinition, String> {
    if let Statement::Class(class_stmt) = ast_class {
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut constructors = Vec::new();

        // Convert class fields
        for field in &class_stmt.fields {
            fields.push(FieldDefinition {
                name: field.name.clone(),
                field_type: Some(field.field_type.clone()),
                default_value: None, // TODO: Support default values in AST
                visibility: if field.is_public { Visibility::Public } else { Visibility::Private },
            });
        }

        // Convert class methods
        for method in &class_stmt.methods {
            let method_def = MethodDefinition {
                name: method.name.clone(),
                params: method.parameters.iter().map(|p| Parameter {
                    name: p.name.clone(),
                    type_annotation: p.type_annotation.clone(),
                    default_value: p.default_value.clone(),
                }).collect(),
                return_type: method.return_type.clone(),
                body: vec![], // TODO: Convert expression body to statements
                visibility: Visibility::Public, // Default visibility for now
                is_static: false,
                is_virtual: false,
                is_override: false,
            };

            if method.name == "constructor" || method.name == class_stmt.name {
                constructors.push(method_def);
            } else {
                methods.push(method_def);
            }
        }

        Ok(ClassDefinition {
            name: class_stmt.name.clone(),
            parent: class_stmt.superclass.clone(),
            fields,
            methods,
            constructors,
            visibility: HashMap::new(),
        })
    } else {
        Err("Expected class statement".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_registry() {
        let mut registry = ClassRegistry::new();
        
        let class_def = ClassDefinition {
            name: "TestClass".to_string(),
            parent: None,
            fields: vec![],
            methods: vec![],
            constructors: vec![],
            visibility: HashMap::new(),
        };
        
        assert!(registry.register_class(class_def).is_ok());
        assert!(registry.classes.contains_key("TestClass"));
    }

    #[test]
    fn test_inheritance_cycle_detection() {
        let mut registry = ClassRegistry::new();
        
        // Create parent class
        let parent = ClassDefinition {
            name: "Parent".to_string(),
            parent: None,
            fields: vec![],
            methods: vec![],
            constructors: vec![],
            visibility: HashMap::new(),
        };
        registry.register_class(parent).unwrap();
        
        // Try to create child that inherits from itself (cycle)
        let child = ClassDefinition {
            name: "Child".to_string(),
            parent: Some("Child".to_string()),
            fields: vec![],
            methods: vec![],
            constructors: vec![],
            visibility: HashMap::new(),
        };
        
        assert!(registry.register_class(child).is_err());
    }

    #[test]
    fn test_value_types() {
        let int_val = Value::Integer(42);
        let str_val = Value::String("hello".to_string());
        let null_val = Value::Null;
        
        assert_eq!(int_val.type_name(), "int");
        assert_eq!(str_val.type_name(), "string");
        assert_eq!(null_val.type_name(), "null");
        
        assert!(int_val.is_truthy());
        assert!(str_val.is_truthy());
        assert!(!null_val.is_truthy());
    }
}
