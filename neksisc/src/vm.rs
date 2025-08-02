use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum VMValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    Function(String, Vec<VMValue>),
    BuiltinFunction(String),
    Object(HashMap<String, VMValue>),
    Array(Vec<VMValue>),
}

impl VMValue {
    pub fn to_string(&self) -> String {
        match self {
            VMValue::Int(i) => i.to_string(),
            VMValue::Float(f) => f.to_string(),
            VMValue::String(s) => s.clone(),
            VMValue::Bool(b) => b.to_string(),
            VMValue::Null => "null".to_string(),
            VMValue::Function(name, _) => format!("<function {}>", name),
            VMValue::BuiltinFunction(name) => format!("<builtin {}>", name),
            VMValue::Object(map) => {
                let entries: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            },
            VMValue::Array(arr) => {
                let elements: Vec<String> = arr.iter()
                    .map(|v| v.to_string())
                    .collect();
                format!("[{}]", elements.join(", "))
            },
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            VMValue::Int(i) => *i != 0,
            VMValue::Float(f) => *f != 0.0,
            VMValue::String(s) => !s.is_empty(),
            VMValue::Bool(b) => *b,
            VMValue::Null => false,
            VMValue::Function(_, _) => true,
            VMValue::BuiltinFunction(_) => true,
            VMValue::Object(_) => true,
            VMValue::Array(_) => true,
        }
    }
}

impl PartialOrd for VMValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (VMValue::Int(a), VMValue::Int(b)) => a.partial_cmp(b),
            (VMValue::Float(a), VMValue::Float(b)) => a.partial_cmp(b),
            (VMValue::String(a), VMValue::String(b)) => a.partial_cmp(b),
            (VMValue::Bool(a), VMValue::Bool(b)) => a.partial_cmp(b),
            (VMValue::Int(a), VMValue::Float(b)) => (*a as f64).partial_cmp(b),
            (VMValue::Float(a), VMValue::Int(b)) => a.partial_cmp(&(*b as f64)),
            _ => None,
        }
    }
}

impl PartialEq for VMValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VMValue::Int(a), VMValue::Int(b)) => a == b,
            (VMValue::Float(a), VMValue::Float(b)) => a == b,
            (VMValue::String(a), VMValue::String(b)) => a == b,
            (VMValue::Bool(a), VMValue::Bool(b)) => a == b,
            (VMValue::Null, VMValue::Null) => true,
            (VMValue::Int(a), VMValue::Float(b)) => (*a as f64) == *b,
            (VMValue::Float(a), VMValue::Int(b)) => *a == (*b as f64),
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum BytecodeInstruction {
    // Stack operations
    PushInt(i64),
    PushFloat(f64),
    PushString(String),
    PushBool(bool),
    PushNull,
    Pop,
    Dup,
    Swap,
    
    // Variable operations
    Load(String),
    Store(String),
    LoadGlobal(String),
    StoreGlobal(String),
    
    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    
    // Comparison operations
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    
    // Logical operations
    And,
    Or,
    Not,
    
    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    
    // Function operations
    Call(String, usize),
    Return,
    DefineFunction(String, usize),
    EndFunction,
    
    // Built-in functions
    Print,
    Println,
    ReadLine,
    ReadFile,
    WriteFile,
    AppendFile,
    FileExists,
    
    // Math functions
    Abs,
    Sqrt,
    Sin,
    Cos,
    Tan,
    Floor,
    Ceil,
    Round,
    Pow,
    Min,
    Max,
    
    // Error handling
    Throw(String),
    Try(usize),
    Catch,
    Finally,
    
    // Object operations
    NewObject,
    GetProperty(String),
    SetProperty(String),
    CallMethod(String, usize),
    
    // Array operations
    NewArray,
    GetIndex,
    SetIndex,
    
    // Type operations
    ToString,
    ToInt,
    ToFloat,
    ToBool,
    
    // String operations
    StringLen,
    Substring,
    StringConcat,
    StringContains,
    StringStartsWith,
    StringEndsWith,
    StringToUpper,
    StringToLower,
    StringTrim,
    StringSplit,
    StringJoin,
    
    // Utility operations
    Random,
    RandomInt,
    TypeOf,
    Time,
    Sleep,
    Exit,
    
    // Advanced Data Structures - HashMap/Dictionary
    DictNew,
    DictSet,
    DictGet,
    DictHas,
    DictKeys,
    DictSize,
    DictRemove,
    DictClear,
    
    // Advanced Array functions
    ArrayPush,
    ArrayPop,
    ArrayReverse,
    ArraySort,
    ArrayFilter,
    ArrayMap,
    ArrayReduce,
    ArrayFind,
    ArraySlice,
    
    // JSON Support
    JsonParse,
    JsonStringify,
    
    // Error Handling
    TryCatch,
    ThrowError,
}

pub struct VM {
    stack: Vec<VMValue>,
    locals: HashMap<String, VMValue>,
    globals: HashMap<String, VMValue>,
    instructions: Vec<BytecodeInstruction>,
    instruction_pointer: usize,
    call_stack: Vec<usize>,
    function_table: HashMap<String, (usize, usize, usize)>,
    error: Option<String>,
    in_function_definition: bool,
    scope_stack: Vec<HashMap<String, VMValue>>, // Stack of local variable scopes
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            locals: HashMap::new(),
            globals: HashMap::new(),
            instructions: Vec::new(),
            instruction_pointer: 0,
            call_stack: Vec::new(),
            function_table: HashMap::new(),
            error: None,
            in_function_definition: false,
            scope_stack: Vec::new(),
        }
    }

    pub fn load_instructions(&mut self, instructions: Vec<BytecodeInstruction>) {
        self.instructions = instructions;
        self.instruction_pointer = 0;
        self.build_function_table();
    }

    fn build_function_table(&mut self) {
        self.function_table.clear();
        let mut current_function: Option<String> = None;
        let mut start_ip = 0;
        let mut param_count = 0;

        for (ip, instruction) in self.instructions.iter().enumerate() {
            match instruction {
                BytecodeInstruction::DefineFunction(name, params) => {
                    if let Some(func_name) = &current_function {
                        self.function_table.insert(func_name.clone(), (start_ip, ip, param_count));
                    }
                    current_function = Some(name.clone());
                    start_ip = ip + 1;
                    param_count = *params;
                }
                BytecodeInstruction::EndFunction => {
                    if let Some(func_name) = &current_function {
                        self.function_table.insert(func_name.clone(), (start_ip, ip, param_count));
                        current_function = None;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.instruction_pointer < self.instructions.len() {
            let instruction = self.instructions[self.instruction_pointer].clone();
            
            // Skip instructions that are part of function definitions
            if self.in_function_definition && !matches!(instruction, 
                BytecodeInstruction::DefineFunction(_, _) | 
                BytecodeInstruction::EndFunction) {
                self.instruction_pointer += 1;
                continue;
            }
            
            match instruction {
                BytecodeInstruction::PushInt(value) => {
                    self.stack.push(VMValue::Int(value));
                }
                BytecodeInstruction::PushFloat(value) => {
                    self.stack.push(VMValue::Float(value));
                }
                BytecodeInstruction::PushString(value) => {
                    self.stack.push(VMValue::String(value));
                }
                BytecodeInstruction::PushBool(value) => {
                    self.stack.push(VMValue::Bool(value));
                }
                BytecodeInstruction::PushNull => {
                    self.stack.push(VMValue::Null);
                }
                BytecodeInstruction::Load(name) => {
                    if let Some(value) = self.locals.get(&name)
                        .or_else(|| self.globals.get(&name)) {
                        self.stack.push(value.clone());
                    } else if name == "print" || name == "println" || name == "read_line" {
                        self.stack.push(VMValue::BuiltinFunction(name));
                    } else {
                        return Err(format!("Undefined variable: {}", name));
                    }
                }
                BytecodeInstruction::Store(name) => {
                    if let Some(value) = self.stack.pop() {
                        self.locals.insert(name, value);
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::LoadGlobal(name) => {
                    let value = self.globals.get(&name)
                        .ok_or(format!("Global variable '{}' not found", name))?
                        .clone();
                    self.stack.push(value);
                }
                BytecodeInstruction::StoreGlobal(name) => {
                    if let Some(value) = self.stack.pop() {
                        self.globals.insert(name, value);
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Add => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        match (left, right) {
                            (VMValue::Int(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Int(a + b));
                            }
                            (VMValue::Float(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float(a + b));
                            }
                            (VMValue::Int(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float(a as f64 + b));
                            }
                            (VMValue::Float(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Float(a + b as f64));
                            }
                            (VMValue::String(a), VMValue::String(b)) => {
                                self.stack.push(VMValue::String(a + &b));
                            }
                            (VMValue::String(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::String(a + &b.to_string()));
                            }
                            (VMValue::Int(a), VMValue::String(b)) => {
                                self.stack.push(VMValue::String(a.to_string() + &b));
                            }
                            (VMValue::String(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::String(a + &b.to_string()));
                            }
                            (VMValue::Float(a), VMValue::String(b)) => {
                                self.stack.push(VMValue::String(a.to_string() + &b));
                            }
                            _ => {
                                return Err("Cannot perform arithmetic on non-numeric value".to_string());
                            }
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Sub => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        match (left, right) {
                            (VMValue::Int(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Int(a - b));
                            }
                            (VMValue::Float(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float(a - b));
                            }
                            (VMValue::Int(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float(a as f64 - b));
                            }
                            (VMValue::Float(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Float(a - b as f64));
                            }
                            _ => {
                                return Err("Cannot perform arithmetic on non-numeric value".to_string());
                            }
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Mul => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        match (left, right) {
                            (VMValue::Int(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Int(a * b));
                            }
                            (VMValue::Float(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float(a * b));
                            }
                            (VMValue::Int(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float(a as f64 * b));
                            }
                            (VMValue::Float(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Float(a * b as f64));
                            }
                            _ => {
                                return Err("Cannot perform arithmetic on non-numeric value".to_string());
                            }
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Div => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        match (left, right) {
                            (VMValue::Int(a), VMValue::Int(b)) => {
                                if b == 0 {
                                    return Err("Division by zero".to_string());
                                }
                                self.stack.push(VMValue::Int(a / b));
                            }
                            (VMValue::Float(a), VMValue::Float(b)) => {
                                if b == 0.0 {
                                    return Err("Division by zero".to_string());
                                }
                                self.stack.push(VMValue::Float(a / b));
                            }
                            (VMValue::Int(a), VMValue::Float(b)) => {
                                if b == 0.0 {
                                    return Err("Division by zero".to_string());
                                }
                                self.stack.push(VMValue::Float(a as f64 / b));
                            }
                            (VMValue::Float(a), VMValue::Int(b)) => {
                                if b == 0 {
                                    return Err("Division by zero".to_string());
                                }
                                self.stack.push(VMValue::Float(a / b as f64));
                            }
                            _ => {
                                return Err("Cannot perform arithmetic on non-numeric value".to_string());
                            }
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Mod => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        match (left, right) {
                            (VMValue::Int(a), VMValue::Int(b)) => {
                                if b == 0 {
                                    return Err("Division by zero".to_string());
                                }
                                self.stack.push(VMValue::Int(a % b));
                            }
                            (VMValue::Float(a), VMValue::Float(b)) => {
                                if b == 0.0 {
                                    return Err("Division by zero".to_string());
                                }
                                self.stack.push(VMValue::Float((a as f64) % b));
                            }
                            (VMValue::Int(a), VMValue::Float(b)) => {
                                if b == 0.0 {
                                    return Err("Division by zero".to_string());
                                }
                                self.stack.push(VMValue::Float((a as f64) % b));
                            }
                            (VMValue::Float(a), VMValue::Int(b)) => {
                                if b == 0 {
                                    return Err("Division by zero".to_string());
                                }
                                self.stack.push(VMValue::Float(a % (b as f64)));
                            }
                            _ => {
                                return Err("Cannot perform arithmetic on non-numeric value".to_string());
                            }
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Neg => {
                    if let Some(value) = self.stack.pop() {
                        let result = match value {
                            VMValue::Int(i) => VMValue::Int(-i),
                            VMValue::Float(f) => VMValue::Float(-f),
                            _ => return Err("Cannot negate non-numeric value".to_string()),
                        };
                        self.stack.push(result);
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Eq => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(VMValue::Bool(left == right));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Ne => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(VMValue::Bool(left != right));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Lt => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        if let Some(ordering) = left.partial_cmp(&right) {
                            self.stack.push(VMValue::Bool(ordering == std::cmp::Ordering::Less));
                        } else {
                            return Err("Cannot compare values".to_string());
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Le => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        if let Some(ordering) = left.partial_cmp(&right) {
                            self.stack.push(VMValue::Bool(ordering == std::cmp::Ordering::Less || ordering == std::cmp::Ordering::Equal));
                        } else {
                            return Err("Cannot compare values".to_string());
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Gt => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        if let Some(ordering) = left.partial_cmp(&right) {
                            self.stack.push(VMValue::Bool(ordering == std::cmp::Ordering::Greater));
                        } else {
                            return Err("Cannot compare values".to_string());
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Ge => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        if let Some(ordering) = left.partial_cmp(&right) {
                            self.stack.push(VMValue::Bool(ordering == std::cmp::Ordering::Greater || ordering == std::cmp::Ordering::Equal));
                        } else {
                            return Err("Cannot compare values".to_string());
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::And => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(VMValue::Bool(left.to_bool() && right.to_bool()));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Or => {
                    if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(VMValue::Bool(left.to_bool() || right.to_bool()));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Not => {
                    if let Some(value) = self.stack.pop() {
                        self.stack.push(VMValue::Bool(!value.to_bool()));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Jump(offset) => {
                    self.instruction_pointer = offset;
                    continue;
                }
                BytecodeInstruction::JumpIfFalse(offset) => {
                    if let Some(condition) = self.stack.pop() {
                        if !condition.to_bool() {
                            self.instruction_pointer = offset;
                            continue;
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::JumpIfTrue(offset) => {
                    if let Some(condition) = self.stack.pop() {
                        if condition.to_bool() {
                            self.instruction_pointer = offset;
                            continue;
                        }
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Call(name, arg_count) => {
                    if name.is_empty() {
                        // Dynamic function call
                        if let Some(VMValue::BuiltinFunction(func_name)) = self.stack.pop() {
                            self.call_builtin_function(&func_name, arg_count)?;
                        } else if let Some(VMValue::Function(func_name, _)) = self.stack.pop() {
                            self.call_user_function(&func_name, arg_count)?;
                        } else {
                            return Err("Invalid function call".to_string());
                        }
                    } else {
                        // Direct function call
                        if let Some((start, _end, param_count)) = self.function_table.get(&name).cloned() {
                            if arg_count != param_count {
                                return Err(format!("Function {} expects {} arguments, got {}", name, param_count, arg_count));
                            }
                            
                            // Create new scope for function
                            self.push_scope();
                            
                            // Save return address
                            let return_ip = self.instruction_pointer + 1;
                            self.call_stack.push(return_ip);
                            // Jump to function (parameters will be handled by Store instructions in the function)
                            self.instruction_pointer = start;
                            continue;
                        } else {
                            // Try built-in function
                            self.call_builtin_function(&name, arg_count)?;
                        }
                    }
                }
                BytecodeInstruction::Return => {
                    if let Some(return_ip) = self.call_stack.pop() {
                        // Pop the function scope
                        self.pop_scope();
                        self.instruction_pointer = return_ip;
                        continue;
                    } else {
                        return Err("Return without function call".to_string());
                    }
                }
                BytecodeInstruction::EndFunction => {
                    // End of function definition
                    self.in_function_definition = false;
                }
                BytecodeInstruction::Print => {
                    if let Some(value) = self.stack.pop() {
                        print!("{}", value.to_string());
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Println => {
                    if let Some(value) = self.stack.pop() {
                        println!("{}", value.to_string());
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::ReadLine => {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
                    self.stack.push(VMValue::String(input.trim().to_string()));
                }
                BytecodeInstruction::ReadFile => {
                    if let Some(VMValue::String(path)) = self.stack.pop() {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => self.stack.push(VMValue::String(content)),
                            Err(e) => return Err(format!("Failed to read file '{}': {}", path, e)),
                        }
                    } else {
                        return Err("read_file expects a string path".to_string());
                    }
                }
                BytecodeInstruction::WriteFile => {
                    if let (Some(VMValue::String(content)), Some(VMValue::String(path))) = (self.stack.pop(), self.stack.pop()) {
                        match std::fs::write(&path, &content) {
                            Ok(_) => self.stack.push(VMValue::Null),
                            Err(e) => return Err(format!("Failed to write file '{}': {}", path, e)),
                        }
                    } else {
                        return Err("write_file expects string arguments".to_string());
                    }
                }
                BytecodeInstruction::AppendFile => {
                    if let (Some(VMValue::String(content)), Some(VMValue::String(path))) = (self.stack.pop(), self.stack.pop()) {
                        use std::fs::OpenOptions;
                        use std::io::Write;
                        
                        match OpenOptions::new().create(true).append(true).open(&path) {
                            Ok(mut file) => {
                                match file.write_all(content.as_bytes()) {
                                    Ok(_) => self.stack.push(VMValue::Null),
                                    Err(e) => return Err(format!("Failed to append to file '{}': {}", path, e)),
                                }
                            },
                            Err(e) => return Err(format!("Failed to open file '{}' for appending: {}", path, e)),
                        }
                    } else {
                        return Err("append_file expects string arguments".to_string());
                    }
                }
                BytecodeInstruction::FileExists => {
                    if let Some(VMValue::String(path)) = self.stack.pop() {
                        let exists = std::path::Path::new(&path).exists();
                        self.stack.push(VMValue::Bool(exists));
                    } else {
                        return Err("file_exists expects a string path".to_string());
                    }
                }
                BytecodeInstruction::Abs => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            VMValue::Int(n) => self.stack.push(VMValue::Int(n.abs())),
                            VMValue::Float(n) => self.stack.push(VMValue::Float(n.abs())),
                            _ => return Err("abs expects a numeric value".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for abs".to_string());
                    }
                }
                BytecodeInstruction::Sqrt => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            VMValue::Float(n) => {
                                if n < 0.0 {
                                    return Err("sqrt of negative number".to_string());
                                }
                                self.stack.push(VMValue::Float(n.sqrt()));
                            },
                            VMValue::Int(n) => {
                                if n < 0 {
                                    return Err("sqrt of negative number".to_string());
                                }
                                self.stack.push(VMValue::Float((n as f64).sqrt()));
                            },
                            _ => return Err("sqrt expects a numeric value".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for sqrt".to_string());
                    }
                }
                BytecodeInstruction::Sin => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            VMValue::Float(n) => self.stack.push(VMValue::Float(n.sin())),
                            VMValue::Int(n) => self.stack.push(VMValue::Float((n as f64).sin())),
                            _ => return Err("sin expects a numeric value".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for sin".to_string());
                    }
                }
                BytecodeInstruction::Cos => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            VMValue::Float(n) => self.stack.push(VMValue::Float(n.cos())),
                            VMValue::Int(n) => self.stack.push(VMValue::Float((n as f64).cos())),
                            _ => return Err("cos expects a numeric value".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for cos".to_string());
                    }
                }
                BytecodeInstruction::Tan => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            VMValue::Float(n) => self.stack.push(VMValue::Float(n.tan())),
                            VMValue::Int(n) => self.stack.push(VMValue::Float((n as f64).tan())),
                            _ => return Err("tan expects a numeric value".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for tan".to_string());
                    }
                }
                BytecodeInstruction::Floor => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            VMValue::Float(n) => self.stack.push(VMValue::Int(n.floor() as i64)),
                            VMValue::Int(n) => self.stack.push(VMValue::Int(n)), // Already an integer
                            _ => return Err("floor expects a numeric value".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for floor".to_string());
                    }
                }
                BytecodeInstruction::Ceil => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            VMValue::Float(n) => self.stack.push(VMValue::Int(n.ceil() as i64)),
                            VMValue::Int(n) => self.stack.push(VMValue::Int(n)), // Already an integer
                            _ => return Err("ceil expects a numeric value".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for ceil".to_string());
                    }
                }
                BytecodeInstruction::Round => {
                    if let Some(value) = self.stack.pop() {
                        match value {
                            VMValue::Float(n) => self.stack.push(VMValue::Int(n.round() as i64)),
                            VMValue::Int(n) => self.stack.push(VMValue::Int(n)), // Already an integer
                            _ => return Err("round expects a numeric value".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for round".to_string());
                    }
                }
                BytecodeInstruction::Pow => {
                    if let (Some(exp), Some(base)) = (self.stack.pop(), self.stack.pop()) {
                        match (base, exp) {
                            (VMValue::Float(base), VMValue::Float(exp)) => {
                                self.stack.push(VMValue::Float(base.powf(exp)));
                            },
                            (VMValue::Float(base), VMValue::Int(exp)) => {
                                self.stack.push(VMValue::Float(base.powf(exp as f64)));
                            },
                            (VMValue::Int(base), VMValue::Float(exp)) => {
                                self.stack.push(VMValue::Float((base as f64).powf(exp)));
                            },
                            (VMValue::Int(base), VMValue::Int(exp)) => {
                                if exp >= 0 {
                                    self.stack.push(VMValue::Int(base.pow(exp as u32)));
                                } else {
                                    self.stack.push(VMValue::Float((base as f64).powf(exp as f64)));
                                }
                            },
                            _ => return Err("pow expects numeric values".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for pow".to_string());
                    }
                }
                BytecodeInstruction::Min => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        match (a, b) {
                            (VMValue::Float(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float(a.min(b)));
                            },
                            (VMValue::Float(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Float(a.min(b as f64)));
                            },
                            (VMValue::Int(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float((a as f64).min(b)));
                            },
                            (VMValue::Int(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Int(a.min(b)));
                            },
                            _ => return Err("min expects numeric values".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for min".to_string());
                    }
                }
                BytecodeInstruction::Max => {
                    if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
                        match (a, b) {
                            (VMValue::Float(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float(a.max(b)));
                            },
                            (VMValue::Float(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Float(a.max(b as f64)));
                            },
                            (VMValue::Int(a), VMValue::Float(b)) => {
                                self.stack.push(VMValue::Float((a as f64).max(b)));
                            },
                            (VMValue::Int(a), VMValue::Int(b)) => {
                                self.stack.push(VMValue::Int(a.max(b)));
                            },
                            _ => return Err("max expects numeric values".to_string()),
                        }
                    } else {
                        return Err("Stack underflow for max".to_string());
                    }
                }
                BytecodeInstruction::Pop => {
                    self.stack.pop();
                }
                BytecodeInstruction::Dup => {
                    if let Some(value) = self.stack.last() {
                        self.stack.push(value.clone());
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::Swap => {
                    if self.stack.len() < 2 {
                        return Err("Stack underflow for swap".to_string());
                    }
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
                BytecodeInstruction::DefineFunction(_name, _param_count) => {
                    // Function definition is handled during compilation
                    // Skip to the next instruction
                    self.in_function_definition = true;
                }
                BytecodeInstruction::Throw(message) => {
                    self.error = Some(message);
                    return Ok(());
                }
                BytecodeInstruction::Try(_offset) => {
                    // Try block - just continue execution
                }
                BytecodeInstruction::Catch => {
                    // Catch block - clear error and continue
                    self.error = None;
                }
                BytecodeInstruction::Finally => {
                    // Finally block - always execute
                }
                BytecodeInstruction::NewObject => {
                    self.stack.push(VMValue::Object(HashMap::new()));
                }
                BytecodeInstruction::GetProperty(name) => {
                    if let Some(VMValue::Object(mut props)) = self.stack.pop() {
                        let value = props.remove(&name)
                            .unwrap_or(VMValue::Null);
                        self.stack.push(value);
                    } else {
                        return Err("Cannot get property from non-object".to_string());
                    }
                }
                BytecodeInstruction::SetProperty(name) => {
                    if let (Some(value), Some(VMValue::Object(mut props))) = (self.stack.pop(), self.stack.pop()) {
                        props.insert(name, value);
                        self.stack.push(VMValue::Object(props));
                    } else {
                        return Err("Cannot set property on non-object".to_string());
                    }
                }
                BytecodeInstruction::CallMethod(name, arg_count) => {
                    // Similar to Call but for object methods
                    self.call_function(&name, arg_count)?;
                }
                BytecodeInstruction::NewArray => {
                    self.stack.push(VMValue::Array(Vec::new()));
                }
                BytecodeInstruction::GetIndex => {
                    if let (Some(index), Some(VMValue::Array(mut arr))) = (self.stack.pop(), self.stack.pop()) {
                        if let VMValue::Int(i) = index {
                            if i >= 0 && i < arr.len() as i64 {
                                self.stack.push(arr.remove(i as usize));
                            } else {
                                self.stack.push(VMValue::Null);
                            }
                        } else {
                            return Err("Invalid array index".to_string());
                        }
                    } else {
                        return Err("Invalid array access".to_string());
                    }
                }
                BytecodeInstruction::SetIndex => {
                    if let (Some(value), Some(index), Some(VMValue::Array(mut arr))) = (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
                        if let VMValue::Int(i) = index {
                            if i >= 0 && i < arr.len() as i64 {
                                arr[i as usize] = value;
                            }
                            self.stack.push(VMValue::Array(arr));
                        } else {
                            return Err("Invalid array index".to_string());
                        }
                    } else {
                        return Err("Invalid array assignment".to_string());
                    }
                }
                BytecodeInstruction::ToString => {
                    if let Some(value) = self.stack.pop() {
                        self.stack.push(VMValue::String(value.to_string()));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::ToInt => {
                    if let Some(value) = self.stack.pop() {
                        let int_value = match value {
                            VMValue::Int(i) => i,
                            VMValue::Float(f) => f as i64,
                            VMValue::String(s) => s.parse().unwrap_or(0),
                            VMValue::Bool(b) => if b { 1 } else { 0 },
                            _ => 0,
                        };
                        self.stack.push(VMValue::Int(int_value));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::ToFloat => {
                    if let Some(value) = self.stack.pop() {
                        let float_value = match value {
                            VMValue::Int(i) => i as f64,
                            VMValue::Float(f) => f,
                            VMValue::String(s) => s.parse().unwrap_or(0.0),
                            VMValue::Bool(b) => if b { 1.0 } else { 0.0 },
                            _ => 0.0,
                        };
                        self.stack.push(VMValue::Float(float_value));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::ToBool => {
                    if let Some(value) = self.stack.pop() {
                        self.stack.push(VMValue::Bool(value.to_bool()));
                    } else {
                        return Err("Stack underflow".to_string());
                    }
                }
                BytecodeInstruction::StringLen => {
                    if let Some(VMValue::String(s)) = self.stack.pop() {
                        self.stack.push(VMValue::Int(s.len() as i64));
                    } else {
                        return Err("len expects a string".to_string());
                    }
                }
                BytecodeInstruction::Substring => {
                    if let (Some(VMValue::Int(len)), Some(VMValue::Int(start)), Some(VMValue::String(s))) = 
                        (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
                        let start_idx = start as usize;
                        let end_idx = start_idx + (len as usize);
                        
                        if start_idx >= s.len() || end_idx > s.len() {
                            return Err("substring indices out of bounds".to_string());
                        }
                        
                        self.stack.push(VMValue::String(s[start_idx..end_idx].to_string()));
                    } else {
                        return Err("substring expects string, int, int".to_string());
                    }
                }
                BytecodeInstruction::StringConcat => {
                    if let (Some(VMValue::String(s2)), Some(VMValue::String(s1))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(VMValue::String(format!("{}{}", s1, s2)));
                    } else {
                        return Err("concat expects string arguments".to_string());
                    }
                }
                BytecodeInstruction::StringContains => {
                    if let (Some(VMValue::String(substr)), Some(VMValue::String(s))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(VMValue::Bool(s.contains(&substr)));
                    } else {
                        return Err("contains expects string arguments".to_string());
                    }
                }
                BytecodeInstruction::StringStartsWith => {
                    if let (Some(VMValue::String(prefix)), Some(VMValue::String(s))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(VMValue::Bool(s.starts_with(&prefix)));
                    } else {
                        return Err("starts_with expects string arguments".to_string());
                    }
                }
                BytecodeInstruction::StringEndsWith => {
                    if let (Some(VMValue::String(suffix)), Some(VMValue::String(s))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(VMValue::Bool(s.ends_with(&suffix)));
                    } else {
                        return Err("ends_with expects string arguments".to_string());
                    }
                }
                BytecodeInstruction::StringToUpper => {
                    if let Some(VMValue::String(s)) = self.stack.pop() {
                        self.stack.push(VMValue::String(s.to_uppercase()));
                    } else {
                        return Err("to_upper expects a string".to_string());
                    }
                }
                BytecodeInstruction::StringToLower => {
                    if let Some(VMValue::String(s)) = self.stack.pop() {
                        self.stack.push(VMValue::String(s.to_lowercase()));
                    } else {
                        return Err("to_lower expects a string".to_string());
                    }
                }
                BytecodeInstruction::StringTrim => {
                    if let Some(VMValue::String(s)) = self.stack.pop() {
                        self.stack.push(VMValue::String(s.trim().to_string()));
                    } else {
                        return Err("trim expects a string".to_string());
                    }
                }
                BytecodeInstruction::StringSplit => {
                    if let (Some(VMValue::String(delimiter)), Some(VMValue::String(s))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        let parts: Vec<VMValue> = s.split(&delimiter)
                            .map(|part| VMValue::String(part.to_string()))
                            .collect();
                        self.stack.push(VMValue::Array(parts));
                    } else {
                        return Err("split expects string arguments".to_string());
                    }
                }
                BytecodeInstruction::StringJoin => {
                    if let (Some(VMValue::String(delimiter)), Some(VMValue::Array(arr))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        let strings: Result<Vec<String>, String> = arr.iter().map(|item| {
                            match item {
                                VMValue::String(s) => Ok(s.clone()),
                                VMValue::Int(i) => Ok(i.to_string()),
                                VMValue::Float(f) => Ok(f.to_string()),
                                VMValue::Bool(b) => Ok(b.to_string()),
                                VMValue::Null => Ok("null".to_string()),
                                _ => Err("join can only handle primitive types".to_string()),
                            }
                        }).collect();
                        
                        match strings {
                            Ok(string_vec) => self.stack.push(VMValue::String(string_vec.join(&delimiter))),
                            Err(e) => return Err(e),
                        }
                    } else {
                        return Err("join expects array and string arguments".to_string());
                    }
                }
                BytecodeInstruction::Random => {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    use std::time::{SystemTime, UNIX_EPOCH};
                    
                    let mut hasher = DefaultHasher::new();
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
                    let hash = hasher.finish();
                    
                    let random_float = (hash as f64) / (u64::MAX as f64);
                    self.stack.push(VMValue::Float(random_float));
                }
                BytecodeInstruction::RandomInt => {
                    if let (Some(VMValue::Int(max)), Some(VMValue::Int(min))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        if min >= max {
                            return Err("random_int: min must be less than max".to_string());
                        }
                        
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        use std::time::{SystemTime, UNIX_EPOCH};
                        
                        let mut hasher = DefaultHasher::new();
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
                        let hash = hasher.finish();
                        
                        let range = (max - min) as u64;
                        let random_int = min + ((hash % range) as i64);
                        self.stack.push(VMValue::Int(random_int));
                    } else {
                        return Err("random_int expects integer arguments".to_string());
                    }
                }
                BytecodeInstruction::TypeOf => {
                    if let Some(value) = self.stack.pop() {
                        let type_name = match value {
                            VMValue::Int(_) => "int",
                            VMValue::Float(_) => "float",
                            VMValue::String(_) => "string",
                            VMValue::Bool(_) => "bool",
                            VMValue::Array(_) => "array",
                            VMValue::Null => "null",
                            VMValue::Function(_, _) => "function",
                            VMValue::BuiltinFunction(_) => "builtin_function",
                            VMValue::Object(_) => "object",
                        };
                        self.stack.push(VMValue::String(type_name.to_string()));
                    } else {
                        return Err("Stack underflow for typeof".to_string());
                    }
                }
                BytecodeInstruction::Time => {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    
                    self.stack.push(VMValue::Int(timestamp));
                }
                BytecodeInstruction::Sleep => {
                    if let Some(value) = self.stack.pop() {
                        let duration = match value {
                            VMValue::Int(seconds) => seconds as u64,
                            VMValue::Float(seconds) => seconds as u64,
                            _ => return Err("sleep expects a numeric argument".to_string()),
                        };
                        
                        std::thread::sleep(std::time::Duration::from_secs(duration));
                        self.stack.push(VMValue::Null);
                    } else {
                        return Err("Stack underflow for sleep".to_string());
                    }
                }
                BytecodeInstruction::Exit => {
                    let exit_code = if let Some(VMValue::Int(code)) = self.stack.pop() {
                        code as i32
                    } else {
                        0
                    };
                    
                    std::process::exit(exit_code);
                }
                
                // Advanced Data Structures - HashMap/Dictionary
                BytecodeInstruction::DictNew => {
                    let dict = HashMap::new();
                    self.stack.push(VMValue::Object(dict));
                }
                BytecodeInstruction::DictSet => {
                    if let (Some(value), Some(key), Some(VMValue::Object(mut dict))) = 
                        (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
                        let key_str = key.to_string();
                        dict.insert(key_str, value);
                        self.stack.push(VMValue::Object(dict));
                    } else {
                        return Err("Invalid arguments for dict_set".to_string());
                    }
                }
                BytecodeInstruction::DictGet => {
                    if let (Some(key), Some(VMValue::Object(dict))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        let key_str = key.to_string();
                        let value = dict.get(&key_str).cloned().unwrap_or(VMValue::Null);
                        self.stack.push(value);
                    } else {
                        return Err("Invalid arguments for dict_get".to_string());
                    }
                }
                BytecodeInstruction::DictHas => {
                    if let (Some(key), Some(VMValue::Object(dict))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        let key_str = key.to_string();
                        let has_key = dict.contains_key(&key_str);
                        self.stack.push(VMValue::Bool(has_key));
                    } else {
                        return Err("Invalid arguments for dict_has".to_string());
                    }
                }
                BytecodeInstruction::DictKeys => {
                    if let Some(VMValue::Object(dict)) = self.stack.pop() {
                        let keys: Vec<VMValue> = dict.keys()
                            .map(|k| VMValue::String(k.clone()))
                            .collect();
                        self.stack.push(VMValue::Array(keys));
                    } else {
                        return Err("Invalid argument for dict_keys".to_string());
                    }
                }
                BytecodeInstruction::DictSize => {
                    if let Some(VMValue::Object(dict)) = self.stack.pop() {
                        self.stack.push(VMValue::Int(dict.len() as i64));
                    } else {
                        return Err("Invalid argument for dict_size".to_string());
                    }
                }
                BytecodeInstruction::DictRemove => {
                    if let (Some(key), Some(VMValue::Object(mut dict))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        let key_str = key.to_string();
                        let removed = dict.remove(&key_str).unwrap_or(VMValue::Null);
                        self.stack.push(VMValue::Object(dict));
                        self.stack.push(removed);
                    } else {
                        return Err("Invalid arguments for dict_remove".to_string());
                    }
                }
                BytecodeInstruction::DictClear => {
                    if let Some(VMValue::Object(mut dict)) = self.stack.pop() {
                        dict.clear();
                        self.stack.push(VMValue::Object(dict));
                    } else {
                        return Err("Invalid argument for dict_clear".to_string());
                    }
                }
                
                // Advanced Array functions
                BytecodeInstruction::ArrayPush => {
                    if let (Some(value), Some(VMValue::Array(mut arr))) = 
                        (self.stack.pop(), self.stack.pop()) {
                        arr.push(value);
                        self.stack.push(VMValue::Array(arr));
                    } else {
                        return Err("Invalid arguments for array_push".to_string());
                    }
                }
                BytecodeInstruction::ArrayPop => {
                    if let Some(VMValue::Array(mut arr)) = self.stack.pop() {
                        let popped = arr.pop().unwrap_or(VMValue::Null);
                        self.stack.push(VMValue::Array(arr));
                        self.stack.push(popped);
                    } else {
                        return Err("Invalid argument for array_pop".to_string());
                    }
                }
                BytecodeInstruction::ArrayReverse => {
                    if let Some(VMValue::Array(mut arr)) = self.stack.pop() {
                        arr.reverse();
                        self.stack.push(VMValue::Array(arr));
                    } else {
                        return Err("Invalid argument for array_reverse".to_string());
                    }
                }
                BytecodeInstruction::ArraySort => {
                    if let Some(VMValue::Array(mut arr)) = self.stack.pop() {
                        arr.sort_by(|a, b| {
                            match (a, b) {
                                (VMValue::Int(a), VMValue::Int(b)) => a.cmp(b),
                                (VMValue::Float(a), VMValue::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
                                (VMValue::String(a), VMValue::String(b)) => a.cmp(b),
                                _ => std::cmp::Ordering::Equal,
                            }
                        });
                        self.stack.push(VMValue::Array(arr));
                    } else {
                        return Err("Invalid argument for array_sort".to_string());
                    }
                }
                BytecodeInstruction::ArraySlice => {
                    if let (Some(end), Some(start), Some(VMValue::Array(arr))) = 
                        (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
                        if let (VMValue::Int(start), VMValue::Int(end)) = (start, end) {
                            let start_idx = start.max(0) as usize;
                            let end_idx = (end.max(0) as usize).min(arr.len());
                            if start_idx <= end_idx && start_idx < arr.len() {
                                let slice = arr[start_idx..end_idx].to_vec();
                                self.stack.push(VMValue::Array(slice));
                            } else {
                                self.stack.push(VMValue::Array(vec![]));
                            }
                        } else {
                            return Err("Array slice indices must be integers".to_string());
                        }
                    } else {
                        return Err("Invalid arguments for array_slice".to_string());
                    }
                }
                
                // Simplified versions of filter, map, reduce, find
                BytecodeInstruction::ArrayFilter => {
                    // For now, just return the original array
                    // TODO: Implement proper filtering with callback functions
                    if let Some(VMValue::Array(arr)) = self.stack.pop() {
                        self.stack.push(VMValue::Array(arr));
                    } else {
                        return Err("Invalid argument for array_filter".to_string());
                    }
                }
                BytecodeInstruction::ArrayMap => {
                    // For now, just return the original array
                    // TODO: Implement proper mapping with callback functions
                    if let Some(VMValue::Array(arr)) = self.stack.pop() {
                        self.stack.push(VMValue::Array(arr));
                    } else {
                        return Err("Invalid argument for array_map".to_string());
                    }
                }
                BytecodeInstruction::ArrayReduce => {
                    // For now, just return the first element or null
                    // TODO: Implement proper reduction with callback functions
                    if let Some(VMValue::Array(arr)) = self.stack.pop() {
                        let result = arr.first().cloned().unwrap_or(VMValue::Null);
                        self.stack.push(result);
                    } else {
                        return Err("Invalid argument for array_reduce".to_string());
                    }
                }
                BytecodeInstruction::ArrayFind => {
                    // For now, just return the first element or null
                    // TODO: Implement proper finding with callback functions
                    if let Some(VMValue::Array(arr)) = self.stack.pop() {
                        let result = arr.first().cloned().unwrap_or(VMValue::Null);
                        self.stack.push(result);
                    } else {
                        return Err("Invalid argument for array_find".to_string());
                    }
                }
                
                // JSON Support
                BytecodeInstruction::JsonParse => {
                    if let Some(VMValue::String(json_str)) = self.stack.pop() {
                        // Simple JSON parsing - for demo purposes
                        // TODO: Implement proper JSON parsing
                        if json_str.starts_with('{') && json_str.ends_with('}') {
                            let dict = HashMap::new();
                            self.stack.push(VMValue::Object(dict));
                        } else if json_str.starts_with('[') && json_str.ends_with(']') {
                            let arr = Vec::new();
                            self.stack.push(VMValue::Array(arr));
                        } else {
                            self.stack.push(VMValue::String(json_str));
                        }
                    } else {
                        return Err("Invalid argument for json_parse".to_string());
                    }
                }
                BytecodeInstruction::JsonStringify => {
                    if let Some(value) = self.stack.pop() {
                        let json_str = match value {
                            VMValue::Object(dict) => {
                                let entries: Vec<String> = dict.iter()
                                    .map(|(k, v)| format!("\"{}\":{}", k, match v {
                                        VMValue::String(s) => format!("\"{}\"", s),
                                        VMValue::Int(i) => i.to_string(),
                                        VMValue::Float(f) => f.to_string(),
                                        VMValue::Bool(b) => b.to_string(),
                                        VMValue::Null => "null".to_string(),
                                        _ => "null".to_string(),
                                    }))
                                    .collect();
                                format!("{{{}}}", entries.join(","))
                            }
                            VMValue::Array(arr) => {
                                let elements: Vec<String> = arr.iter()
                                    .map(|v| match v {
                                        VMValue::String(s) => format!("\"{}\"", s),
                                        VMValue::Int(i) => i.to_string(),
                                        VMValue::Float(f) => f.to_string(),
                                        VMValue::Bool(b) => b.to_string(),
                                        VMValue::Null => "null".to_string(),
                                        _ => "null".to_string(),
                                    })
                                    .collect();
                                format!("[{}]", elements.join(","))
                            }
                            VMValue::String(s) => format!("\"{}\"", s),
                            VMValue::Int(i) => i.to_string(),
                            VMValue::Float(f) => f.to_string(),
                            VMValue::Bool(b) => b.to_string(),
                            VMValue::Null => "null".to_string(),
                            _ => "null".to_string(),
                        };
                        self.stack.push(VMValue::String(json_str));
                    } else {
                        return Err("Stack underflow for json_stringify".to_string());
                    }
                }
                
                // Error Handling (simplified)
                BytecodeInstruction::TryCatch => {
                    // For now, just push null
                    // TODO: Implement proper try-catch mechanism
                    self.stack.push(VMValue::Null);
                }
                BytecodeInstruction::ThrowError => {
                    if let Some(VMValue::String(error_msg)) = self.stack.pop() {
                        return Err(format!("Thrown error: {}", error_msg));
                    } else {
                        return Err("Thrown error: Unknown error".to_string());
                    }
                }
            }
            
            self.instruction_pointer += 1;
        }
        
        Ok(())
    }

    fn call_builtin_function(&mut self, name: &str, arg_count: usize) -> Result<(), String> {
        match name {
            "print" => {
                if arg_count != 1 {
                    return Err("print expects 1 argument".to_string());
                }
                if let Some(value) = self.stack.pop() {
                    print!("{}", value.to_string());
                } else {
                    return Err("Stack underflow".to_string());
                }
            }
            "println" => {
                if arg_count != 1 {
                    return Err("println expects 1 argument".to_string());
                }
                if let Some(value) = self.stack.pop() {
                    println!("{}", value.to_string());
                } else {
                    return Err("Stack underflow".to_string());
                }
            }
            "read_line" => {
                if arg_count != 0 {
                    return Err("read_line expects 0 arguments".to_string());
                }
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
                self.stack.push(VMValue::String(input.trim().to_string()));
            }
            "read_file" => {
                if arg_count != 1 {
                    return Err("read_file expects 1 argument".to_string());
                }
                if let Some(VMValue::String(path)) = self.stack.pop() {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => self.stack.push(VMValue::String(content)),
                        Err(e) => return Err(format!("Failed to read file '{}': {}", path, e)),
                    }
                } else {
                    return Err("read_file expects a string path".to_string());
                }
            }
            "write_file" => {
                if arg_count != 2 {
                    return Err("write_file expects 2 arguments".to_string());
                }
                if let (Some(VMValue::String(content)), Some(VMValue::String(path))) = (self.stack.pop(), self.stack.pop()) {
                    match std::fs::write(&path, &content) {
                        Ok(_) => self.stack.push(VMValue::Null),
                        Err(e) => return Err(format!("Failed to write file '{}': {}", path, e)),
                    }
                } else {
                    return Err("write_file expects string arguments".to_string());
                }
            }
            "append_file" => {
                if arg_count != 2 {
                    return Err("append_file expects 2 arguments".to_string());
                }
                if let (Some(VMValue::String(content)), Some(VMValue::String(path))) = (self.stack.pop(), self.stack.pop()) {
                    use std::fs::OpenOptions;
                    use std::io::Write;
                    
                    match OpenOptions::new().create(true).append(true).open(&path) {
                        Ok(mut file) => {
                            match file.write_all(content.as_bytes()) {
                                Ok(_) => self.stack.push(VMValue::Null),
                                Err(e) => return Err(format!("Failed to append to file '{}': {}", path, e)),
                            }
                        },
                        Err(e) => return Err(format!("Failed to open file '{}' for appending: {}", path, e)),
                    }
                } else {
                    return Err("append_file expects string arguments".to_string());
                }
            }
            "file_exists" => {
                if arg_count != 1 {
                    return Err("file_exists expects 1 argument".to_string());
                }
                if let Some(VMValue::String(path)) = self.stack.pop() {
                    let exists = std::path::Path::new(&path).exists();
                    self.stack.push(VMValue::Bool(exists));
                } else {
                    return Err("file_exists expects a string path".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown built-in function: {}", name));
            }
        }
        Ok(())
    }

    fn call_user_function(&mut self, name: &str, arg_count: usize) -> Result<(), String> {
        if let Some((start, _end, param_count)) = self.function_table.get(name).cloned() {
            if arg_count != param_count {
                return Err(format!("Function {} expects {} arguments, got {}", name, param_count, arg_count));
            }
            
            // Create new scope for function
            self.push_scope();
            
            // Save return address
            self.call_stack.push(self.instruction_pointer + 1);
            // Jump to function (parameters will be handled by Store instructions)
            self.instruction_pointer = start;
        } else {
            return Err(format!("Undefined function: {}", name));
        }
        Ok(())
    }

    fn call_function(&mut self, name: &str, arg_count: usize) -> Result<(), String> {
        // This is a placeholder for method calls
        // For now, just call as a regular function
        self.call_user_function(name, arg_count)
    }
    
    fn push_scope(&mut self) {
        // Save current locals to scope stack
        self.scope_stack.push(self.locals.clone());
        // Clear current locals for new scope
        self.locals.clear();
    }
    
    fn pop_scope(&mut self) {
        // Restore previous scope
        if let Some(previous_locals) = self.scope_stack.pop() {
            self.locals = previous_locals;
        }
    }
} 