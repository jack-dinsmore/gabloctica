pub mod compiler;
pub mod machine;

use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;
use crate::{game::object::computer::compiler::compile, util::{Tagged, Vendor}};
pub type Instructions = Tagged<InstructionData>;

pub struct InstructionData {
    name: String,
    instructions: Vec<u8>,
}
impl InstructionData {
    fn new(name: String, loaded: &FxHashMap<String, &'static str>) -> Self {
        let text = match loaded.get(&name) {
            Some(t) => t,
            None => {
                unimplemented!()
            }
        };
        // Process the script
        let instructions = compile(text);
        Self {
            name,
            instructions,
        }
    }
}

pub struct BlockProperties {
    pub command_blocks: SortedSet<u8>,
    pub conductor_blocks: SortedSet<u8>,
    pub pipe_blocks: SortedSet<u8>,
    pub command_block_scripts: FxHashMap<u8, Instructions>,
    preloaded_scripts: FxHashMap<String, &'static str>,
    script_vendor: Vendor<InstructionData>,
}

impl BlockProperties {
    pub fn new() -> Self {
        Self {
            command_blocks: SortedSet::default(),
            conductor_blocks: SortedSet::new(),
            pipe_blocks: SortedSet::new(),
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

            if is_command {
                let data = InstructionData::new(name, &self.preloaded_scripts);
                let instructions = self.script_vendor.insert(data);
                self.command_block_scripts.insert(id, instructions);
            }

            if is_conductor { self.conductor_blocks.push(id); }
            if is_pipe { self.pipe_blocks.push(id); }
            if is_command { self.command_blocks.push(id); }
        }
    }
}