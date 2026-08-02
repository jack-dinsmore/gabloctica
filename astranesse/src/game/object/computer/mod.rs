use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;
use crate::{game::object::computer::compiler::compile, util::{Tagged, Vendor}};


pub struct BlockProperties {
    pub command_blocks: SortedSet<u8>,
    pub conductor_blocks: SortedSet<u8>,
    pub pipe_blocks: SortedSet<u8>,
    pub command_block_scripts: FxHashMap<u8, Instructions>,
    pub chair_blocks: SortedSet<u8>,
    preloaded_scripts: FxHashMap<String, &'static str>,
    script_vendor: Vendor<InstructionData>,
}

impl BlockProperties {
    pub fn new() -> Self {
        Self {
            command_blocks: SortedSet::default(),
            conductor_blocks: SortedSet::new(),
            pipe_blocks: SortedSet::new(),
            chair_blocks: SortedSet::new(),
            preloaded_scripts: FxHashMap::default(),
            command_block_scripts: FxHashMap::default(),
            script_vendor: Vendor::new(),
        }
    }

    pub fn preload_script(&mut self, script: &'static str, name: &str) {
        self.preloaded_scripts.insert(name.to_owned(), script);
    }

    pub fn load_manifest(&mut self, text: &str) {
        for line in text.split('\n') {
            if line.starts_with('#') { continue; }
            if line.is_empty() { continue; }
            let mut arguments = line.split_whitespace();
            let id: u8 = arguments.next().unwrap().parse().unwrap();
            let name = arguments.next().unwrap().to_owned();
            let is_conductor = arguments.next().unwrap() == "T";
            let is_pipe = arguments.next().unwrap() == "T";
            let is_command = arguments.next().unwrap() == "T";
            let is_chair = arguments.next().unwrap() == "T";

            if is_command {
                let data = InstructionData::new(name, &self.preloaded_scripts);
                let instructions = self.script_vendor.insert(data);
                self.command_block_scripts.insert(id, instructions);
            }

            if is_conductor { self.conductor_blocks.push(id); }
            if is_pipe { self.pipe_blocks.push(id); }
            if is_command { self.command_blocks.push(id); }
            if is_chair { self.chair_blocks.push(id); }
        }
    }
}

/*


    pub fn tick(&mut self, block: &mut CommandBlockInfo) -> Result<(), MachineError> {
        let mut message_index = 0;
        while let Some(function) = self.run_to_call()? {
            match function {
                0 => {// Print
                    let arg = self.stack.pop().ok_or(MachineError::Stack)?;
                    println!("Breakpoint {}", arg);
                },
                1 => {// Add force
                    let mut force = Vector3::new(
                        self.stack.pop().ok_or(MachineError::Stack)?,
                        self.stack.pop().ok_or(MachineError::Stack)?,
                        self.stack.pop().ok_or(MachineError::Stack)?
                    );
                    force = block.quat.rotate_vector(force);
                    let torque = block.pos.cross(force);
                    let force = block.body.ori.rotate_vector(force);
                    block.body.add_force(force);
                    block.body.add_torque(torque);
                },
                2 => {// Add torque
                    let torque = Vector3::new(self.stack.pop().ok_or(MachineError::Stack)?, self.stack.pop().ok_or(MachineError::Stack)?, self.stack.pop().ok_or(MachineError::Stack)?);
                    block.body.add_torque(torque);
                },
                3 => {// Emit signal
                    let recv_block = self.stack.pop().ok_or(MachineError::Stack)?.round() as u8;
                    let n_send = self.stack.pop().ok_or(MachineError::Stack)?.round() as usize;
                    
                    let mut data = Vec::with_capacity(n_send);
                    for _ in 0..n_send {
                        data.push(self.stack.pop().ok_or(MachineError::Stack)?);
                    }
                    
                    match &mut block.circuit {
                        Some(c) => {
                            c.send(recv_block, data)
                        },
                        None => (),
                    }
                },
                4 => {// Receive signal
                    let arg = self.stack.pop().ok_or(MachineError::Stack)?;
                    let jtrue = u64::from_le_bytes(self.stack.pop().ok_or(MachineError::Stack)?.to_le_bytes()) as usize;
                    let jfalse = u64::from_le_bytes(self.stack.pop().ok_or(MachineError::Stack)?.to_le_bytes()) as usize;
                    match &block.circuit {
                        Some(c) => {
                            let (data_option, new_message_index) = c.recv(block.id, message_index);
                            match data_option {
                                Some(data) => {
                                    for d in data {
                                        self.stack.push(*d);
                                    }
                                    self.stack.push(data.len() as f64);
                                    self.ip = jtrue;
                                },
                                None => {
                                    self.ip = jfalse;
                                }
                            }
                            message_index = new_message_index;
                        },
                        None => self.ip = jfalse,
                    }
                },
                _ => return Err(MachineError::Func)
            }
        }
        Ok(())
    } */