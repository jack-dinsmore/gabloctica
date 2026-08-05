use std::{ops::{Deref, DerefMut}, slice::Iter};

use rustc_hash::FxHashMap;

use crate::{bytecode::VariableType, compiler::{Function, ssa::{Instruction, Location, ssa_data::SsaData}}, parser::SyntaxNode};


#[derive(Debug, Clone)]
pub(crate) enum Ssa {
    Unordered { data: SsaData, },
    Ordered { data: SsaData, instruction_order: Vec<u32>, last_used: FxHashMap<u32, u32> }
}

impl Ssa {
    pub fn new(node: &SyntaxNode, arguments: &FxHashMap<String, VariableType>, available_functions: &FxHashMap<String, Function>) -> Result<Self, String> {
        Ok(Ssa::Unordered{data: SsaData::new(node, arguments, available_functions)?})
    }

    /// Get the instruction order of this branch and return those used by previous tiers (if there are any)
    pub fn order(&mut self) -> Vec<Location> {
        let mut last_used = FxHashMap::default();

        // Add all the return variables
        for v in &self.return_variables {
            if v.tier == 0 { 
                last_used.insert(v.index, u32::MAX);
            }
        }

        let mut reverse_instruction_order = Vec::new();
        let mut previously_used = Vec::new();
        for instruction_index in (0..self.instructions.len() as u32).rev() {
            let instruction = self.instructions[&instruction_index].clone();
            if !last_used.contains_key(&instruction_index) && !instruction.is_action() { continue; }

            reverse_instruction_order.push(instruction_index);
            let dependencies = match instruction {
                Instruction::Action(b) | Instruction::Theta(b, _) => {
                    // Get the branch dependencies
                    match &mut self.branches[b] {
                        super::Branch::If(items, ssa) => {
                            let mut output = Vec::new();
                            for (body, condition) in items {
                                output.append(&mut body.order());
                                dbg!(&output);
                                output.append(&mut condition.order());
                                dbg!(&output);
                            }
                            if let Some(ssa) = ssa {
                                output.append(&mut ssa.order());
                                dbg!(&output);
                            }
                            output
                        },
                        super::Branch::Loop(ssa) => {
                            ssa.order()
                        },
                    }
                },
                _ => {
                    instruction.get_var_dependencies()
                }
            };
            for var in dependencies {
                if var.tier == 0 {
                    if !last_used.contains_key(&var.index) {
                        last_used.insert(var.index, instruction_index);
                    }
                } else {
                    previously_used.push(Location {
                        tier: var.tier - 1,
                        index: var.index
                    })
                }
            }
        }
        reverse_instruction_order.reverse();
        let instruction_order = reverse_instruction_order;

        let data = match self {
            Self::Ordered { data, .. } => data,
            Self::Unordered { data, .. } => data,
        };
        // OPTIMIZE
        *self = Self::Ordered { data: data.clone(), instruction_order, last_used };

        previously_used
    }

    pub fn iter<'a>(&'a self) -> InstructionIterator<'a> {
        match self {
            Self::Ordered { data, instruction_order, .. } => {
                InstructionIterator {
                    instructions: &data.instructions,
                    instruction_index_iterator: instruction_order.iter()
                }
            },
            _ => panic!("Called iter on an unordered Ssa")
        }
    }
}

impl Deref for Ssa {
    type Target = SsaData;

    fn deref(&self) -> &Self::Target {
        match self {
            Ssa::Unordered { data } => data,
            Ssa::Ordered { data, .. } => data,
        }
    }
}

impl DerefMut for Ssa {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Ssa::Unordered { data } => data,
            Ssa::Ordered { data, .. } => data,
        }
    }
}


pub(crate) struct InstructionIterator<'a> {
    instructions: &'a FxHashMap<u32, Instruction>,
    instruction_index_iterator: Iter<'a, u32>
}
impl<'a> Iterator for InstructionIterator<'a> {
    type Item = (u32, &'a Instruction);

    fn next(&mut self) -> Option<Self::Item> {
        match self.instruction_index_iterator.next() {
            Some(i) => Some((*i, &self.instructions[i])),
            None => None
        }
    }
}