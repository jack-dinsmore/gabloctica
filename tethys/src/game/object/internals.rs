use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;
use crate::{game::object::{BlockProperties, blocks::BlockId, chunk::{PhysicalBlock, PhysicalChunk}}, util::CHUNK_SIZE};
use biscuit::{MachineOutput, Machine, MachineError};
use crate::util::{Tagged, Vendor};

type PhysicalPipe = Tagged<PipeData>;
type PhysicalCircuit = Tagged<CircuitData>;
pub type BlockGlobalPos = ((i32, i32, i32), (u32, u32, u32));

const MAX_LINES_PER_TICK: usize = 1000;

/// Controls the internal logic which is not connected to individual blocks
pub struct Internals {
    command_blocks: FxHashMap<BlockGlobalPos, CommandBlock>,
    circuits: Vendor<CircuitData>,
    pipes: Vendor<PipeData>,
}
impl Internals {
    pub fn new() -> Self {
        Self {
            circuits: Vendor::new(),
            pipes: Vendor::new(),
            command_blocks: FxHashMap::default(),
        }
    }

    pub fn get_network(&mut self, properties: &BlockProperties, chunks: &FxHashMap<(i32, i32, i32), PhysicalChunk>) {
        // Get the networks
        let attached_circuits = bucket_fill(chunks, &properties.conductor_blocks, &properties.command_blocks);
        let attached_pipes = bucket_fill(chunks, &properties.pipe_blocks, &properties.command_blocks);

        // Assign the command blocks
        let mut circuit_structs: FxHashMap<u32, PhysicalCircuit> = FxHashMap::default();
        let mut pipe_structs: FxHashMap<u32, PhysicalPipe>  = FxHashMap::default();
        for (chunk_pos, chunk) in chunks {
            for i in 0..CHUNK_SIZE {
                for j in 0..CHUNK_SIZE {
                    for k in 0..CHUNK_SIZE {
                        let block_pos = (*chunk_pos, (i,j,k));
                        let block = chunks[&block_pos.0].grid[block_pos.1];
                        if block.is_null() {continue;}
                        if properties[block.id].script.is_none() { continue; }


                        let circuit = match attached_circuits.get(&block_pos) {
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
                        let pipe = match attached_pipes.get(&block_pos) {
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

                        self.command_blocks.insert(block_pos, CommandBlock::new(block_pos, chunks, properties, circuit, pipe));
                    }
                }
            }
        }
    }

    pub fn place_block(&mut self, block: &PhysicalBlock, block_properties: &BlockProperties, chunks: &FxHashMap<(i32, i32, i32), PhysicalChunk>) {
        // OPTIMIZE
        self.get_network(block_properties, chunks);
    }

    pub fn update(&mut self, delta_t: f64) {
        for block in self.command_blocks.values_mut() {
            block.update(delta_t);
        }
        for mut circuit in &mut self.circuits.iter() {
            circuit.tick();
        }
    }

    pub fn interrupt(&mut self, block: BlockGlobalPos, interrupt: Interrupt) {
        if let Some(b) = self.command_blocks.get_mut(&block) {
            // Call an interrupt on block b
            b.interrupts.push(interrupt);
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

pub struct CommandBlock {
    pub(super) id: BlockId,
    pub(super) pipe: Option<PhysicalPipe>,
    pub(super) circuit: Option<PhysicalCircuit>,
    pub(super) pos: BlockGlobalPos,
    machine: Machine,
    interrupts: Vec<Interrupt>,
}

impl CommandBlock {
    fn new(block_pos: BlockGlobalPos, chunks: &FxHashMap<(i32, i32, i32), PhysicalChunk>, properties: &BlockProperties, circuit: Option<PhysicalCircuit>, pipe: Option<PhysicalPipe>) -> Self {
        let block = chunks[&block_pos.0].grid[block_pos.1];
        let machine = Machine::new(
            properties[block.id].script.clone().unwrap(),
            MAX_LINES_PER_TICK
        );
        Self {
            id: chunks[&block_pos.0].grid[block_pos.1].id,
            pos: block_pos,
            pipe,
            circuit,
            machine,
            interrupts: Vec::new(),
        }
    }

    fn update(&mut self, delta_t: f64) {
        match self.machine.run_to_call() {
            Err(e) => {
                match e {
                    MachineError::Stack => println!("Segmentation fault in block (type {})", self.id),
                    MachineError::Memory => println!("Memory overflow"),
                    MachineError::Ip => println!("Program overflow in block (type {})", self.id),
                    MachineError::Func => println!("Invalid function call in block (type {})", self.id),
                    MachineError::OpCode => println!("Invalid opcode in block (type {})", self.id),
                };
                self.machine.reset();
            },
            Ok(MachineOutput::Call {func, args}) => match func {
                biscuit::GlobalFunction::Dbg => { dbg!(&args); },
                biscuit::GlobalFunction::Tick => {  },
                biscuit::GlobalFunction::Interrupt => { dbg!(&args); },
            }
            Ok(MachineOutput::None) => ()
        }
    }
}

fn bucket_fill(chunks: &FxHashMap<(i32, i32, i32), PhysicalChunk>, targets: &SortedSet<BlockId>, commands: &SortedSet<BlockId>) -> FxHashMap<BlockGlobalPos, u32>{
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