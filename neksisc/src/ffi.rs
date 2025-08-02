use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char};
use std::collections::HashMap;
use crate::ast::Type;
use crate::error::CompilerError;


#[derive(Debug)]
pub struct FFILibrary {
    pub name: String,
    pub functions: HashMap<String, FFIFunction>,
    pub handle: Option<libloading::Library>,
}

#[derive(Debug, Clone)]
pub struct FFIFunction {
    pub name: String,
    pub signature: FFISignature,
    pub symbol: Option<libloading::Symbol<'static, fn()>>,
}

#[derive(Debug, Clone)]
pub struct FFISignature {
    pub return_type: FFIType,
    pub parameters: Vec<FFIParameter>,
    pub calling_convention: CallingConvention,
}

#[derive(Debug, Clone)]
pub struct FFIParameter {
    pub name: String,
    pub ffi_type: FFIType,
    pub direction: ParameterDirection,
}

#[derive(Debug, Clone)]
pub enum FFIType {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Bool,
    String,
    Pointer(Box<FFIType>),
    Array(Box<FFIType>, usize),
    Struct(Vec<FFIField>),
    Union(Vec<FFIField>),
    Function(Box<FFISignature>),
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct FFIField {
    pub name: String,
    pub ffi_type: FFIType,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub enum ParameterDirection {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone)]
pub enum CallingConvention {
    C,
    StdCall,
    FastCall,
    ThisCall,
    VectorCall,
    Custom(String),
}

#[derive(Debug)]
pub struct FFIContext {
    pub libraries: HashMap<String, FFILibrary>,
    pub type_mappings: HashMap<String, FFIType>,
    pub memory_manager: FFIMemoryManager,
}

#[derive(Debug, Clone)]
pub struct FFIMemoryManager {
    pub allocations: HashMap<*mut c_void, AllocationInfo>,
    pub next_id: usize,
}

#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub id: usize,
    pub size: usize,
    pub ffi_type: FFIType,
    pub is_managed: bool,
}

impl FFIContext {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
            type_mappings: HashMap::new(),
            memory_manager: FFIMemoryManager::new(),
        }
    }

    pub fn load_library(&mut self, name: &str, path: &str) -> Result<(), CompilerError> {
        unsafe {
            let library = libloading::Library::new(path)
                .map_err(|e| CompilerError::ffi_error("library", &format!("Failed to load library: {}", e)))?;

            let mut ffi_library = FFILibrary {
                name: name.to_string(),
                functions: HashMap::new(),
                handle: Some(library),
            };

            // Register common functions
            self.register_common_functions(&mut ffi_library)?;

            self.libraries.insert(name.to_string(), ffi_library);
            Ok(())
        }
    }

    fn register_common_functions(&self, library: &mut FFILibrary) -> Result<(), CompilerError> {
        // Register malloc/free
        let malloc_sig = FFISignature {
            return_type: FFIType::Pointer(Box::new(FFIType::Void)),
            parameters: vec![
                FFIParameter {
                    name: "size".to_string(),
                    ffi_type: FFIType::UInt64,
                    direction: ParameterDirection::In,
                }
            ],
            calling_convention: CallingConvention::C,
        };

        let free_sig = FFISignature {
            return_type: FFIType::Void,
            parameters: vec![
                FFIParameter {
                    name: "ptr".to_string(),
                    ffi_type: FFIType::Pointer(Box::new(FFIType::Void)),
                    direction: ParameterDirection::In,
                }
            ],
            calling_convention: CallingConvention::C,
        };

        library.functions.insert("malloc".to_string(), FFIFunction {
            name: "malloc".to_string(),
            signature: malloc_sig,
            symbol: None,
        });

        library.functions.insert("free".to_string(), FFIFunction {
            name: "free".to_string(),
            signature: free_sig,
            symbol: None,
        });

        Ok(())
    }

    pub fn call_function(&mut self, library_name: &str, function_name: &str, args: Vec<FFIValue>) -> Result<FFIValue, CompilerError> {
        // Get library and function signatures without holding mutable borrow
        let (signature, return_type) = {
            let library = self.libraries.get(library_name)
                .ok_or_else(|| CompilerError::ffi_error("library", &format!("Library '{}' not found", library_name)))?;
            
            let function = library.functions.get(function_name)
                .ok_or_else(|| CompilerError::ffi_error("function", &format!("Function '{}' not found", function_name)))?;
            
            (function.signature.clone(), function.signature.return_type.clone())
        };

        // Validate arguments
        self.validate_function_call(&signature, &args)?;

        // Convert arguments to C types
        let c_args = self.convert_to_c_args(&signature.parameters, args)?;

        // Call the function
        let result = unsafe {
            self.execute_function_call(&signature, &c_args)?
        };

        // Convert result back to Neksis type
        let neksis_result = self.convert_from_c_value(&return_type, result)?;

        Ok(neksis_result)
    }

    fn validate_function_call(&self, signature: &FFISignature, args: &[FFIValue]) -> Result<(), CompilerError> {
        if args.len() != signature.parameters.len() {
            return Err(CompilerError::ffi_error("arguments", "Argument count mismatch"));
        }

        for (i, (arg, param)) in args.iter().zip(signature.parameters.iter()).enumerate() {
            // We need to infer the FFI type from the value since FFIValue doesn't have ffi_type field
            let arg_type = self.infer_ffi_type_from_value(arg);
            if !self.is_compatible_type(&arg_type, &param.ffi_type) {
                return Err(CompilerError::ffi_error("type", &format!("Argument {} type mismatch", i)));
            }
        }

        Ok(())
    }

    fn is_compatible_type(&self, from: &FFIType, to: &FFIType) -> bool {
        match (from, to) {
            (FFIType::Int32, FFIType::Int32) => true,
            (FFIType::Float64, FFIType::Float64) => true,
            (FFIType::Pointer(_), FFIType::Pointer(_)) => true,
            (FFIType::Bool, FFIType::Bool) => true,
            _ => false, // Add more compatibility rules as needed
        }
    }

    fn infer_ffi_type_from_value(&self, value: &FFIValue) -> FFIType {
        match value {
            FFIValue::Void => FFIType::Void,
            FFIValue::Int8(_) => FFIType::Int8,
            FFIValue::Int16(_) => FFIType::Int16,
            FFIValue::Int32(_) => FFIType::Int32,
            FFIValue::Int64(_) => FFIType::Int64,
            FFIValue::UInt8(_) => FFIType::UInt8,
            FFIValue::UInt16(_) => FFIType::UInt16,
            FFIValue::UInt32(_) => FFIType::UInt32,
            FFIValue::UInt64(_) => FFIType::UInt64,
            FFIValue::Float32(_) => FFIType::Float32,
            FFIValue::Float64(_) => FFIType::Float64,
            FFIValue::Bool(_) => FFIType::Bool,
            FFIValue::Pointer(_) => FFIType::Pointer(Box::new(FFIType::Void)),
            FFIValue::String(_) => FFIType::String,
            FFIValue::Array(_) => FFIType::Array(Box::new(FFIType::Void), 0),
            FFIValue::Struct(_) => FFIType::Struct(vec![]),
        }
    }

    fn convert_to_c_args(&self, parameters: &[FFIParameter], args: Vec<FFIValue>) -> Result<Vec<FFIValue>, CompilerError> {
        let mut c_args = Vec::new();

        for (param, arg) in parameters.iter().zip(args.iter()) {
            let converted = self.convert_to_c_value(&param.ffi_type, arg)?;
            c_args.push(converted);
        }

        Ok(c_args)
    }

    fn convert_to_c_value(&self, target_type: &FFIType, value: &FFIValue) -> Result<FFIValue, CompilerError> {
        match (target_type, value) {
            (FFIType::Int32, FFIValue::Int32(v)) => Ok(FFIValue::Int32(*v)),
            (FFIType::Float64, FFIValue::Float64(v)) => Ok(FFIValue::Float64(*v)),
            (FFIType::Bool, FFIValue::Bool(v)) => Ok(FFIValue::Bool(*v)),
            (FFIType::Pointer(_), FFIValue::Pointer(p)) => Ok(FFIValue::Pointer(*p)),
            (FFIType::String, FFIValue::String(s)) => {
                // Convert string to C string
                let c_string = CString::new(s.as_str())
                    .map_err(|e| CompilerError::ffi_error("string", &format!("Invalid string: {}", e)))?;
                let ptr = c_string.into_raw();
                Ok(FFIValue::Pointer(ptr as *mut c_void))
            }
            _ => Err(CompilerError::ffi_error("conversion", "Unsupported type conversion")),
        }
    }

    fn convert_from_c_value(&self, target_type: &FFIType, value: FFIValue) -> Result<FFIValue, CompilerError> {
        match target_type {
            FFIType::Int32 => Ok(value),
            FFIType::Float64 => Ok(value),
            FFIType::Bool => Ok(value),
            FFIType::Pointer(_) => Ok(value),
            FFIType::String => {
                // Convert C string back to Neksis string
                match value {
                    FFIValue::Pointer(ptr) => {
                        if ptr.is_null() {
                            Ok(FFIValue::String("".to_string()))
                        } else {
                            unsafe {
                                let c_str = CStr::from_ptr(ptr as *const c_char);
                                let string = c_str.to_string_lossy().to_string();
                                Ok(FFIValue::String(string))
                            }
                        }
                    }
                    _ => Err(CompilerError::ffi_error("conversion", "Expected pointer for string conversion")),
                }
            }
            _ => Err(CompilerError::ffi_error("conversion", "Unsupported return type conversion")),
        }
    }

    unsafe fn execute_function_call(&self, signature: &FFISignature, _args: &[FFIValue]) -> Result<FFIValue, CompilerError> {
        // This is a simplified implementation
        // In a real implementation, you would use libffi or similar to call the function
        match signature.return_type {
            FFIType::Int32 => Ok(FFIValue::Int32(0)), // Placeholder
            FFIType::Float64 => Ok(FFIValue::Float64(0.0)), // Placeholder
            FFIType::Bool => Ok(FFIValue::Bool(false)), // Placeholder
            FFIType::Void => Ok(FFIValue::Void), // Placeholder
            _ => Err(CompilerError::ffi_error("call", "Unsupported return type")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FFIValue {
    Void,
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    Pointer(*mut c_void),
    String(String),
    Array(Vec<FFIValue>),
    Struct(HashMap<String, FFIValue>),
}

impl FFIMemoryManager {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn allocate(&mut self, size: usize, ffi_type: FFIType) -> Result<*mut c_void, CompilerError> {
        unsafe {
            let ptr = libc::malloc(size);
            if ptr.is_null() {
                return Err(CompilerError::ffi_error("memory", "Failed to allocate memory"));
            }

            let allocation = AllocationInfo {
                id: self.next_id,
                size,
                ffi_type,
                is_managed: true,
            };

            self.allocations.insert(ptr, allocation);
            self.next_id += 1;

            Ok(ptr)
        }
    }

    pub fn deallocate(&mut self, ptr: *mut c_void) -> Result<(), CompilerError> {
        if let Some(_allocation) = self.allocations.remove(&ptr) {
            unsafe {
                libc::free(ptr);
            }
            Ok(())
        } else {
            Err(CompilerError::ffi_error("memory", "Attempted to free unmanaged pointer"))
        }
    }

    pub fn reallocate(&mut self, ptr: *mut c_void, new_size: usize) -> Result<*mut c_void, CompilerError> {
        unsafe {
            let new_ptr = libc::realloc(ptr, new_size);
            if new_ptr.is_null() {
                return Err(CompilerError::ffi_error("memory", "Failed to reallocate memory"));
            }

            // Update allocation info
            if let Some(mut allocation) = self.allocations.remove(&ptr) {
                allocation.size = new_size;
                self.allocations.insert(new_ptr, allocation);
            }

            Ok(new_ptr)
        }
    }
}

// Python interop support
#[derive(Clone)]
pub struct PythonInterop {
    pub interpreter: Option<pyo3::Python<'static>>,
    pub modules: HashMap<String, pyo3::PyObject>,
}

impl PythonInterop {
    pub fn new() -> Result<Self, CompilerError> {
        // Initialize Python interpreter
        let _interpreter: Result<(), CompilerError> = pyo3::Python::with_gil(|_py| {
            // Set up Python environment
            Ok(())
        });

        Ok(Self {
            interpreter: None, // Will be set when needed
            modules: HashMap::new(),
        })
    }

    pub fn call_python_function(&self, _function_name: &str, _args: Vec<FFIValue>) -> Result<FFIValue, CompilerError> {
        // For now, return a placeholder since Python interop is not fully implemented
        Err(CompilerError::ffi_error("Python", "Python interop not yet implemented"))
    }


}

// Rust interop support
#[derive(Debug, Clone)]
pub struct RustInterop {
    pub crates: HashMap<String, String>, // crate_name -> path
}

impl RustInterop {
    pub fn new() -> Self {
        Self {
            crates: HashMap::new(),
        }
    }

    pub fn add_crate(&mut self, name: &str, path: &str) {
        self.crates.insert(name.to_string(), path.to_string());
    }

    pub fn call_rust_function(&self, _crate_name: &str, _function_name: &str, _args: Vec<FFIValue>) -> Result<FFIValue, CompilerError> {
        // This would involve dynamic linking to Rust libraries
        // For now, we'll return a placeholder
        Err(CompilerError::ffi_error("rust", "Rust interop not yet implemented"))
    }
}

// FFI utilities
pub fn create_ffi_context() -> FFIContext {
    FFIContext::new()
}

pub fn load_c_library(context: &mut FFIContext, name: &str, path: &str) -> Result<(), CompilerError> {
    context.load_library(name, path)
}

pub fn call_c_function(context: &mut FFIContext, library: &str, function: &str, args: Vec<FFIValue>) -> Result<FFIValue, CompilerError> {
    context.call_function(library, function, args)
}

pub fn create_python_interop() -> Result<PythonInterop, CompilerError> {
    PythonInterop::new()
}

pub fn create_rust_interop() -> RustInterop {
    RustInterop::new()
}

// Type conversion utilities
pub fn neksis_to_ffi_type(neksis_type: &Type) -> FFIType {
    match neksis_type {
        Type::Int => FFIType::Int32,
        Type::Float => FFIType::Float64,
        Type::Bool => FFIType::Bool,
        Type::String => FFIType::String,
        Type::Void => FFIType::Void,
        Type::Pointer(inner) => FFIType::Pointer(Box::new(neksis_to_ffi_type(inner))),
        _ => FFIType::Custom(format!("{:?}", neksis_type)),
    }
}

pub fn ffi_to_neksis_type(ffi_type: &FFIType) -> Type {
    match ffi_type {
        FFIType::Int32 => Type::Int,
        FFIType::Float64 => Type::Float,
        FFIType::Bool => Type::Bool,
        FFIType::String => Type::String,
        FFIType::Void => Type::Void,
        FFIType::Pointer(inner) => Type::Pointer(Box::new(ffi_to_neksis_type(inner))),
        _ => Type::Void, // Default fallback
    }
} 