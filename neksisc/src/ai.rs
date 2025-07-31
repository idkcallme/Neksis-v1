use crate::ast::{Annotation, Program};
use crate::error::CompilerError;
use serde_json::Value;

pub struct AiProcessor {
    api_key: Option<String>,
    api_endpoint: String,
}

impl AiProcessor {
    pub fn new() -> Self {
        Self {
            api_key: std::env::var("NEXUS_AI_API_KEY").ok(),
            api_endpoint: std::env::var("NEXUS_AI_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string()),
        }
    }
    
    pub fn generate_test(&self, annotation: &Annotation, ast: &Program) -> Result<String, CompilerError> {
        let prompt = self.build_test_prompt(annotation, ast)?;
        let response = self.call_llm_api(&prompt)?;
        
        // Extract the generated test code from the response
        self.extract_code_from_response(&response)
    }
    
    pub fn generate_optimization_hints(&self, annotation: &Annotation, ast: &Program) -> Result<Vec<String>, CompilerError> {
        let prompt = self.build_optimization_prompt(annotation, ast)?;
        let response = self.call_llm_api(&prompt)?;
        
        // Extract optimization hints from the response
        self.extract_optimization_hints(&response)
    }
    
    fn build_test_prompt(&self, annotation: &Annotation, ast: &Program) -> Result<String, CompilerError> {
        let binding = "unknown".to_string();
        let function_name = annotation.attached_to.as_ref()
            .unwrap_or(&binding);
        
        // Find the function in the AST
        let function = self.find_function(ast, function_name).unwrap_or_else(|_| {
            // Return a dummy function if not found
            crate::ast::FunctionStatement {
                name: function_name.clone(),
                parameters: vec![],
                return_type: None,
                body: Box::new(crate::ast::Expression::Literal(crate::ast::Literal::Int(0))),
                annotations: vec![],
                signature: crate::ast::FunctionSignature {
                    parameters: vec![],
                    return_type: None,
                },
            }
        });
        
        let prompt = format!(
            r#"SYSTEM: You are an expert neksis language test generation bot. Your output must be only valid neksis code. Do not add any commentary.

USER:
## TASK
Generate a unit test for the following function based on the user's request.

## USER REQUEST
"{}"

## FUNCTION SOURCE CODE
{}

## REQUIRED OUTPUT FORMAT
// A single neksis function starting with `fn test_...`.
"#,
            annotation.arguments.iter().map(|arg| format!("{:?}", arg)).collect::<Vec<_>>().join(" "),
            self.format_function(function)
        );
        
        Ok(prompt)
    }
    
    fn build_optimization_prompt(&self, annotation: &Annotation, ast: &Program) -> Result<String, CompilerError> {
        let binding = "unknown".to_string();
        let function_name = annotation.attached_to.as_ref()
            .unwrap_or(&binding);
        
        let function = self.find_function(ast, function_name).unwrap_or_else(|_| {
            // Return a dummy function if not found
            crate::ast::FunctionStatement {
                name: function_name.clone(),
                parameters: vec![],
                return_type: None,
                body: Box::new(crate::ast::Expression::Literal(crate::ast::Literal::Int(0))),
                annotations: vec![],
                signature: crate::ast::FunctionSignature {
                    parameters: vec![],
                    return_type: None,
                },
            }
        });
        
        let prompt = format!(
            r#"SYSTEM: You are an expert compiler optimization advisor. Analyze the following function and suggest optimization strategies.

USER:
## TASK
Analyze the following function and suggest optimization strategies for: {}

## FUNCTION SOURCE CODE
{}

## REQUIRED OUTPUT FORMAT
Provide a JSON array of optimization hints.
"#,
            annotation.arguments.iter().map(|arg| format!("{:?}", arg)).collect::<Vec<_>>().join(" "),
            self.format_function(function)
        );
        
        Ok(prompt)
    }
    
    fn call_llm_api(&self, _prompt: &str) -> Result<String, CompilerError> {
        // TODO: Implement actual API call to LLM service
        // This would use reqwest to make HTTP requests to OpenAI, Anthropic, etc.
        
        // For now, return a placeholder response
        Ok(r#"fn test_example_handles_empty_lists() {
    let empty_list: [Int] = []
    let result = process_list(empty_list)
    assert(result == 0, "Processing an empty list should result in 0")
}"#.to_string())
    }
    
    fn extract_code_from_response(&self, response: &str) -> Result<String, CompilerError> {
        // TODO: Implement proper code extraction from LLM response
        // This should parse the response and extract only the neksis code
        Ok(response.to_string())
    }
    
    fn extract_optimization_hints(&self, _response: &str) -> Result<Vec<String>, CompilerError> {
        // TODO: Parse JSON response and extract optimization hints
        Ok(vec!["vectorize".to_string(), "inline".to_string()])
    }
    
    fn find_function(&self, ast: &Program, name: &str) -> Result<crate::ast::FunctionStatement, CompilerError> {
        for statement in &ast.statements {
            if let crate::ast::Statement::Function(func) = statement {
                if func.name == name {
                    return Ok(func.clone());
                }
            }
        }
        
        Err(CompilerError::ai_error(&format!("Function '{}' not found", name)))
    }
    
    fn format_function(&self, function: crate::ast::FunctionStatement) -> String {
        // TODO: Implement proper function formatting
        format!("fn {}() {{ ... }}", function.name)
    }
} 