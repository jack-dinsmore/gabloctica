use cgmath::Vector3;
use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;
use crate::{game::object::{Chunk, computer::{BlockProperties, machine::Machine}}, graphics::CHUNK_SIZE, util::{Tagged, Vendor}};

type Pipe = Tagged<PipeData>;
type Circuit = Tagged<CircuitData>;
type BlockKey = ((i32, i32, i32), (u32, u32, u32));

fn recursive_search(block: BlockKey, chunks: &FxHashMap<(i32, i32, i32), Chunk>, visited: &mut SortedSet<BlockKey>, attached: &mut FxHashMap<((i32, i32, i32), (u32, u32, u32)), u32>, conductors: &SortedSet<u8>, commands: &SortedSet<u8>, index: u32){
    // Check if block is visited
    if visited.contains(&block) { return; }
    visited.push(block);

    // Add the command block to the given circuit
    if commands.contains(&chunks[&block.0].grid[block.1].id) {
        if !attached.contains_key(&block) {
            attached.insert(block, index);
        }
    }

    // Check if block is a conductor
    if !conductors.contains(&chunks[&block.0].grid[block.1].id) {return;}
    
    let new_coords = [
        if block.1.0 != 0 {
            (block.0, (block.1.0-1, block.1.1, block.1.2))
        } else {
            ((block.0.0-1, block.0.1, block.0.2), (CHUNK_SIZE-1, block.1.1, block.1.2))
        },

        if block.1.0 != CHUNK_SIZE-1 {
            (block.0, (block.1.0+1, block.1.1, block.1.2))
        } else {
            ((block.0.0+1, block.0.1, block.0.2), (0, block.1.1, block.1.2))
        },

        if block.1.1 != 0 {
            (block.0, (block.1.0, block.1.1-1, block.1.2))
        } else {
            ((block.0.0, block.0.1-1, block.0.2), (block.1.0, CHUNK_SIZE-1, block.1.2))
        },

        if block.1.1 != CHUNK_SIZE-1 {
            (block.0, (block.1.0, block.1.1+1, block.1.2))
        } else {
            ((block.0.0, block.0.1+1, block.0.2), (block.1.0, 0, block.1.2))
        },

        if block.1.2 != 0 {
            (block.0, (block.1.0, block.1.1, block.1.2-1))
        } else {
            ((block.0.0, block.0.1, block.0.2-1), (block.1.0, block.1.1, CHUNK_SIZE-1))
        },

        if block.1.2 != CHUNK_SIZE-1 {
            (block.0, (block.1.0, block.1.1, block.1.2+1))
        } else {
            ((block.0.0, block.0.1, block.0.2+1), (block.1.0, block.1.1, 0))
        },
    ];
    for c in new_coords {
        recursive_search(c, chunks, visited, attached, conductors, commands, index);
    }
}

pub struct Internals {
    blocks: Vec<CommandBlock>,
    circuits: Vendor<CircuitData>,
    pipes: Vendor<PipeData>,
}
impl Internals {
    pub fn new() -> Self {
        Self {
            circuits: Vendor::new(),
            pipes: Vendor::new(),
            blocks: Vec::new(),
        }
    }

    pub fn update_info(&mut self, chunks: &FxHashMap<(i32, i32, i32), Chunk>, properties: &BlockProperties) {
        let mut attached_circuits = FxHashMap::default();
        let mut attached_pipes = FxHashMap::default();
        let mut visited = SortedSet::new();
        let mut index = 0;

        // Get all adjoining circuits
        for (chunk_pos, chunk) in chunks {
            for i in 0..CHUNK_SIZE {
                for j in 0..CHUNK_SIZE {
                    for k in 0..CHUNK_SIZE {
                        let block = (*chunk_pos, (i,j,k));
                        if visited.contains(&block) { continue; }
                        recursive_search(block, chunks, &mut visited, &mut attached_circuits, &properties.conductor_blocks, &properties.command_blocks, index);
                        index += 1;
                    }
                }
            }
        }

        // Get all adjoining pipes
        visited.clear();
        index = 0;
        for (chunk_pos, chunk) in chunks {
            for i in 0..CHUNK_SIZE {
                for j in 0..CHUNK_SIZE {
                    for k in 0..CHUNK_SIZE {
                        let block = (*chunk_pos, (i,j,k));
                        if visited.contains(&block) { continue; }
                        recursive_search(block, chunks, &mut visited, &mut attached_pipes, &properties.pipe_blocks, &properties.command_blocks, index);
                        index += 1;
                    }
                }
            }
        }

        let mut circuit_structs: FxHashMap<u32, Circuit> = FxHashMap::default();
        let mut pipe_structs: FxHashMap<u32, Pipe>  = FxHashMap::default();
        for (chunk_pos, chunk) in chunks {
            for i in 0..CHUNK_SIZE {
                for j in 0..CHUNK_SIZE {
                    for k in 0..CHUNK_SIZE {
                        let block = (*chunk_pos, (i,j,k));
                        if !properties.command_blocks.contains(&chunks[&block.0].grid[block.1].id) { continue; }

                        let circuit = match attached_circuits.get(&block) {
                            Some(index) => match circuit_structs.get(index) {
                                Some(circuit_struct) => Some(circuit_struct.clone()),
                                None => {
                                    let circuit = self.circuits.insert(CircuitData::new());
                                    circuit_structs.insert(*index, circuit.clone());
                                    Some(circuit)
                                }
                            },
                            None => None
                        };
                        let pipe = match attached_pipes.get(&block) {
                            Some(index) => match pipe_structs.get(index) {
                                Some(pipe_struct) => Some(pipe_struct.clone()),
                                None => {
                                    let pipe = self.pipes.insert(PipeData::new());
                                    pipe_structs.insert(*index, pipe.clone());
                                    Some(pipe)
                                }
                            },
                            None => None
                        };

                        self.blocks.push(CommandBlock::new(block, chunks, circuit, pipe, properties));
                    }
                }
            }
        }
    }

    pub fn update(&mut self, delta_t: f64) {
        for block in &mut self.blocks {
            block.update(delta_t);
        }
    }
}

struct CircuitData {

}

impl CircuitData {
    fn new() -> Self {
        Self {
            
        }
    }
}

struct PipeData {

}

impl PipeData {
    fn new() -> Self {
        Self {

        }
    }
}

struct CommandBlock {
    block: BlockKey,
    pipe: Option<Pipe>,
    circuit: Option<Circuit>,
    machine: Machine,
}

impl CommandBlock {
    fn new(block: BlockKey, chunks: &FxHashMap<(i32, i32, i32), Chunk>, circuit: Option<Circuit>, pipe: Option<Pipe>, properties: &BlockProperties) -> Self {
        let block_type = chunks[&block.0].grid[block.1].id;
        let machine = Machine::new(properties.command_block_scripts.get(&block_type).unwrap().clone());
        Self {
            block,
            pipe,
            circuit,
            machine,
        }
    }

    fn update(&mut self, delta_t: f64) {
        self.machine.tick();
        while !self.machine.calls.is_empty() {
            let function = self.machine.calls.pop().unwrap();
            match function {
                0. => {
                    let arg = self.machine.calls.pop().unwrap();
                    println!("Breakpoint {}", arg);
                }
                1. => {
                    let force = Vector3::new(self.machine.calls.pop().unwrap(), self.machine.calls.pop().unwrap(), self.machine.calls.pop().unwrap());
                    // TODO
                }
                _ => panic!("Unrecognized function number"),
            }
        }
    }
}