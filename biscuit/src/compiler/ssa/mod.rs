mod ssa_data;
mod ordering;

use crate::bytecode::VariableType;
pub(crate) use ordering::Ssa;

#[derive(Clone, Debug)]
pub enum Instruction {
    Argument,
    LiteralVector(Vec<f64>),
    LiteralFloat(f64),

    Call(u8, Location),
    LocalCall(String, Location),
    Theta(usize, String),
    Action(usize),

    Ld(Location, Location), // Get index B f vector A
    St(Location, Location, Location), // Store C in index B of vector A 
    Stb(Location, Location), // Store C in back of vector A 
    
    Lt(Location, Location),
    Gt(Location, Location),
    Le(Location, Location), 
    Ge(Location, Location), 
    Eq(Location, Location),
    
    And(Location, Location),
    Or(Location, Location),
    Xor(Location, Location),
    Not(Location),

    Add(Location, Location),
    Sub(Location, Location),
    Mul(Location, Location),
    Div(Location, Location),
    Neg(Location),
    Pow(Location, Location),
}
impl Instruction {
    pub fn is_action(&self) -> bool {
        match &self {
            Instruction::Action(_) | Instruction::Call(_, _) | Instruction::LocalCall(_, _) | Instruction::Stb(_, _) |
            Instruction::St(_, _, _) => true,
            _ => false,
        }
    }
    pub fn typ(&self) -> VariableType {
        match &self {
            Instruction::LiteralVector(_) | Instruction::St(_, _, _) | Instruction::Stb(_, _) => VariableType::List,
            Instruction::Ld(_, _) | Instruction::LiteralFloat(_) | Instruction::Lt(_, _) | Instruction::Gt(_, _) | 
            Instruction::Le(_, _) |  Instruction::Ge(_, _) | Instruction::Eq(_, _) | Instruction::And(_, _) |
            Instruction::Or(_, _) | Instruction::Xor(_, _) | Instruction::Not(_) | Instruction::Add(_, _) | 
            Instruction::Sub(_, _) | Instruction::Mul(_, _) | Instruction::Div(_, _) | Instruction::Neg(_) | 
            Instruction::Pow(_, _) => VariableType::Float,
            Instruction::LocalCall(_, _) | Instruction::Action(_) => VariableType::Null,
            Instruction::Argument | Instruction::Call(_, _) | Instruction::Theta(_, _) => unreachable!(),
        }
    }
    fn get_var_dependencies(&self) -> Vec<Location> {
        match &self {
            Instruction::Argument | Instruction::LiteralVector(_) | Instruction::LiteralFloat(_) => vec![],
            Instruction::Theta(_, _) | Instruction::Action(_) => unreachable!(),
            Instruction::Call(_, a) | Instruction::LocalCall(_, a) | Instruction::Not(a) | Instruction::Neg(a) => {
                vec![*a]
            },
            Instruction::Ld(a, b) | Instruction::Stb(a, b) | Instruction::Lt(a, b) | Instruction::Gt(a, b) |
            Instruction::Le(a, b) | Instruction::Ge(a, b) | Instruction::Eq(a, b) | Instruction::And(a, b) |
            Instruction::Or(a, b) | Instruction::Xor(a, b) | Instruction::Add(a, b) | Instruction::Sub(a, b) |
            Instruction::Mul(a, b) | Instruction::Div(a, b) | Instruction::Pow(a, b) => {
                vec![*a, *b]
            },
            Instruction::St(a, b, c) => {
                vec![*a, *b, *c]
            },
        }
    }
}

#[derive(Clone)]
pub(super) enum Branch {
    If(Vec<(Ssa, Ssa)>, Option<Ssa>), // ((body code, if condition), else code)
    Loop(Ssa),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Location {
    pub tier: u32,
    pub index: u32,
}
impl Location {
    pub fn internal(index: u32) -> Self {
        Self {
            tier: 0,
            index
        }
    }
    pub fn graduate(&self) -> Self{
        Self {
            tier: self.tier + 1,
            index: self.index
        }
    }
}
impl std::fmt::Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.tier == 0 {
            write!(f, "{:04}", self.index)
        } else {
            write!(f, "{}_{:04}", self.tier, self.index)
        }
    }
}

