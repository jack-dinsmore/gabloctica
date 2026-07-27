use lazy_static::lazy_static;
use num_enum::TryFromPrimitive;
use rustc_hash::FxHashMap;
use strum::IntoEnumIterator;

lazy_static! {
    static ref COMMAND_MAP : FxHashMap<String, Command> = {
        let mut map = FxHashMap::default();
        for v in Command::iter() {
            map.insert(v.to_string().to_lowercase(), v);
            map.insert(v.to_string().to_uppercase(), v);
        }
        map
    };
    static ref FUNCTION_MAP : FxHashMap<String, GlobalFunction> = {
        let mut map = FxHashMap::default();
        for v in GlobalFunction::iter() {
            map.insert(v.to_string().to_lowercase(), v);
            map.insert(v.to_string().to_uppercase(), v);
        }
        map
    };
    pub static ref FUNCTION_MAP_LOWER : FxHashMap<String, GlobalFunction> = {
        let mut map = FxHashMap::default();
        for v in GlobalFunction::iter() {
            map.insert(v.to_string().to_lowercase(), v);
        }
        map
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VariableType {
    Null,
    Float,
    List,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, strum_macros::Display, strum_macros::EnumIter, TryFromPrimitive)]
pub enum Command {
    // In the above, T stands for stack top, N stands for next, and A stands for an argument.
    Nop,        // No operation
    Call,       // Call T
    Jmp,        // Unconditional jump to A
    Jnz,        // Jump to A if not equal to zero

    Push,       // Push A
    Pop,        // Pop T
    Dup,        // Copy T
    Pip,        // Push the IP
    Jpop,       // Pop T to IP
    Swp,        // Swap T and N
    Pick,       // Pick T down in the stack

    Alc,        // Allocate new vector and push the vector index
    Ld,         // Push the index N of vector T
    St,         // Store T in index NN of vector N
    Stb,        // Store T at back of vector N
    
    Lt,         // Push T < N
    Gt,         // Push T > N
    Le,         // Push T <= N
    Ge,         // Push T >= N
    Eq,         // Push T == N
    
    And,        // Push T && N
    Or,         // Push T || N
    Xor,        // Push T ^ N
    Not,        // Push !T

    Add,        // Push T + N
    Sub,        // Push T - N
    Mul,        // Push T * N
    Div,        // Push T / N
    Neg,        // Push -T
    Pow,        // Push T**N
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, strum_macros::Display, strum_macros::EnumIter, TryFromPrimitive)]
pub enum GlobalFunction {
    Dbg,
    Tick,
    Interrupt,
}

impl Command {
    pub fn from_string(s: &str) -> Option<Self> {
        match COMMAND_MAP.get(s) {
            Some(k) => Some(k.to_owned()),
            None => None,
        }
    }

    pub fn takes_lit_arg(&self) -> bool {
        match self {
            Self::Push => true,
            _ => false,
        }
    }

    pub fn takes_func_arg(&self) -> bool {
        match self {
            Self::Call => true,
            _ => false,
        }
    }

    pub fn takes_label_arg(&self) -> bool {
        match self {
            Self::Jmp | Command::Jnz => true,
            _ => false,
        }
    }
}

impl GlobalFunction {
    pub fn from_string(s: &str) -> Option<Self> {
        match FUNCTION_MAP.get(s) {
            Some(k) => Some(k.to_owned()),
            None => None,
        }
    }

    pub fn return_type(&self) -> VariableType {
        match self {
            GlobalFunction::Dbg => VariableType::Null,
            GlobalFunction::Tick => VariableType::Null,
            GlobalFunction::Interrupt => VariableType::Null,
        }
    }
}