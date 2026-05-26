use cgmath::{Quaternion, Rotation, Vector3};
use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;
use crate::{game::object::{Chunk, computer::{BlockProperties, machine::Machine}}, graphics::CHUNK_SIZE, physics::RigidBody, util::{Tagged, Vendor}};

type Pipe = Tagged<PipeData>;
type Circuit = Tagged<CircuitData>;
pub type BlockKey = ((i32, i32, i32), (u32, u32, u32));

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
    blocks: FxHashMap<BlockKey, CommandBlock>,
    circuits: Vendor<CircuitData>,
    pipes: Vendor<PipeData>,
}
impl Internals {
    pub fn new() -> Self {
        Self {
            circuits: Vendor::new(),
            pipes: Vendor::new(),
            blocks: FxHashMap::default(),
        }
    }

    pub fn update_info(&mut self, properties: &BlockProperties, chunks: &FxHashMap<(i32, i32, i32), Chunk>, body: RigidBody) {
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

                        self.blocks.insert(block, CommandBlock::new(block, chunks, properties, circuit, pipe, body.clone()));
                    }
                }
            }
        }
    }

    pub fn update(&mut self, delta_t: f64) {
        for block in self.blocks.values_mut() {
            block.update(delta_t);
        }
    }

    pub fn interrupt(&mut self, block: BlockKey, interrupt: Interrupt) {
        if let Some(b) = self.blocks.get_mut(&block) {
            // Call an interrupt on block b
            let irps = &mut b.machine.interrupts;
            match interrupt {
                Interrupt::Interact => irps.push(1.),
                Interrupt::Forward(f) => {irps.push(f); irps.push(2.);},
                Interrupt::Backward(f) => {irps.push(f); irps.push(3.);},
                Interrupt::Left(f) => {irps.push(f); irps.push(4.);},
                Interrupt::Right(f) => {irps.push(f); irps.push(5.);},
                Interrupt::Up(f) => {irps.push(f); irps.push(6.);},
                Interrupt::Down(f) => {irps.push(f); irps.push(7.);},
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Interrupt {
    Interact,
    Forward(f64),
    Backward(f64),
    Left(f64),
    Right(f64),
    Up(f64),
    Down(f64),
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
    body: RigidBody,
    pos: Vector3<f64>,
    quat: Quaternion<f64>,
    machine: Machine,
}

impl CommandBlock {
    fn new(block_pos: BlockKey, chunks: &FxHashMap<(i32, i32, i32), Chunk>, properties: &BlockProperties, circuit: Option<Circuit>, pipe: Option<Pipe>, body: RigidBody) -> Self {
        let block = chunks[&block_pos.0].grid[block_pos.1];
        let mut pos: Vector3<f64> = Vector3::new(block_pos.0.0 * CHUNK_SIZE as i32, block_pos.0.1 * CHUNK_SIZE as i32, block_pos.0.2 * CHUNK_SIZE as i32).cast().unwrap();
        pos += Vector3::new(block_pos.1.0 as f64 + 0.5, block_pos.1.1 as f64 + 0.5, block_pos.1.2 as f64 + 0.5);
        pos -= body.com_pos;
        let quat = block.quat();
        let machine = Machine::new(properties.command_block_scripts.get(&block.id).unwrap().clone());
        Self {
            block: block_pos,
            pipe,
            circuit,
            body,
            machine,
            pos,
            quat,
        }
    }

    fn update(&mut self, delta_t: f64) {
        if let None = self.machine.tick() {
            println!("Stack invalidation");
            self.machine.reset();
        }
        if let None = self.run_functions(delta_t) {
            println!("Function failure");
            self.machine.reset();
        }
    }

    fn run_functions(&mut self, delta_t: f64) -> Option<()> {
        while !self.machine.calls.is_empty() {
            let function = self.machine.calls.pop()?;
            match function {
                0. => {
                    let arg = self.machine.calls.pop()?;
                    println!("Breakpoint {}", arg);
                }
                1. => {
                    let mut force = Vector3::new(self.machine.calls.pop()?, self.machine.calls.pop()?, self.machine.calls.pop()?);
                    force = self.quat.rotate_vector(force);
                    let torque = self.pos.cross(force);
                    let force = self.body.ori.rotate_vector(force);
                    self.body.add_force(force);
                    self.body.add_torque(torque);
                }
                _ => panic!("Unrecognized function number"),
            }
        }
        Some(())
    }
}