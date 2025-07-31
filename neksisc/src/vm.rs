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
            VMValue::Object(_) => "<object>".to_string(),
            VMValue::Array(_) => "<array>".to_string(),
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
                        if let Some((start, _end, param_count)) = self.function_table.get(&name) {
                            if arg_count != *param_count {
                                return Err(format!("Function {} expects {} arguments, got {}", name, param_count, arg_count));
                            }
                            // Save return address
                            let return_ip = self.instruction_pointer + 1;
                            self.call_stack.push(return_ip);
                            // Jump to function
                            self.instruction_pointer = *start;
                            continue;
                        } else {
                            // Try built-in function
                            self.call_builtin_function(&name, arg_count)?;
                        }
                    }
                }
                BytecodeInstruction::Return => {
                    if let Some(return_ip) = self.call_stack.pop() {
                        self.instruction_pointer = return_ip;
                        continue;
                    } else {
                        return Err("Return without function call".to_string());
                    }
                }
                BytecodeInstruction::EndFunction => {
                    // End of function definition
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
            _ => {
                return Err(format!("Unknown built-in function: {}", name));
            }
        }
        Ok(())
    }

    fn call_user_function(&mut self, name: &str, arg_count: usize) -> Result<(), String> {
        if let Some((start, _end, param_count)) = self.function_table.get(name) {
            if arg_count != *param_count {
                return Err(format!("Function {} expects {} arguments, got {}", name, param_count, arg_count));
            }
            // Save return address
            self.call_stack.push(self.instruction_pointer + 1);
            // Jump to function
            self.instruction_pointer = *start;
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
} 