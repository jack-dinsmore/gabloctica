use cgmath::{Quaternion, Vector3};
use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;
use crate::{game::object::{Chunk, computer::{BlockProperties, machine::{Machine, MachineError}}}, graphics::CHUNK_SIZE, physics::RigidBody, util::{Tagged, Vendor}};

type Pipe = Tagged<PipeData>;
type Circuit = Tagged<CircuitData>;
pub type BlockKey = ((i32, i32, i32), (u32, u32, u32));

fn bucket_fill(chunks: &FxHashMap<(i32, i32, i32), Chunk>, targets: &SortedSet<u8>, commands: &SortedSet<u8>) -> FxHashMap<BlockKey, u32>{
    let mut attached = FxHashMap::default();
    let mut all_blocks = SortedSet::new();

    for (chunk_pos, chunk) in chunks {
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    let block = (*chunk_pos, (i,j,k));
                    let block_type = chunks[&block.0].grid[block.1].id;
                    if targets.contains(&block_type) { all_blocks.push(block); }
                }
            }
        }
    }

    let mut index = 0;
    while !all_blocks.is_empty() {
        let mut queue = vec![*all_blocks.first().unwrap()];
        while !queue.is_empty() {
            let block = queue.pop().unwrap();
            
            if commands.contains(&chunks[&block.0].grid[block.1].id) {
                attached.insert(block, index);
            }
            if let None = all_blocks.remove_item(&block) { continue; } // Already visited, or not a target.
            
            let new_coords = [
                if block.1.0 != 0 { (block.0, (block.1.0-1, block.1.1, block.1.2)) }
                else { ((block.0.0-1, block.0.1, block.0.2), (CHUNK_SIZE-1, block.1.1, block.1.2)) },

                if block.1.0 != CHUNK_SIZE-1 { (block.0, (block.1.0+1, block.1.1, block.1.2)) }
                else { ((block.0.0+1, block.0.1, block.0.2), (0, block.1.1, block.1.2)) },

                if block.1.1 != 0 { (block.0, (block.1.0, block.1.1-1, block.1.2)) }
                else { ((block.0.0, block.0.1-1, block.0.2), (block.1.0, CHUNK_SIZE-1, block.1.2)) },

                if block.1.1 != CHUNK_SIZE-1 { (block.0, (block.1.0, block.1.1+1, block.1.2)) }
                else { ((block.0.0, block.0.1+1, block.0.2), (block.1.0, 0, block.1.2)) },

                if block.1.2 != 0 { (block.0, (block.1.0, block.1.1, block.1.2-1)) }
                else { ((block.0.0, block.0.1, block.0.2-1), (block.1.0, block.1.1, CHUNK_SIZE-1)) },

                if block.1.2 != CHUNK_SIZE-1 { (block.0, (block.1.0, block.1.1, block.1.2+1)) }
                else { ((block.0.0, block.0.1, block.0.2+1), (block.1.0, block.1.1, 0)) },
            ];
            for c in new_coords {
                if chunks[&c.0].grid[c.1].id == 0 {continue;}
                queue.push(c);
            }
        }
        index += 1;
    }
    attached
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
        let attached_circuits = bucket_fill(chunks, &properties.conductor_blocks, &properties.command_blocks);
        let attached_pipes = bucket_fill(chunks, &properties.pipe_blocks, &properties.command_blocks);

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
        for mut circuit in &mut self.circuits.iter() {
            circuit.tick();
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

pub struct CircuitData {
    /// Messages are double buffered
    messages: [Vec<(u8, Vec<f64>)>; 2],
    up_index: usize,
}

impl CircuitData {
    pub fn new() -> Self {
        Self {
            messages: [Vec::new(), Vec::new()],
            up_index: 0,
        }
    }
    /// Send a message for to_block 
    pub fn send(&mut self, to_block: u8, data: Vec<f64>) {
        let down_messages = &mut self.messages[(self.up_index + 1) % 2];
        down_messages.push((to_block, data));
    }

    /// Receive a message for to_block. Returns the data if it exists, else None. Also return the index of the first unexplored message.
    pub fn recv(&self, to_block: u8, start_message: usize) -> (Option<&[f64]>, usize) {
        let up_messages = &self.messages[self.up_index];
        for (i, (m_to_block, m_data)) in up_messages.iter().skip(start_message).enumerate() {
            if *m_to_block != to_block {continue;}
            return (Some(m_data), i + 1)
        }
        (None, self.messages.len())
    }

    pub fn tick(&mut self) {
        self.messages[self.up_index].clear();
        self.up_index = (self.up_index + 1) % 2;
    }
}

pub struct PipeData {

}

impl PipeData {
    fn new() -> Self {
        Self {

        }
    }
}

pub struct CommandBlockInfo {
    pub(super) pipe: Option<Pipe>,
    pub(super) circuit: Option<Circuit>,
    pub(super) id: u8,
    pub(super) block: BlockKey,
    pub(super) body: RigidBody,
    pub(super) pos: Vector3<f64>,
    pub(super) quat: Quaternion<f64>,
}
pub struct CommandBlock {
    info: CommandBlockInfo,
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
            info: CommandBlockInfo {
                id: chunks[&block_pos.0].grid[block_pos.1].id,
                block: block_pos,
                pipe,
                circuit,
                body,
                pos,
                quat,
            },
            machine,
        }
    }

    fn update(&mut self, delta_t: f64) {
        let tick = self.machine.tick(&mut self.info);
        if let Err(e) = tick {
            match e {
                MachineError::Stack => println!("Segmentation fault in block (type {})", self.info.id),
                MachineError::Ip => println!("Program overflow in block (type {})", self.info.id),
                MachineError::Func => println!("Invalid function call in block (type {})", self.info.id),
                MachineError::OpCode => println!("Invalid opcode in block (type {})", self.info.id),
            }
            ;
            self.machine.reset();
        }
    }
}