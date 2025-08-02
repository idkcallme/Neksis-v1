use crate::ast::*;
use crate::error::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction {
    // I/O functions
    Print,
    Println,
    ReadLine,
    
    // File I/O functions
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
    
    // String functions
    StringLen,
    Substring,
    Concat,
    Contains,
    StartsWith,
    EndsWith,
    ToUpper,
    ToLower,
    Trim,
    Split,
    Join,
    
    // Type conversion
    ToString,
    ToInt,
    ToFloat,
    
    // Utility functions  
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
    
    // Memory management
    Malloc,
    Free,
    
    // Error handling
    Panic,
    Assert,
}

impl BuiltinFunction {
    pub fn execute(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        match self {
            BuiltinFunction::Print => self.execute_print(args),
            BuiltinFunction::Println => self.execute_println(args),
            BuiltinFunction::ReadLine => self.execute_read_line(args),
            BuiltinFunction::ReadFile => self.execute_read_file(args),
            BuiltinFunction::WriteFile => self.execute_write_file(args),
            BuiltinFunction::AppendFile => self.execute_append_file(args),
            BuiltinFunction::FileExists => self.execute_file_exists(args),
            BuiltinFunction::Abs => self.execute_abs(args),
            BuiltinFunction::Sqrt => self.execute_sqrt(args),
            BuiltinFunction::Sin => self.execute_sin(args),
            BuiltinFunction::Cos => self.execute_cos(args),
            BuiltinFunction::Tan => self.execute_tan(args),
            BuiltinFunction::Floor => self.execute_floor(args),
            BuiltinFunction::Ceil => self.execute_ceil(args),
            BuiltinFunction::Round => self.execute_round(args),
            BuiltinFunction::Pow => self.execute_pow(args),
            BuiltinFunction::Min => self.execute_min(args),
            BuiltinFunction::Max => self.execute_max(args),
            BuiltinFunction::StringLen => self.execute_string_len(args),
            BuiltinFunction::Substring => self.execute_substring(args),
            BuiltinFunction::Concat => self.execute_concat(args),
            BuiltinFunction::Contains => self.execute_contains(args),
            BuiltinFunction::StartsWith => self.execute_starts_with(args),
            BuiltinFunction::EndsWith => self.execute_ends_with(args),
            BuiltinFunction::ToUpper => self.execute_to_upper(args),
            BuiltinFunction::ToLower => self.execute_to_lower(args),
            BuiltinFunction::Trim => self.execute_trim(args),
            BuiltinFunction::Split => self.execute_split(args),
            BuiltinFunction::Join => self.execute_join(args),
            BuiltinFunction::ToString => self.execute_to_string(args),
            BuiltinFunction::ToInt => self.execute_to_int(args),
            BuiltinFunction::ToFloat => self.execute_to_float(args),
            BuiltinFunction::Random => self.execute_random(args),
            BuiltinFunction::RandomInt => self.execute_random_int(args),
            BuiltinFunction::TypeOf => self.execute_typeof(args),
            BuiltinFunction::Time => self.execute_time(args),
            BuiltinFunction::Sleep => self.execute_sleep(args),
            BuiltinFunction::Exit => self.execute_exit(args),
            
            // Advanced Data Structures - HashMap/Dictionary
            BuiltinFunction::DictNew => self.execute_dict_new(args),
            BuiltinFunction::DictSet => self.execute_dict_set(args),
            BuiltinFunction::DictGet => self.execute_dict_get(args),
            BuiltinFunction::DictHas => self.execute_dict_has(args),
            BuiltinFunction::DictKeys => self.execute_dict_keys(args),
            BuiltinFunction::DictSize => self.execute_dict_size(args),
            BuiltinFunction::DictRemove => self.execute_dict_remove(args),
            BuiltinFunction::DictClear => self.execute_dict_clear(args),
            
            // Advanced Array functions
            BuiltinFunction::ArrayPush => self.execute_array_push(args),
            BuiltinFunction::ArrayPop => self.execute_array_pop(args),
            BuiltinFunction::ArrayReverse => self.execute_array_reverse(args),
            BuiltinFunction::ArraySort => self.execute_array_sort(args),
            BuiltinFunction::ArrayFilter => self.execute_array_filter(args),
            BuiltinFunction::ArrayMap => self.execute_array_map(args),
            BuiltinFunction::ArrayReduce => self.execute_array_reduce(args),
            BuiltinFunction::ArrayFind => self.execute_array_find(args),
            BuiltinFunction::ArraySlice => self.execute_array_slice(args),
            
            // JSON Support
            BuiltinFunction::JsonParse => self.execute_json_parse(args),
            BuiltinFunction::JsonStringify => self.execute_json_stringify(args),
            
            // Error Handling
            BuiltinFunction::TryCatch => self.execute_try_catch(args),
            BuiltinFunction::ThrowError => self.execute_throw_error(args),
            
            BuiltinFunction::Malloc => self.execute_malloc(args),
            BuiltinFunction::Free => self.execute_free(args),
            BuiltinFunction::Panic => self.execute_panic(args),
            BuiltinFunction::Assert => self.execute_assert(args),
        }
    }

    fn execute_print(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("print expects exactly 1 argument"));
        }
        
        let output = match &args[0] {
            Expression::Literal(Literal::String(s)) => s.clone(),
            Expression::Literal(Literal::Int(i)) => i.to_string(),
            Expression::Literal(Literal::Float(f)) => f.to_string(),
            Expression::Literal(Literal::Bool(b)) => b.to_string(),
            Expression::Literal(Literal::Null) => "null".to_string(),
            _ => return Err(CompilerError::runtime_error("print argument must be a literal value")),
        };
        
        print!("{}", output);
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_println(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("println expects exactly 1 argument"));
        }
        
        let output = match &args[0] {
            Expression::Literal(Literal::String(s)) => s.clone(),
            Expression::Literal(Literal::Int(i)) => i.to_string(),
            Expression::Literal(Literal::Float(f)) => f.to_string(),
            Expression::Literal(Literal::Bool(b)) => b.to_string(),
            Expression::Literal(Literal::Null) => "null".to_string(),
            _ => return Err(CompilerError::runtime_error("println argument must be a literal value")),
        };
        
        println!("{}", output);
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_read_line(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if !args.is_empty() {
            return Err(CompilerError::runtime_error("read_line expects no arguments"));
        }
        
        use std::io::{self, Write};
        
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                // Remove trailing newline
                if input.ends_with('\n') {
                    input.pop();
                    if input.ends_with('\r') {
                        input.pop();
                    }
                }
                Ok(Expression::Literal(Literal::String(input)))
            }
            Err(e) => Err(CompilerError::runtime_error(&format!("Failed to read line: {}", e))),
        }
    }

    // File I/O functions
    fn execute_read_file(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("read_file expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(path)) = &args[0] {
            match std::fs::read_to_string(path) {
                Ok(content) => Ok(Expression::Literal(Literal::String(content))),
                Err(e) => Err(CompilerError::runtime_error(&format!("Failed to read file '{}': {}", path, e))),
            }
        } else {
            Err(CompilerError::runtime_error("read_file expects a string path"))
        }
    }

    fn execute_write_file(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("write_file expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(path)), Expression::Literal(Literal::String(content))) = (&args[0], &args[1]) {
            match std::fs::write(path, content) {
                Ok(_) => Ok(Expression::Literal(Literal::Null)),
                Err(e) => Err(CompilerError::runtime_error(&format!("Failed to write file '{}': {}", path, e))),
            }
        } else {
            Err(CompilerError::runtime_error("write_file expects string arguments"))
        }
    }

    fn execute_append_file(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("append_file expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(path)), Expression::Literal(Literal::String(content))) = (&args[0], &args[1]) {
            use std::fs::OpenOptions;
            use std::io::Write;
            
            match OpenOptions::new().create(true).append(true).open(path) {
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(_) => Ok(Expression::Literal(Literal::Null)),
                        Err(e) => Err(CompilerError::runtime_error(&format!("Failed to append to file '{}': {}", path, e))),
                    }
                },
                Err(e) => Err(CompilerError::runtime_error(&format!("Failed to open file '{}' for appending: {}", path, e))),
            }
        } else {
            Err(CompilerError::runtime_error("append_file expects string arguments"))
        }
    }

    fn execute_file_exists(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("file_exists expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(path)) = &args[0] {
            let exists = std::path::Path::new(path).exists();
            Ok(Expression::Literal(Literal::Bool(exists)))
        } else {
            Err(CompilerError::runtime_error("file_exists expects a string path"))
        }
    }

    fn execute_abs(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("abs expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Int(i.abs()))),
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Float(f.abs()))),
            _ => Err(CompilerError::runtime_error("abs expects a numeric argument")),
        }
    }

    fn execute_sqrt(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("sqrt expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Float(f)) => {
                if *f < 0.0 {
                    Err(CompilerError::runtime_error("sqrt of negative number"))
                } else {
                    Ok(Expression::Literal(Literal::Float(f.sqrt())))
                }
            }
            Expression::Literal(Literal::Int(i)) => {
                if *i < 0 {
                    Err(CompilerError::runtime_error("sqrt of negative number"))
                } else {
                    Ok(Expression::Literal(Literal::Float((*i as f64).sqrt())))
                }
            }
            _ => Err(CompilerError::runtime_error("sqrt expects a numeric argument")),
        }
    }

    fn execute_sin(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("sin expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Float(f.sin()))),
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Float((*i as f64).sin()))),
            _ => Err(CompilerError::runtime_error("sin expects a numeric argument")),
        }
    }

    fn execute_cos(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("cos expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Float(f.cos()))),
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Float((*i as f64).cos()))),
            _ => Err(CompilerError::runtime_error("cos expects a numeric argument")),
        }
    }

    fn execute_tan(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("tan expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Float(f.tan()))),
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Float((*i as f64).tan()))),
            _ => Err(CompilerError::runtime_error("tan expects a numeric argument")),
        }
    }

    fn execute_floor(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("floor expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Int(f.floor() as i64))),
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Int(*i))),
            _ => Err(CompilerError::runtime_error("floor expects a numeric argument")),
        }
    }

    fn execute_ceil(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("ceil expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Int(f.ceil() as i64))),
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Int(*i))),
            _ => Err(CompilerError::runtime_error("ceil expects a numeric argument")),
        }
    }

    fn execute_round(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("round expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Int(f.round() as i64))),
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Int(*i))),
            _ => Err(CompilerError::runtime_error("round expects a numeric argument")),
        }
    }

    fn execute_pow(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("pow expects exactly 2 arguments"));
        }
        
        match (&args[0], &args[1]) {
            (Expression::Literal(Literal::Float(base)), Expression::Literal(Literal::Float(exp))) => {
                Ok(Expression::Literal(Literal::Float(base.powf(*exp))))
            }
            (Expression::Literal(Literal::Int(base)), Expression::Literal(Literal::Int(exp))) => {
                Ok(Expression::Literal(Literal::Float((*base as f64).powf(*exp as f64))))
            }
            (Expression::Literal(Literal::Float(base)), Expression::Literal(Literal::Int(exp))) => {
                Ok(Expression::Literal(Literal::Float(base.powf(*exp as f64))))
            }
            (Expression::Literal(Literal::Int(base)), Expression::Literal(Literal::Float(exp))) => {
                Ok(Expression::Literal(Literal::Float((*base as f64).powf(*exp))))
            }
            _ => Err(CompilerError::runtime_error("pow expects numeric arguments")),
        }
    }

    fn execute_min(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("min expects exactly 2 arguments"));
        }
        
        match (&args[0], &args[1]) {
            (Expression::Literal(Literal::Int(a)), Expression::Literal(Literal::Int(b))) => {
                Ok(Expression::Literal(Literal::Int(*a.min(b))))
            }
            (Expression::Literal(Literal::Float(a)), Expression::Literal(Literal::Float(b))) => {
                Ok(Expression::Literal(Literal::Float(a.min(*b))))
            }
            (Expression::Literal(Literal::Int(a)), Expression::Literal(Literal::Float(b))) => {
                Ok(Expression::Literal(Literal::Float((*a as f64).min(*b))))
            }
            (Expression::Literal(Literal::Float(a)), Expression::Literal(Literal::Int(b))) => {
                Ok(Expression::Literal(Literal::Float(a.min(*b as f64))))
            }
            _ => Err(CompilerError::runtime_error("min expects numeric arguments")),
        }
    }

    fn execute_max(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("max expects exactly 2 arguments"));
        }
        
        match (&args[0], &args[1]) {
            (Expression::Literal(Literal::Int(a)), Expression::Literal(Literal::Int(b))) => {
                Ok(Expression::Literal(Literal::Int(*a.max(b))))
            }
            (Expression::Literal(Literal::Float(a)), Expression::Literal(Literal::Float(b))) => {
                Ok(Expression::Literal(Literal::Float(a.max(*b))))
            }
            (Expression::Literal(Literal::Int(a)), Expression::Literal(Literal::Float(b))) => {
                Ok(Expression::Literal(Literal::Float((*a as f64).max(*b))))
            }
            (Expression::Literal(Literal::Float(a)), Expression::Literal(Literal::Int(b))) => {
                Ok(Expression::Literal(Literal::Float(a.max(*b as f64))))
            }
            _ => Err(CompilerError::runtime_error("max expects numeric arguments")),
        }
    }

    fn execute_string_len(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("len expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            Ok(Expression::Literal(Literal::Int(s.len() as i64)))
        } else {
            Err(CompilerError::runtime_error("len expects a string argument"))
        }
    }

    fn execute_substring(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 3 {
            return Err(CompilerError::runtime_error("substring expects exactly 3 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(s)), 
                Expression::Literal(Literal::Int(start)), 
                Expression::Literal(Literal::Int(len))) = (&args[0], &args[1], &args[2]) {
            
            let start_idx = *start as usize;
            let end_idx = start_idx + (*len as usize);
            
            if start_idx >= s.len() || end_idx > s.len() {
                return Err(CompilerError::runtime_error("substring indices out of bounds"));
            }
            
            Ok(Expression::Literal(Literal::String(s[start_idx..end_idx].to_string())))
        } else {
            Err(CompilerError::runtime_error("substring expects string, int, int arguments"))
        }
    }

    fn execute_concat(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("concat expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(s1)), Expression::Literal(Literal::String(s2))) = (&args[0], &args[1]) {
            Ok(Expression::Literal(Literal::String(format!("{}{}", s1, s2))))
        } else {
            Err(CompilerError::runtime_error("concat expects string arguments"))
        }
    }

    fn execute_contains(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("contains expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(s)), Expression::Literal(Literal::String(substr))) = (&args[0], &args[1]) {
            Ok(Expression::Literal(Literal::Bool(s.contains(substr))))
        } else {
            Err(CompilerError::runtime_error("contains expects string arguments"))
        }
    }

    fn execute_starts_with(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("starts_with expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(s)), Expression::Literal(Literal::String(prefix))) = (&args[0], &args[1]) {
            Ok(Expression::Literal(Literal::Bool(s.starts_with(prefix))))
        } else {
            Err(CompilerError::runtime_error("starts_with expects string arguments"))
        }
    }

    fn execute_ends_with(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("ends_with expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(s)), Expression::Literal(Literal::String(suffix))) = (&args[0], &args[1]) {
            Ok(Expression::Literal(Literal::Bool(s.ends_with(suffix))))
        } else {
            Err(CompilerError::runtime_error("ends_with expects string arguments"))
        }
    }

    fn execute_to_upper(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("to_upper expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            Ok(Expression::Literal(Literal::String(s.to_uppercase())))
        } else {
            Err(CompilerError::runtime_error("to_upper expects a string argument"))
        }
    }

    fn execute_to_lower(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("to_lower expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            Ok(Expression::Literal(Literal::String(s.to_lowercase())))
        } else {
            Err(CompilerError::runtime_error("to_lower expects a string argument"))
        }
    }

    fn execute_trim(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("trim expects exactly 1 argument"));
        }
        
        if let Expression::Literal(Literal::String(s)) = &args[0] {
            Ok(Expression::Literal(Literal::String(s.trim().to_string())))
        } else {
            Err(CompilerError::runtime_error("trim expects a string argument"))
        }
    }

    fn execute_split(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("split expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::String(s)), Expression::Literal(Literal::String(delimiter))) = (&args[0], &args[1]) {
            let parts: Vec<Literal> = s.split(delimiter)
                .map(|part| Literal::String(part.to_string()))
                .collect();
            Ok(Expression::Literal(Literal::Array(parts)))
        } else {
            Err(CompilerError::runtime_error("split expects string arguments"))
        }
    }

    fn execute_join(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("join expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::Array(arr)), Expression::Literal(Literal::String(delimiter))) = (&args[0], &args[1]) {
            let strings: Result<Vec<String>, _> = arr.iter().map(|item| {
                match item {
                    Literal::String(s) => Ok(s.clone()),
                    Literal::Int(i) => Ok(i.to_string()),
                    Literal::Float(f) => Ok(f.to_string()),
                    Literal::Bool(b) => Ok(b.to_string()),
                    Literal::Null => Ok("null".to_string()),
                    _ => Err(CompilerError::runtime_error("join can only handle primitive types")),
                }
            }).collect();
            
            match strings {
                Ok(string_vec) => Ok(Expression::Literal(Literal::String(string_vec.join(delimiter)))),
                Err(e) => Err(e),
            }
        } else {
            Err(CompilerError::runtime_error("join expects array and string arguments"))
        }
    }

    fn execute_to_string(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("to_string expects exactly 1 argument"));
        }
        
        let result = match &args[0] {
            Expression::Literal(Literal::Int(i)) => i.to_string(),
            Expression::Literal(Literal::Float(f)) => f.to_string(),
            Expression::Literal(Literal::Bool(b)) => b.to_string(),
            Expression::Literal(Literal::String(s)) => s.clone(),
            Expression::Literal(Literal::Null) => "null".to_string(),
            _ => return Err(CompilerError::runtime_error("to_string argument must be a literal value")),
        };
        
        Ok(Expression::Literal(Literal::String(result)))
    }

    fn execute_to_int(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("to_int expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::String(s)) => {
                match s.parse::<i64>() {
                    Ok(i) => Ok(Expression::Literal(Literal::Int(i))),
                    Err(_) => Err(CompilerError::runtime_error("Failed to parse string as integer")),
                }
            }
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Int(*f as i64))),
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Int(*i))),
            _ => Err(CompilerError::runtime_error("to_int expects a string or numeric argument")),
        }
    }

    fn execute_to_float(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("to_float expects exactly 1 argument"));
        }
        
        match &args[0] {
            Expression::Literal(Literal::String(s)) => {
                match s.parse::<f64>() {
                    Ok(f) => Ok(Expression::Literal(Literal::Float(f))),
                    Err(_) => Err(CompilerError::runtime_error("Failed to parse string as float")),
                }
            }
            Expression::Literal(Literal::Int(i)) => Ok(Expression::Literal(Literal::Float(*i as f64))),
            Expression::Literal(Literal::Float(f)) => Ok(Expression::Literal(Literal::Float(*f))),
            _ => Err(CompilerError::runtime_error("to_float expects a string or numeric argument")),
        }
    }

    fn execute_malloc(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("malloc expects exactly 1 argument"));
        }
        
        // For now, just return a mock pointer value
        // In a real implementation, this would allocate memory
        if let Expression::Literal(Literal::Int(size)) = &args[0] {
            if *size <= 0 {
                return Err(CompilerError::runtime_error("malloc size must be positive"));
            }
            // Return a mock pointer address
            Ok(Expression::Literal(Literal::Int(*size)))
        } else {
            Err(CompilerError::runtime_error("malloc expects an integer size"))
        }
    }

    fn execute_free(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("free expects exactly 1 argument"));
        }
        
        // For now, just validate the argument is an integer (mock pointer)
        // In a real implementation, this would free memory
        if let Expression::Literal(Literal::Int(_)) = &args[0] {
            Ok(Expression::Literal(Literal::Null))
        } else {
            Err(CompilerError::runtime_error("free expects a pointer argument"))
        }
    }

    fn execute_panic(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("panic expects exactly 1 argument"));
        }
        
        let message = match &args[0] {
            Expression::Literal(Literal::String(s)) => s.clone(),
            _ => "panic called".to_string(),
        };
        
        Err(CompilerError::runtime_error(&format!("panic: {}", message)))
    }

    fn execute_assert(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("assert expects exactly 2 arguments"));
        }
        
        if let Expression::Literal(Literal::Bool(condition)) = &args[0] {
            if !condition {
                let message = if let Expression::Literal(Literal::String(s)) = &args[1] {
                    s.clone()
                } else {
                    "assertion failed".to_string()
                };
                return Err(CompilerError::runtime_error(&format!("assert: {}", message)));
            }
            Ok(Expression::Literal(Literal::Null))
        } else {
            Err(CompilerError::runtime_error("assert expects a boolean condition"))
        }
    }

    fn execute_random(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 0 {
            return Err(CompilerError::runtime_error("random expects no arguments"));
        }
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Simple pseudo-random number generator using current time
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let hash = hasher.finish();
        
        // Convert to float between 0.0 and 1.0
        let random_float = (hash as f64) / (u64::MAX as f64);
        Ok(Expression::Literal(Literal::Float(random_float)))
    }

    fn execute_random_int(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("random_int expects exactly 2 arguments"));
        }
        
        if let (Expression::Literal(Literal::Int(min)), Expression::Literal(Literal::Int(max))) = (&args[0], &args[1]) {
            if min >= max {
                return Err(CompilerError::runtime_error("random_int: min must be less than max"));
            }
            
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            use std::time::{SystemTime, UNIX_EPOCH};
            
            let mut hasher = DefaultHasher::new();
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
            let hash = hasher.finish();
            
            let range = (max - min) as u64;
            let random_int = min + ((hash % range) as i64);
            Ok(Expression::Literal(Literal::Int(random_int)))
        } else {
            Err(CompilerError::runtime_error("random_int expects integer arguments"))
        }
    }

    fn execute_typeof(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("typeof expects exactly 1 argument"));
        }
        
        let type_name = match &args[0] {
            Expression::Literal(Literal::Int(_)) => "int",
            Expression::Literal(Literal::Float(_)) => "float", 
            Expression::Literal(Literal::String(_)) => "string",
            Expression::Literal(Literal::Bool(_)) => "bool",
            Expression::Literal(Literal::Char(_)) => "char",
            Expression::Literal(Literal::Array(_)) => "array",
            Expression::Literal(Literal::Null) => "null",
            Expression::Identifier(_) => "identifier",
            Expression::FunctionCall(_, _) => "function_call",
            Expression::BinaryOp(_) => "binary_op",
            Expression::UnaryOp(_) => "unary_op",
            Expression::Block(_) => "block",
            Expression::If(_) => "if",
            Expression::While(_) => "while",
            Expression::ArrayAccess(_) => "array_access",
            _ => "unknown",
        };
        
        Ok(Expression::Literal(Literal::String(type_name.to_string())))
    }

    fn execute_time(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 0 {
            return Err(CompilerError::runtime_error("time expects no arguments"));
        }
        
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        Ok(Expression::Literal(Literal::Int(timestamp)))
    }

    fn execute_sleep(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("sleep expects exactly 1 argument"));
        }
        
        let duration = match &args[0] {
            Expression::Literal(Literal::Int(seconds)) => *seconds as u64,
            Expression::Literal(Literal::Float(seconds)) => *seconds as u64,
            _ => return Err(CompilerError::runtime_error("sleep expects a numeric argument")),
        };
        
        std::thread::sleep(std::time::Duration::from_secs(duration));
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_exit(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        let exit_code = if args.len() == 0 {
            0
        } else if args.len() == 1 {
            match &args[0] {
                Expression::Literal(Literal::Int(code)) => *code as i32,
                _ => return Err(CompilerError::runtime_error("exit expects an integer argument")),
            }
        } else {
            return Err(CompilerError::runtime_error("exit expects 0 or 1 arguments"));
        };
        
        std::process::exit(exit_code);
    }

    // Advanced Data Structures - HashMap/Dictionary
    fn execute_dict_new(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 0 {
            return Err(CompilerError::runtime_error("dict_new expects no arguments"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_dict_set(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 3 {
            return Err(CompilerError::runtime_error("dict_set expects 3 arguments: dict, key, value"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_dict_get(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("dict_get expects 2 arguments: dict, key"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_dict_has(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("dict_has expects 2 arguments: dict, key"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Bool(false)))
    }

    fn execute_dict_keys(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("dict_keys expects 1 argument: dict"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Array(vec![])))
    }

    fn execute_dict_size(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("dict_size expects 1 argument: dict"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Int(0)))
    }

    fn execute_dict_remove(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("dict_remove expects 2 arguments: dict, key"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_dict_clear(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("dict_clear expects 1 argument: dict"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    // Advanced Array functions
    fn execute_array_push(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("array_push expects 2 arguments: array, value"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_array_pop(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("array_pop expects 1 argument: array"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_array_reverse(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("array_reverse expects 1 argument: array"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_array_sort(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("array_sort expects 1 argument: array"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_array_filter(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("array_filter expects 2 arguments: array, predicate"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Array(vec![])))
    }

    fn execute_array_map(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("array_map expects 2 arguments: array, mapper"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Array(vec![])))
    }

    fn execute_array_reduce(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 3 {
            return Err(CompilerError::runtime_error("array_reduce expects 3 arguments: array, reducer, initial"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_array_find(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("array_find expects 2 arguments: array, predicate"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_array_slice(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 3 {
            return Err(CompilerError::runtime_error("array_slice expects 3 arguments: array, start, end"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Array(vec![])))
    }

    // JSON Support
    fn execute_json_parse(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("json_parse expects 1 argument: json_string"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_json_stringify(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("json_stringify expects 1 argument: value"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::String("null".to_string())))
    }

    // Error Handling
    fn execute_try_catch(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 2 {
            return Err(CompilerError::runtime_error("try_catch expects 2 arguments: try_expr, catch_expr"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }

    fn execute_throw_error(&self, args: &[Expression]) -> Result<Expression, CompilerError> {
        if args.len() != 1 {
            return Err(CompilerError::runtime_error("throw_error expects 1 argument: error_message"));
        }
        // Return a placeholder - actual implementation handled in VM
        Ok(Expression::Literal(Literal::Null))
    }
}

#[derive(Debug)]
pub struct StandardLibrary {
    functions: HashMap<String, (Type, BuiltinFunction)>,
}

impl StandardLibrary {
    pub fn new() -> Self {
        let mut stdlib = StandardLibrary {
            functions: HashMap::new(),
        };
        stdlib.register_builtins();
        stdlib
    }

    fn register_function(&mut self, name: &str, signature: Type, function: BuiltinFunction) {
        self.functions.insert(name.to_string(), (signature, function));
    }

    fn register_builtins(&mut self) {
        // I/O functions
        self.register_function("print", Type::Function(vec![Type::String], Box::new(Type::Void)), 
            BuiltinFunction::Print);
        self.register_function("println", Type::Function(vec![Type::String], Box::new(Type::Void)), 
            BuiltinFunction::Println);
        self.register_function("read_line", Type::Function(vec![], Box::new(Type::String)), 
            BuiltinFunction::ReadLine);
        
        // File I/O functions
        self.register_function("read_file", Type::Function(vec![Type::String], Box::new(Type::String)), 
            BuiltinFunction::ReadFile);
        self.register_function("write_file", Type::Function(vec![Type::String, Type::String], Box::new(Type::Void)), 
            BuiltinFunction::WriteFile);
        self.register_function("append_file", Type::Function(vec![Type::String, Type::String], Box::new(Type::Void)), 
            BuiltinFunction::AppendFile);
        self.register_function("file_exists", Type::Function(vec![Type::String], Box::new(Type::Bool)), 
            BuiltinFunction::FileExists);
        
        // Math functions
        self.register_function("abs", Type::Function(vec![Type::Int], Box::new(Type::Int)), 
            BuiltinFunction::Abs);
        self.register_function("sqrt", Type::Function(vec![Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Sqrt);
        self.register_function("sin", Type::Function(vec![Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Sin);
        self.register_function("cos", Type::Function(vec![Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Cos);
        self.register_function("tan", Type::Function(vec![Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Tan);
        self.register_function("floor", Type::Function(vec![Type::Float], Box::new(Type::Int)), 
            BuiltinFunction::Floor);
        self.register_function("ceil", Type::Function(vec![Type::Float], Box::new(Type::Int)), 
            BuiltinFunction::Ceil);
        self.register_function("round", Type::Function(vec![Type::Float], Box::new(Type::Int)), 
            BuiltinFunction::Round);
        self.register_function("pow", Type::Function(vec![Type::Float, Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Pow);
        self.register_function("min", Type::Function(vec![Type::Float, Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Min);
        self.register_function("max", Type::Function(vec![Type::Float, Type::Float], Box::new(Type::Float)), 
            BuiltinFunction::Max);
        
        // String functions
        self.register_function("len", Type::Function(vec![Type::String], Box::new(Type::Int)), 
            BuiltinFunction::StringLen);
        self.register_function("substring", Type::Function(vec![Type::String, Type::Int, Type::Int], Box::new(Type::String)), 
            BuiltinFunction::Substring);
        self.register_function("concat", Type::Function(vec![Type::String, Type::String], Box::new(Type::String)), 
            BuiltinFunction::Concat);
        self.register_function("contains", Type::Function(vec![Type::String, Type::String], Box::new(Type::Bool)), 
            BuiltinFunction::Contains);
        self.register_function("starts_with", Type::Function(vec![Type::String, Type::String], Box::new(Type::Bool)), 
            BuiltinFunction::StartsWith);
        self.register_function("ends_with", Type::Function(vec![Type::String, Type::String], Box::new(Type::Bool)), 
            BuiltinFunction::EndsWith);
        self.register_function("to_upper", Type::Function(vec![Type::String], Box::new(Type::String)), 
            BuiltinFunction::ToUpper);
        self.register_function("to_lower", Type::Function(vec![Type::String], Box::new(Type::String)), 
            BuiltinFunction::ToLower);
        self.register_function("trim", Type::Function(vec![Type::String], Box::new(Type::String)), 
            BuiltinFunction::Trim);
        self.register_function("split", Type::Function(vec![Type::String, Type::String], Box::new(Type::Array(Box::new(Type::String), 0))), 
            BuiltinFunction::Split);
        self.register_function("join", Type::Function(vec![Type::Array(Box::new(Type::String), 0), Type::String], Box::new(Type::String)), 
            BuiltinFunction::Join);
        
        // Type conversion
        self.register_function("to_string", Type::Function(vec![Type::Int], Box::new(Type::String)), 
            BuiltinFunction::ToString);
        self.register_function("to_int", Type::Function(vec![Type::String], Box::new(Type::Int)), 
            BuiltinFunction::ToInt);
        self.register_function("to_float", Type::Function(vec![Type::String], Box::new(Type::Float)), 
            BuiltinFunction::ToFloat);
        
        // Utility functions
        self.register_function("random", Type::Function(vec![], Box::new(Type::Float)), 
            BuiltinFunction::Random);
        self.register_function("random_int", Type::Function(vec![Type::Int, Type::Int], Box::new(Type::Int)), 
            BuiltinFunction::RandomInt);
        self.register_function("typeof", Type::Function(vec![Type::Unknown], Box::new(Type::String)), 
            BuiltinFunction::TypeOf);
        self.register_function("time", Type::Function(vec![], Box::new(Type::Int)), 
            BuiltinFunction::Time);
        self.register_function("sleep", Type::Function(vec![Type::Int], Box::new(Type::Void)), 
            BuiltinFunction::Sleep);
        self.register_function("exit", Type::Function(vec![Type::Int], Box::new(Type::Never)), 
            BuiltinFunction::Exit);
        
        // Advanced Data Structures - HashMap/Dictionary
        self.register_function("dict_new", Type::Function(vec![], Box::new(Type::Any)), 
            BuiltinFunction::DictNew);
        self.register_function("dict_set", Type::Function(vec![Type::Any, Type::String, Type::Any], Box::new(Type::Any)), 
            BuiltinFunction::DictSet);
        self.register_function("dict_get", Type::Function(vec![Type::Any, Type::String], Box::new(Type::Any)), 
            BuiltinFunction::DictGet);
        self.register_function("dict_has", Type::Function(vec![Type::Any, Type::String], Box::new(Type::Bool)), 
            BuiltinFunction::DictHas);
        self.register_function("dict_keys", Type::Function(vec![Type::Any], Box::new(Type::Array(Box::new(Type::String), 0))), 
            BuiltinFunction::DictKeys);
        self.register_function("dict_size", Type::Function(vec![Type::Any], Box::new(Type::Int)), 
            BuiltinFunction::DictSize);
        self.register_function("dict_remove", Type::Function(vec![Type::Any, Type::String], Box::new(Type::Any)), 
            BuiltinFunction::DictRemove);
        self.register_function("dict_clear", Type::Function(vec![Type::Any], Box::new(Type::Any)), 
            BuiltinFunction::DictClear);
        
        // Advanced Array functions
        self.register_function("array_push", Type::Function(vec![Type::Array(Box::new(Type::Any), 0), Type::Any], Box::new(Type::Array(Box::new(Type::Any), 0))), 
            BuiltinFunction::ArrayPush);
        self.register_function("array_pop", Type::Function(vec![Type::Array(Box::new(Type::Any), 0)], Box::new(Type::Any)), 
            BuiltinFunction::ArrayPop);
        self.register_function("array_reverse", Type::Function(vec![Type::Array(Box::new(Type::Any), 0)], Box::new(Type::Array(Box::new(Type::Any), 0))), 
            BuiltinFunction::ArrayReverse);
        self.register_function("array_sort", Type::Function(vec![Type::Array(Box::new(Type::Any), 0)], Box::new(Type::Array(Box::new(Type::Any), 0))), 
            BuiltinFunction::ArraySort);
        self.register_function("array_filter", Type::Function(vec![Type::Array(Box::new(Type::Any), 0), Type::Function(vec![Type::Any], Box::new(Type::Bool))], Box::new(Type::Array(Box::new(Type::Any), 0))), 
            BuiltinFunction::ArrayFilter);
        self.register_function("array_map", Type::Function(vec![Type::Array(Box::new(Type::Any), 0), Type::Function(vec![Type::Any], Box::new(Type::Any))], Box::new(Type::Array(Box::new(Type::Any), 0))), 
            BuiltinFunction::ArrayMap);
        self.register_function("array_reduce", Type::Function(vec![Type::Array(Box::new(Type::Any), 0), Type::Function(vec![Type::Any, Type::Any], Box::new(Type::Any)), Type::Any], Box::new(Type::Any)), 
            BuiltinFunction::ArrayReduce);
        self.register_function("array_find", Type::Function(vec![Type::Array(Box::new(Type::Any), 0), Type::Function(vec![Type::Any], Box::new(Type::Bool))], Box::new(Type::Any)), 
            BuiltinFunction::ArrayFind);
        self.register_function("array_slice", Type::Function(vec![Type::Array(Box::new(Type::Any), 0), Type::Int, Type::Int], Box::new(Type::Array(Box::new(Type::Any), 0))), 
            BuiltinFunction::ArraySlice);
        
        // JSON Support
        self.register_function("json_parse", Type::Function(vec![Type::String], Box::new(Type::Any)), 
            BuiltinFunction::JsonParse);
        self.register_function("json_stringify", Type::Function(vec![Type::Any], Box::new(Type::String)), 
            BuiltinFunction::JsonStringify);
        
        // Error Handling
        self.register_function("try_catch", Type::Function(vec![Type::Any, Type::Any], Box::new(Type::Any)), 
            BuiltinFunction::TryCatch);
        self.register_function("throw_error", Type::Function(vec![Type::String], Box::new(Type::Never)), 
            BuiltinFunction::ThrowError);
        
        // Memory management
        self.register_function("malloc", Type::Function(vec![Type::Int], Box::new(Type::Pointer(Box::new(Type::Void)))), 
            BuiltinFunction::Malloc);
        self.register_function("free", Type::Function(vec![Type::Pointer(Box::new(Type::Void))], Box::new(Type::Void)), 
            BuiltinFunction::Free);
        
        // Error handling
        self.register_function("panic", Type::Function(vec![Type::String], Box::new(Type::Never)), 
            BuiltinFunction::Panic);
        self.register_function("assert", Type::Function(vec![Type::Bool, Type::String], Box::new(Type::Void)), 
            BuiltinFunction::Assert);
    }

    pub fn get_function(&self, name: &str) -> Option<&(Type, BuiltinFunction)> {
        self.functions.get(name)
    }
    
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name).map(|(_, builtin)| builtin)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn list_functions(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }
}

impl Default for StandardLibrary {
    fn default() -> Self {
        Self::new()
    }
}