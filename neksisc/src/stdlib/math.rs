use std::f64::consts;
use crate::ast::Expression;
use crate::error::CompilerError;
use rand::Rng;

pub struct MathModule;

impl MathModule {
    pub fn new() -> Self {
        Self
    }
}

// Basic arithmetic functions
pub fn add(a: f64, b: f64) -> f64 {
    a + b
}

pub fn subtract(a: f64, b: f64) -> f64 {
    a - b
}

pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

pub fn divide(a: f64, b: f64) -> Result<f64, CompilerError> {
    if b == 0.0 {
        Err(CompilerError::runtime_error("Division by zero"))
    } else {
        Ok(a / b)
    }
}

pub fn modulo(a: f64, b: f64) -> Result<f64, CompilerError> {
    if b == 0.0 {
        Err(CompilerError::runtime_error("Modulo by zero"))
    } else {
        Ok(a % b)
    }
}

pub fn power(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

// Advanced math functions
pub fn sqrt(x: f64) -> Result<f64, CompilerError> {
    if x < 0.0 {
        Err(CompilerError::runtime_error("Cannot take square root of negative number"))
    } else {
        Ok(x.sqrt())
    }
}

pub fn cbrt(x: f64) -> f64 {
    x.cbrt()
}

pub fn abs(x: f64) -> f64 {
    x.abs()
}

pub fn floor(x: f64) -> f64 {
    x.floor()
}

pub fn ceil(x: f64) -> f64 {
    x.ceil()
}

pub fn round(x: f64) -> f64 {
    x.round()
}

pub fn trunc(x: f64) -> f64 {
    x.trunc()
}

pub fn fract(x: f64) -> f64 {
    x.fract()
}

// Trigonometric functions
pub fn sin(x: f64) -> f64 {
    x.sin()
}

pub fn cos(x: f64) -> f64 {
    x.cos()
}

pub fn tan(x: f64) -> f64 {
    x.tan()
}

pub fn asin(x: f64) -> Result<f64, CompilerError> {
    if x < -1.0 || x > 1.0 {
        Err(CompilerError::runtime_error("Arcsin domain error: value must be between -1 and 1"))
    } else {
        Ok(x.asin())
    }
}

pub fn acos(x: f64) -> Result<f64, CompilerError> {
    if x < -1.0 || x > 1.0 {
        Err(CompilerError::runtime_error("Arccos domain error: value must be between -1 and 1"))
    } else {
        Ok(x.acos())
    }
}

pub fn atan(x: f64) -> f64 {
    x.atan()
}

pub fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

// Hyperbolic functions
pub fn sinh(x: f64) -> f64 {
    x.sinh()
}

pub fn cosh(x: f64) -> f64 {
    x.cosh()
}

pub fn tanh(x: f64) -> f64 {
    x.tanh()
}

pub fn asinh(x: f64) -> f64 {
    x.asinh()
}

pub fn acosh(x: f64) -> Result<f64, CompilerError> {
    if x < 1.0 {
        Err(CompilerError::runtime_error("Acosh domain error: value must be >= 1"))
    } else {
        Ok(x.acosh())
    }
}

pub fn atanh(x: f64) -> Result<f64, CompilerError> {
    if x <= -1.0 || x >= 1.0 {
        Err(CompilerError::runtime_error("Atanh domain error: value must be between -1 and 1"))
    } else {
        Ok(x.atanh())
    }
}

// Logarithmic functions
pub fn ln(x: f64) -> Result<f64, CompilerError> {
    if x <= 0.0 {
        Err(CompilerError::runtime_error("Natural logarithm domain error: value must be positive"))
    } else {
        Ok(x.ln())
    }
}

pub fn log10(x: f64) -> Result<f64, CompilerError> {
    if x <= 0.0 {
        Err(CompilerError::runtime_error("Log base 10 domain error: value must be positive"))
    } else {
        Ok(x.log10())
    }
}

pub fn log2(x: f64) -> Result<f64, CompilerError> {
    if x <= 0.0 {
        Err(CompilerError::runtime_error("Log base 2 domain error: value must be positive"))
    } else {
        Ok(x.log2())
    }
}

pub fn log_base(x: f64, base: f64) -> Result<f64, CompilerError> {
    if x <= 0.0 || base <= 0.0 || base == 1.0 {
        Err(CompilerError::runtime_error("Log domain error: values must be positive and base must not be 1"))
    } else {
        Ok(x.log(base))
    }
}

// Exponential functions
pub fn exp(x: f64) -> f64 {
    x.exp()
}

pub fn exp2(x: f64) -> f64 {
    x.exp2()
}

pub fn exp_m1(x: f64) -> f64 {
    x.exp_m1()
}

// Constants
pub fn pi() -> f64 {
    consts::PI
}

pub fn e() -> f64 {
    consts::E
}

pub fn tau() -> f64 {
    consts::TAU
}

pub fn phi() -> f64 {
    1.618033988749895 // Golden ratio
}

pub fn sqrt_2() -> f64 {
    consts::SQRT_2
}

pub fn sqrt_3() -> f64 {
    1.7320508075688772
}

// Random number generation
pub fn random() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn random_range(min: f64, max: f64) -> Result<f64, CompilerError> {
    if min >= max {
        Err(CompilerError::runtime_error("Random range error: min must be less than max"))
    } else {
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(min..max))
    }
}

pub fn random_int(min: i64, max: i64) -> Result<i64, CompilerError> {
    if min >= max {
        Err(CompilerError::runtime_error("Random int range error: min must be less than max"))
    } else {
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(min..=max))
    }
}

// Statistical functions
pub fn min(a: f64, b: f64) -> f64 {
    a.min(b)
}

pub fn max(a: f64, b: f64) -> f64 {
    a.max(b)
}

pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.clamp(min, max)
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// Comparison functions
pub fn is_nan(x: f64) -> bool {
    x.is_nan()
}

pub fn is_infinite(x: f64) -> bool {
    x.is_infinite()
}

pub fn is_finite(x: f64) -> bool {
    x.is_finite()
}

pub fn is_normal(x: f64) -> bool {
    x.is_normal()
}

// Bit manipulation (for integers)
pub fn bit_and(a: i64, b: i64) -> i64 {
    a & b
}

pub fn bit_or(a: i64, b: i64) -> i64 {
    a | b
}

pub fn bit_xor(a: i64, b: i64) -> i64 {
    a ^ b
}

pub fn bit_not(a: i64) -> i64 {
    !a
}

pub fn left_shift(a: i64, b: i64) -> i64 {
    a << b
}

pub fn right_shift(a: i64, b: i64) -> i64 {
    a >> b
}

// Builtin function implementations for the standard library
pub struct BuiltinFunction;

impl BuiltinFunction {
    pub fn execute(&self, _args: &[Expression]) -> Result<Expression, CompilerError> {
        Err(CompilerError::runtime_error("BuiltinFunction not implemented"))
    }
}

pub struct BuiltinImpl;

impl BuiltinImpl {
    pub fn new() -> Self {
        Self
    }
} 