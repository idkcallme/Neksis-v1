use std::collections::HashMap;
use crate::ast::*;
use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug, Clone)]
pub struct MacroSystem {
    pub macros: HashMap<String, MacroDefinition>,
    pub macro_context: MacroContext,
    pub expansion_history: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MacroDefinition {
    pub name: String,
    pub parameters: Vec<MacroParameter>,
    pub body: MacroBody,
    pub hygiene: HygieneLevel,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MacroParameter {
    pub name: String,
    pub parameter_type: MacroParameterType,
    pub default_value: Option<MacroValue>,
    pub is_optional: bool,
}

#[derive(Debug, Clone)]
pub enum MacroParameterType {
    Expression,
    Statement,
    Type,
    Identifier,
    Literal,
    Pattern,
    Block,
    Repetition,
}

#[derive(Debug, Clone)]
pub struct MacroBody {
    pub template: String,
    pub transformations: Vec<MacroTransformation>,
    pub validation_rules: Vec<MacroValidationRule>,
}

#[derive(Debug, Clone)]
pub enum MacroTransformation {
    Substitution(String, String),
    Conditional(String, MacroCondition),
    Repetition(String, RepetitionRule),
    TypeInference(String),
    CodeGeneration(String),
}

#[derive(Debug, Clone)]
pub struct MacroCondition {
    pub condition: String,
    pub true_branch: String,
    pub false_branch: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RepetitionRule {
    pub variable: String,
    pub separator: Option<String>,
    pub terminator: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MacroValidationRule {
    pub rule_type: ValidationRuleType,
    pub condition: String,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub enum ValidationRuleType {
    TypeCheck,
    SyntaxCheck,
    SemanticCheck,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum MacroValue {
    Expression(Expression),
    Statement(Statement),
    Type(Type),
    Identifier(String),
    Literal(Literal),
    Block(Vec<Expression>),
    Repetition(Vec<MacroValue>),
}

#[derive(Debug, Clone)]
pub enum HygieneLevel {
    Hygienic,    // Variables are renamed to avoid conflicts
    Unhygienic,  // Variables keep their original names
    Selective,   // Some variables are hygienic, others not
}

#[derive(Debug, Clone)]
pub struct MacroContext {
    pub variables: HashMap<String, MacroValue>,
    pub functions: HashMap<String, FunctionStatement>,
    pub types: HashMap<String, Type>,
    pub scope_level: usize,
    pub parent_context: Option<Box<MacroContext>>,
}

impl MacroSystem {
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            macro_context: MacroContext::new(),
            expansion_history: Vec::new(),
        }
    }

    pub fn register_macro(&mut self, definition: MacroDefinition) -> Result<(), CompilerError> {
        // Validate macro definition
        self.validate_macro_definition(&definition)?;
        
        // Register the macro
        self.macros.insert(definition.name.clone(), definition);
        
        Ok(())
    }

    pub fn expand_macro(&mut self, macro_call: &MacroCall) -> Result<Vec<Statement>, CompilerError> {
        let macro_name = &macro_call.name;
        
        // Look up macro definition
        let macro_def = self.macros.get(macro_name)
            .ok_or_else(|| CompilerError::runtime_error(&format!("Macro '{}' not found", macro_name)))?;
        
        // Create expansion context
        let mut context = MacroContext::new();
        context.parent_context = Some(Box::new(self.macro_context.clone()));
        
        // Bind arguments to parameters
        self.bind_macro_arguments(macro_def, &macro_call.arguments, &mut context)?;
        
        // Expand the macro
        let expanded_code = self.expand_macro_body(macro_def, &context)?;
        
        // Parse expanded code
        let statements = self.parse_expanded_code(&expanded_code)?;
        
        // Record expansion
        self.expansion_history.push(format!("{} -> {}", macro_name, expanded_code));
        
        Ok(statements)
    }

    fn validate_macro_definition(&self, definition: &MacroDefinition) -> Result<(), CompilerError> {
        // Check for duplicate macro names
        if self.macros.contains_key(&definition.name) {
            return Err(CompilerError::runtime_error(&format!("Macro '{}' already defined", definition.name)));
        }
        
        // Validate parameter names
        let mut param_names = std::collections::HashSet::new();
        for param in &definition.parameters {
            if param_names.contains(&param.name) {
                return Err(CompilerError::runtime_error(&format!("Duplicate parameter name '{}' in macro '{}'", param.name, definition.name)));
            }
            param_names.insert(param.name.clone());
        }
        
        // Validate macro body
        self.validate_macro_body(&definition.body)?;
        
        Ok(())
    }

    fn validate_macro_body(&self, body: &MacroBody) -> Result<(), CompilerError> {
        // Validate template syntax
        if body.template.is_empty() {
            return Err(CompilerError::runtime_error("Macro body template cannot be empty"));
        }
        
        // Validate transformations
        for transformation in &body.transformations {
            self.validate_transformation(transformation)?;
        }
        
        // Validate validation rules
        for rule in &body.validation_rules {
            self.validate_validation_rule(rule)?;
        }
        
        Ok(())
    }

    fn validate_transformation(&self, transformation: &MacroTransformation) -> Result<(), CompilerError> {
        match transformation {
            MacroTransformation::Substitution(from, to) => {
                if from.is_empty() {
                    return Err(CompilerError::runtime_error("Substitution 'from' pattern cannot be empty"));
                }
            }
            MacroTransformation::Conditional(pattern, condition) => {
                if pattern.is_empty() {
                    return Err(CompilerError::runtime_error("Conditional pattern cannot be empty"));
                }
                if condition.condition.is_empty() {
                    return Err(CompilerError::runtime_error("Conditional condition cannot be empty"));
                }
            }
            MacroTransformation::Repetition(pattern, rule) => {
                if pattern.is_empty() {
                    return Err(CompilerError::runtime_error("Repetition pattern cannot be empty"));
                }
                if rule.variable.is_empty() {
                    return Err(CompilerError::runtime_error("Repetition variable cannot be empty"));
                }
            }
            MacroTransformation::TypeInference(pattern) => {
                if pattern.is_empty() {
                    return Err(CompilerError::runtime_error("Type inference pattern cannot be empty"));
                }
            }
            MacroTransformation::CodeGeneration(pattern) => {
                if pattern.is_empty() {
                    return Err(CompilerError::runtime_error("Code generation pattern cannot be empty"));
                }
            }
        }
        
        Ok(())
    }

    fn validate_validation_rule(&self, rule: &MacroValidationRule) -> Result<(), CompilerError> {
        if rule.condition.is_empty() {
            return Err(CompilerError::runtime_error("Validation rule condition cannot be empty"));
        }
        
        if rule.error_message.is_empty() {
            return Err(CompilerError::runtime_error("Validation rule error message cannot be empty"));
        }
        
        Ok(())
    }

    fn bind_macro_arguments(&self, macro_def: &MacroDefinition, arguments: &[MacroValue], context: &mut MacroContext) -> Result<(), CompilerError> {
        if arguments.len() > macro_def.parameters.len() {
            return Err(CompilerError::runtime_error(&format!("Too many arguments for macro '{}'", macro_def.name)));
        }
        
        for (i, param) in macro_def.parameters.iter().enumerate() {
            if i < arguments.len() {
                let arg = &arguments[i];
                self.validate_argument_type(arg, param)?;
                context.variables.insert(param.name.clone(), arg.clone());
            } else if !param.is_optional {
                return Err(CompilerError::runtime_error(&format!("Required parameter '{}' not provided for macro '{}'", param.name, macro_def.name)));
            } else if let Some(default) = &param.default_value {
                context.variables.insert(param.name.clone(), default.clone());
            }
        }
        
        Ok(())
    }

    fn validate_argument_type(&self, argument: &MacroValue, parameter: &MacroParameter) -> Result<(), CompilerError> {
        let expected_type = &parameter.parameter_type;
        
        match (argument, expected_type) {
            (MacroValue::Expression(_), MacroParameterType::Expression) => Ok(()),
            (MacroValue::Statement(_), MacroParameterType::Statement) => Ok(()),
            (MacroValue::Type(_), MacroParameterType::Type) => Ok(()),
            (MacroValue::Identifier(_), MacroParameterType::Identifier) => Ok(()),
            (MacroValue::Literal(_), MacroParameterType::Literal) => Ok(()),
            (MacroValue::Block(_), MacroParameterType::Block) => Ok(()),
            (MacroValue::Repetition(_), MacroParameterType::Repetition) => Ok(()),
            _ => {
                Err(CompilerError::runtime_error(&format!(
                    "Argument type mismatch: expected {:?}, got {:?}",
                    expected_type, argument
                )))
            }
        }
    }

    fn expand_macro_body(&self, macro_def: &MacroDefinition, context: &MacroContext) -> Result<String, CompilerError> {
        let mut expanded = macro_def.body.template.clone();
        
        // Apply transformations
        for transformation in &macro_def.body.transformations {
            expanded = self.apply_transformation(transformation, &expanded, context)?;
        }
        
        // Apply validation rules
        for rule in &macro_def.body.validation_rules {
            self.apply_validation_rule(rule, &expanded)?;
        }
        
        Ok(expanded)
    }

    fn apply_transformation(&self, transformation: &MacroTransformation, template: &str, context: &MacroContext) -> Result<String, CompilerError> {
        match transformation {
            MacroTransformation::Substitution(from, to) => {
                let result = template.replace(from, to);
                Ok(result)
            }
            MacroTransformation::Conditional(pattern, condition) => {
                let condition_met = self.evaluate_condition(&condition.condition, context)?;
                if condition_met {
                    let result = template.replace(pattern, &condition.true_branch);
                    Ok(result)
                } else if let Some(false_branch) = &condition.false_branch {
                    let result = template.replace(pattern, false_branch);
                    Ok(result)
                } else {
                    let result = template.replace(pattern, "");
                    Ok(result)
                }
            }
            MacroTransformation::Repetition(pattern, rule) => {
                let variable_value = context.variables.get(&rule.variable)
                    .ok_or_else(|| CompilerError::runtime_error(&format!("Variable '{}' not found in macro context", rule.variable)))?;
                
                match variable_value {
                    MacroValue::Repetition(items) => {
                        let mut result = String::new();
                        for (i, item) in items.iter().enumerate() {
                            let item_str = self.format_macro_value(item);
                            let expanded = pattern.replace(&format!("${}", rule.variable), &item_str);
                            
                            if i > 0 {
                                if let Some(separator) = &rule.separator {
                                    result.push_str(separator);
                                }
                            }
                            
                            result.push_str(&expanded);
                        }
                        
                        if let Some(terminator) = &rule.terminator {
                            result.push_str(terminator);
                        }
                        
                        Ok(result)
                    }
                    _ => {
                        Err(CompilerError::runtime_error(&format!("Variable '{}' is not a repetition", rule.variable)))
                    }
                }
            }
            MacroTransformation::TypeInference(pattern) => {
                // Apply type inference
                let inferred_type = self.infer_type_from_context(context)?;
                let result = template.replace(pattern, &inferred_type);
                Ok(result)
            }
            MacroTransformation::CodeGeneration(pattern) => {
                // Generate code based on context
                let generated_code = self.generate_code_from_context(context)?;
                let result = template.replace(pattern, &generated_code);
                Ok(result)
            }
        }
    }

    fn evaluate_condition(&self, condition: &str, context: &MacroContext) -> Result<bool, CompilerError> {
        // Simple condition evaluation
        // In a real implementation, this would be more sophisticated
        match condition {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                // Check if variable exists and is truthy
                if let Some(value) = context.variables.get(condition) {
                    match value {
                        MacroValue::Literal(Literal::Bool(b)) => Ok(*b),
                        MacroValue::Literal(Literal::Int(i)) => Ok(*i != 0),
                        MacroValue::Literal(Literal::Float(f)) => Ok(*f != 0.0),
                        _ => Ok(true), // Non-empty values are truthy
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn infer_type_from_context(&self, context: &MacroContext) -> Result<String, CompilerError> {
        // Simple type inference
        // In a real implementation, this would analyze the context more thoroughly
        Ok("auto".to_string())
    }

    fn generate_code_from_context(&self, context: &MacroContext) -> Result<String, CompilerError> {
        // Generate code based on context variables
        let mut code = String::new();
        
        for (name, value) in &context.variables {
            match value {
                MacroValue::Expression(expr) => {
                    code.push_str(&format!("let {} = {};\n", name, self.format_expression(expr)));
                }
                MacroValue::Statement(stmt) => {
                    code.push_str(&format!("{}\n", self.format_statement(stmt)));
                }
                MacroValue::Type(typ) => {
                    code.push_str(&format!("type {} = {};\n", name, self.format_type(typ)));
                }
                _ => {
                    // Handle other value types
                }
            }
        }
        
        Ok(code)
    }

    fn format_macro_value(&self, value: &MacroValue) -> String {
        match value {
            MacroValue::Expression(expr) => self.format_expression(expr),
            MacroValue::Statement(stmt) => self.format_statement(stmt),
            MacroValue::Type(typ) => self.format_type(typ),
            MacroValue::Identifier(id) => id.clone(),
            MacroValue::Literal(lit) => self.format_literal(lit),
            MacroValue::Block(exprs) => {
                let exprs_str: Vec<String> = exprs.iter()
                    .map(|e| self.format_expression(e))
                    .collect();
                format!("{{ {} }}", exprs_str.join("; "))
            }
            MacroValue::Repetition(items) => {
                let items_str: Vec<String> = items.iter()
                    .map(|i| self.format_macro_value(i))
                    .collect();
                format!("[{}]", items_str.join(", "))
            }
        }
    }

    fn format_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(lit) => self.format_literal(lit),
            Expression::Identifier(name) => name.clone(),
            Expression::BinaryOperation { left, operator, right } => {
                format!("{} {} {}", 
                    self.format_expression(left),
                    self.format_binary_operator(operator),
                    self.format_expression(right))
            }
            Expression::FunctionCall { name, arguments } => {
                let args_str: Vec<String> = arguments.iter()
                    .map(|a| self.format_expression(a))
                    .collect();
                format!("{}({})", name, args_str.join(", "))
            }
            _ => format!("{:?}", expr),
        }
    }

    fn format_statement(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Let(let_stmt) => {
                format!("let {} = {};", 
                    let_stmt.name, 
                    self.format_expression(&let_stmt.value))
            }
            Statement::Function(func) => {
                format!("fn {}({}) -> {} {{ ... }}", 
                    func.name,
                    func.signature.parameters.iter()
                        .map(|p| format!("{}: {}", p.name, self.format_type(&p.param_type)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    func.signature.return_type.as_ref()
                        .map(|t| self.format_type(t))
                        .unwrap_or_else(|| "void".to_string()))
            }
            _ => format!("{:?}", stmt),
        }
    }

    fn format_type(&self, typ: &Type) -> String {
        match typ {
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::String => "String".to_string(),
            Type::Void => "Void".to_string(),
            Type::Pointer(inner) => format!("*{}", self.format_type(inner)),
            _ => format!("{:?}", typ),
        }
    }

    fn format_literal(&self, lit: &Literal) -> String {
        match lit {
            Literal::Int(value) => value.to_string(),
            Literal::Float(value) => value.to_string(),
            Literal::Bool(value) => value.to_string(),
            Literal::String(value) => format!("\"{}\"", value),
            _ => format!("{:?}", lit),
        }
    }

    fn format_binary_operator(&self, op: &BinaryOperator) -> &str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::LessThanOrEqual => "<=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::GreaterThanOrEqual => ">=",
            BinaryOperator::And => "&&",
            BinaryOperator::Or => "||",
            BinaryOperator::Assign => "=",
            BinaryOperator::AddAssign => "+=",
            BinaryOperator::SubtractAssign => "-=",
            BinaryOperator::MultiplyAssign => "*=",
            BinaryOperator::DivideAssign => "/=",
        }
    }

    fn apply_validation_rule(&self, rule: &MacroValidationRule, expanded_code: &str) -> Result<(), CompilerError> {
        // Simple validation - in a real implementation, this would be more sophisticated
        match rule.rule_type {
            ValidationRuleType::TypeCheck => {
                // Check if the expanded code has valid types
                if expanded_code.contains("type_error") {
                    return Err(CompilerError::runtime_error(&rule.error_message));
                }
            }
            ValidationRuleType::SyntaxCheck => {
                // Check if the expanded code has valid syntax
                if expanded_code.contains("syntax_error") {
                    return Err(CompilerError::runtime_error(&rule.error_message));
                }
            }
            ValidationRuleType::SemanticCheck => {
                // Check if the expanded code has valid semantics
                if expanded_code.contains("semantic_error") {
                    return Err(CompilerError::runtime_error(&rule.error_message));
                }
            }
            ValidationRuleType::Custom(_) => {
                // Custom validation logic
                if expanded_code.contains("custom_error") {
                    return Err(CompilerError::runtime_error(&rule.error_message));
                }
            }
        }
        
        Ok(())
    }

    fn parse_expanded_code(&self, code: &str) -> Result<Vec<Statement>, CompilerError> {
        // Parse the expanded macro code
        let mut lexer = Lexer::new(code, "macro_expansion".to_string());
        let tokens = lexer.tokenize()
            .map_err(|e| CompilerError::parse_error("lexer", &e))?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse()
            .map_err(|e| CompilerError::parse_error("parser", &e))?;

        Ok(program.statements)
    }

    pub fn register_builtin_macros(&mut self) -> Result<(), CompilerError> {
        // Register common built-in macros
        
        // println! macro
        let println_macro = MacroDefinition {
            name: "println!".to_string(),
            parameters: vec![
                MacroParameter {
                    name: "format".to_string(),
                    parameter_type: MacroParameterType::Literal,
                    default_value: None,
                    is_optional: false,
                },
                MacroParameter {
                    name: "args".to_string(),
                    parameter_type: MacroParameterType::Repetition,
                    default_value: None,
                    is_optional: true,
                },
            ],
            body: MacroBody {
                template: "print(format!($format, $args...));".to_string(),
                transformations: vec![
                    MacroTransformation::Substitution("$format".to_string(), "format".to_string()),
                    MacroTransformation::Repetition("$args...".to_string(), RepetitionRule {
                        variable: "args".to_string(),
                        separator: Some(", ".to_string()),
                        terminator: None,
                    }),
                ],
                validation_rules: vec![],
            },
            hygiene: HygieneLevel::Hygienic,
            documentation: Some("Print a formatted string with newline".to_string()),
        };
        self.register_macro(println_macro)?;

        // vec! macro
        let vec_macro = MacroDefinition {
            name: "vec!".to_string(),
            parameters: vec![
                MacroParameter {
                    name: "elements".to_string(),
                    parameter_type: MacroParameterType::Repetition,
                    default_value: None,
                    is_optional: false,
                },
            ],
            body: MacroBody {
                template: "[$elements...]".to_string(),
                transformations: vec![
                    MacroTransformation::Repetition("$elements...".to_string(), RepetitionRule {
                        variable: "elements".to_string(),
                        separator: Some(", ".to_string()),
                        terminator: None,
                    }),
                ],
                validation_rules: vec![],
            },
            hygiene: HygieneLevel::Hygienic,
            documentation: Some("Create a vector from elements".to_string()),
        };
        self.register_macro(vec_macro)?;

        // assert! macro
        let assert_macro = MacroDefinition {
            name: "assert!".to_string(),
            parameters: vec![
                MacroParameter {
                    name: "condition".to_string(),
                    parameter_type: MacroParameterType::Expression,
                    default_value: None,
                    is_optional: false,
                },
                MacroParameter {
                    name: "message".to_string(),
                    parameter_type: MacroParameterType::Literal,
                    default_value: Some(MacroValue::Literal(Literal::String("Assertion failed".to_string()))),
                    is_optional: true,
                },
            ],
            body: MacroBody {
                template: "if !($condition) {{ panic!($message); }}".to_string(),
                transformations: vec![
                    MacroTransformation::Substitution("$condition".to_string(), "condition".to_string()),
                    MacroTransformation::Substitution("$message".to_string(), "message".to_string()),
                ],
                validation_rules: vec![],
            },
            hygiene: HygieneLevel::Hygienic,
            documentation: Some("Assert that a condition is true".to_string()),
        };
        self.register_macro(assert_macro)?;

        Ok(())
    }
}

impl MacroContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            scope_level: 0,
            parent_context: None,
        }
    }

    pub fn with_parent(parent: MacroContext) -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            scope_level: parent.scope_level + 1,
            parent_context: Some(Box::new(parent)),
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&MacroValue> {
        self.variables.get(name).or_else(|| {
            self.parent_context.as_ref().and_then(|parent| parent.get_variable(name))
        })
    }

    pub fn set_variable(&mut self, name: String, value: MacroValue) {
        self.variables.insert(name, value);
    }
}

// Macro call representation
#[derive(Debug, Clone)]
pub struct MacroCall {
    pub name: String,
    pub arguments: Vec<MacroValue>,
    pub location: String,
}

// DSL (Domain Specific Language) support
#[derive(Debug, Clone)]
pub struct DSL {
    pub name: String,
    pub grammar: DSLGrammar,
    pub transformations: Vec<DSLTransformation>,
    pub macros: HashMap<String, MacroDefinition>,
}

#[derive(Debug, Clone)]
pub struct DSLGrammar {
    pub rules: Vec<GrammarRule>,
    pub tokens: Vec<String>,
    pub start_symbol: String,
}

#[derive(Debug, Clone)]
pub struct GrammarRule {
    pub name: String,
    pub productions: Vec<Production>,
}

#[derive(Debug, Clone)]
pub struct Production {
    pub symbols: Vec<String>,
    pub action: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DSLTransformation {
    pub pattern: String,
    pub replacement: String,
    pub conditions: Vec<String>,
}

impl DSL {
    pub fn new(name: String) -> Self {
        Self {
            name,
            grammar: DSLGrammar {
                rules: Vec::new(),
                tokens: Vec::new(),
                start_symbol: "start".to_string(),
            },
            transformations: Vec::new(),
            macros: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: GrammarRule) {
        self.grammar.rules.push(rule);
    }

    pub fn add_transformation(&mut self, transformation: DSLTransformation) {
        self.transformations.push(transformation);
    }

    pub fn add_macro(&mut self, name: String, macro_def: MacroDefinition) {
        self.macros.insert(name, macro_def);
    }

    pub fn parse_and_transform(&self, input: &str) -> Result<String, CompilerError> {
        // Parse input according to DSL grammar
        let parsed = self.parse_dsl(input)?;
        
        // Apply transformations
        let mut result = parsed;
        for transformation in &self.transformations {
            result = self.apply_dsl_transformation(transformation, &result)?;
        }
        
        Ok(result)
    }

    fn parse_dsl(&self, input: &str) -> Result<String, CompilerError> {
        // Simple DSL parsing - in a real implementation, this would be more sophisticated
        Ok(input.to_string())
    }

    fn apply_dsl_transformation(&self, transformation: &DSLTransformation, input: &str) -> Result<String, CompilerError> {
        // Apply DSL transformation
        let result = input.replace(&transformation.pattern, &transformation.replacement);
        Ok(result)
    }
}

// Macro utilities
pub fn create_macro_system() -> MacroSystem {
    let mut system = MacroSystem::new();
    system.register_builtin_macros().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to register builtin macros: {}", e);
    });
    system
}

pub fn create_dsl(name: String) -> DSL {
    DSL::new(name)
} 