use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::ast::Program;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::error::CompilerError;

#[derive(Debug, Serialize, Deserialize)]
pub struct LSPMessage {
    pub jsonrpc: String,
    pub id: Option<i64>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<LSPError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LSPError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    pub language_id: String,
    pub version: i64,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub line: u64,
    pub character: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: i64,
    pub code: Option<String>,
    pub source: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: i64,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

pub struct LSPServer {
    documents: HashMap<String, String>,
    ast_cache: HashMap<String, Program>,
}

impl LSPServer {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            ast_cache: HashMap::new(),
        }
    }

    pub fn handle_message(&mut self, message: &str) -> Result<String, CompilerError> {
        let lsp_message: LSPMessage = serde_json::from_str(message)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to parse LSP message: {}", e)))?;
        
        match lsp_message.method.as_deref() {
            Some("initialize") => self.handle_initialize(&lsp_message),
            Some("textDocument/didOpen") => self.handle_did_open(&lsp_message),
            Some("textDocument/didChange") => self.handle_did_change(&lsp_message),
            Some("textDocument/completion") => self.handle_completion(&lsp_message),
            Some("textDocument/definition") => self.handle_definition(&lsp_message),
            Some("textDocument/references") => self.handle_references(&lsp_message),
            Some("textDocument/diagnostic") => self.handle_diagnostic(&lsp_message),
            Some("shutdown") => self.handle_shutdown(&lsp_message),
            _ => Ok(self.create_error_response(&lsp_message, -32601, "Method not found")),
        }
    }

    fn handle_initialize(&self, message: &LSPMessage) -> Result<String, CompilerError> {
        let capabilities = serde_json::json!({
            "textDocumentSync": {
                "openClose": true,
                "change": 2,
                "willSave": false,
                "willSaveWaitUntil": false,
                "save": { "includeText": false }
            },
            "completionProvider": {
                "triggerCharacters": [".", ":", "("],
                "resolveProvider": false
            },
            "definitionProvider": true,
            "referencesProvider": true,
            "diagnosticProvider": {
                "identifier": "nexus",
                "interFileDependencies": true,
                "workspaceDiagnostics": true
            }
        });

        let response = LSPMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::json!({
                "capabilities": capabilities,
                "serverInfo": {
                    "name": "nexus-lsp",
                    "version": "0.1.0"
                }
            })),
            error: None,
        };

        Ok(serde_json::to_string(&response)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to serialize response: {}", e)))?)
    }

    fn handle_did_open(&mut self, message: &LSPMessage) -> Result<String, CompilerError> {
        if let Some(params) = &message.params {
            let document: TextDocumentItem = serde_json::from_value(params.clone())
                .map_err(|e| CompilerError::runtime_error(&format!("Failed to parse document: {}", e)))?;
            
            self.documents.insert(document.uri.clone(), document.text.clone());
            self.update_ast(&document.uri, &document.text)?;
        }
        
        Ok(self.create_success_response(message))
    }

    fn handle_did_change(&mut self, message: &LSPMessage) -> Result<String, CompilerError> {
        if let Some(params) = &message.params {
            let changes: serde_json::Value = serde_json::from_value(params.clone())
                .map_err(|e| CompilerError::runtime_error(&format!("Failed to parse changes: {}", e)))?;
            
            if let Some(text_document) = changes.get("textDocument") {
                if let Some(uri) = text_document.get("uri").and_then(|u| u.as_str()) {
                    if let Some(content_changes) = changes.get("contentChanges") {
                        if let Some(first_change) = content_changes.get(0) {
                            if let Some(text) = first_change.get("text").and_then(|t| t.as_str()) {
                                self.documents.insert(uri.to_string(), text.to_string());
                                self.update_ast(uri, text)?;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(self.create_success_response(message))
    }

    fn handle_completion(&self, message: &LSPMessage) -> Result<String, CompilerError> {
        let mut items = Vec::new();
        
        // Add basic completions
        items.push(CompletionItem {
            label: "fn".to_string(),
            kind: 14, // Function
            detail: Some("Function declaration".to_string()),
            documentation: Some("Declare a new function".to_string()),
            insert_text: Some("fn ${1:name}() {\n\t$0\n}".to_string()),
        });
        
        items.push(CompletionItem {
            label: "let".to_string(),
            kind: 6, // Variable
            detail: Some("Variable declaration".to_string()),
            documentation: Some("Declare a new variable".to_string()),
            insert_text: Some("let ${1:name} = ${2:value};".to_string()),
        });
        
        items.push(CompletionItem {
            label: "if".to_string(),
            kind: 14, // Function
            detail: Some("If statement".to_string()),
            documentation: Some("Conditional statement".to_string()),
            insert_text: Some("if ${1:condition} {\n\t$0\n}".to_string()),
        });
        
        items.push(CompletionItem {
            label: "while".to_string(),
            kind: 14, // Function
            detail: Some("While loop".to_string()),
            documentation: Some("Loop while condition is true".to_string()),
            insert_text: Some("while ${1:condition} {\n\t$0\n}".to_string()),
        });
        
        items.push(CompletionItem {
            label: "return".to_string(),
            kind: 6, // Variable
            detail: Some("Return statement".to_string()),
            documentation: Some("Return from function".to_string()),
            insert_text: Some("return ${1:value};".to_string()),
        });
        
        let response = LSPMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::json!({
                "isIncomplete": false,
                "items": items
            })),
            error: None,
        };

        Ok(serde_json::to_string(&response)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to serialize response: {}", e)))?)
    }

    fn handle_definition(&self, _message: &LSPMessage) -> Result<String, CompilerError> {
        // TODO: Implement go-to-definition
        let response = LSPMessage {
            jsonrpc: "2.0".to_string(),
            id: _message.id,
            method: None,
            params: None,
            result: Some(serde_json::Value::Null),
            error: None,
        };

        Ok(serde_json::to_string(&response)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to serialize response: {}", e)))?)
    }

    fn handle_references(&self, _message: &LSPMessage) -> Result<String, CompilerError> {
        // TODO: Implement find-references
        let response = LSPMessage {
            jsonrpc: "2.0".to_string(),
            id: _message.id,
            method: None,
            params: None,
            result: Some(serde_json::json!([])),
            error: None,
        };

        Ok(serde_json::to_string(&response)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to serialize response: {}", e)))?)
    }

    fn handle_diagnostic(&self, message: &LSPMessage) -> Result<String, CompilerError> {
        let mut diagnostics = Vec::new();
        
        if let Some(params) = &message.params {
            if let Some(uri) = params.get("textDocument").and_then(|td| td.get("uri")).and_then(|u| u.as_str()) {
                if let Some(text) = self.documents.get(uri) {
                    // Parse and analyze the document
                    match self.analyze_document(text) {
                        Ok(_) => {
                            // No errors
                        }
                        Err(e) => {
                            // Add diagnostic for the error
                            diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position { line: 0, character: 0 },
                                    end: Position { line: 0, character: 0 },
                                },
                                severity: 1, // Error
                                code: Some("parse_error".to_string()),
                                source: Some("nexus".to_string()),
                                message: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        let response = LSPMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::json!({
                "diagnostics": diagnostics
            })),
            error: None,
        };

        Ok(serde_json::to_string(&response)
            .map_err(|e| CompilerError::runtime_error(&format!("Failed to serialize response: {}", e)))?)
    }

    fn handle_shutdown(&self, message: &LSPMessage) -> Result<String, CompilerError> {
        Ok(self.create_success_response(message))
    }

    fn update_ast(&mut self, uri: &str, text: &str) -> Result<(), CompilerError> {
        match self.analyze_document(text) {
            Ok(ast) => {
                self.ast_cache.insert(uri.to_string(), ast);
                Ok(())
            }
            Err(e) => {
                // Keep the error but don't fail the LSP operation
                eprintln!("Failed to update AST for {}: {}", uri, e);
                Ok(())
            }
        }
    }

    fn analyze_document(&self, text: &str) -> Result<Program, CompilerError> {
        let mut lexer = Lexer::new(text, "lsp_document".to_string());
        let tokens = lexer.tokenize()
            .map_err(|e| CompilerError::syntax_error(&e))?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| CompilerError::syntax_error(&e))?;
        
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)
            .map_err(|e| CompilerError::semantic_error(&e.to_string()))?;
        
        Ok(ast)
    }

    fn create_success_response(&self, message: &LSPMessage) -> String {
        let response = LSPMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::Value::Null),
            error: None,
        };
        
        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
    }

    fn create_error_response(&self, message: &LSPMessage, code: i64, message_text: &str) -> String {
        let response = LSPMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: None,
            error: Some(LSPError {
                code,
                message: message_text.to_string(),
                data: None,
            }),
        };
        
        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
    }
} 