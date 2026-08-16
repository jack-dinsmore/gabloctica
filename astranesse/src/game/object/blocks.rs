use biscuit::Instructions;
use rustc_hash::FxHashMap;
use sorted_vec::SortedSet;
use std::ops::Index;
use lazy_static::lazy_static;

pub type BlockId = u16;
pub type GraphicId = u16;

include!(concat!(env!("OUT_DIR"), "/scripts.rs"));

pub struct BlockInfo {
    pub id: BlockId,
    pub graphic_id: GraphicId,
    pub script: Option<Instructions>,
    pub is_pipe: bool,
    pub is_conductor: bool,
    pub is_chair: bool,
    pub is_clutter: bool,
}

impl BlockInfo {
    pub fn from_line(line: &str) -> Self {
        let mut arguments = line.split_whitespace();
        let id = BlockId::from_str_radix(arguments.next().unwrap(), 16).unwrap();
        let graphic_id = GraphicId::from_str_radix(arguments.next().unwrap(), 16).unwrap();
        let name = arguments.next().unwrap().to_owned();
        let is_conductor = arguments.next().unwrap() == "T";
        let is_pipe = arguments.next().unwrap() == "T";
        let is_command = arguments.next().unwrap() == "T";
        let is_chair = arguments.next().unwrap() == "T";
        let is_clutter = arguments.next().unwrap() == "T";

        let script = match is_command {
            true => {
                let script = match COMPILED_SCRIPTS.get(&name.as_str()) {
                    Some(s) => s,
                    None => panic!("Could not find the script {}", name)
                };
                Some(Instructions::from_compiled(script))
            },
            false => None
        };

        Self {
            id,
            graphic_id,
            script,
            is_pipe,
            is_conductor,
            is_chair,
            is_clutter,
        }
    }
}

pub struct BlockProperties {
    blocks: FxHashMap<BlockId, BlockInfo>,
    pub command_blocks: SortedSet<BlockId>,
    pub conductor_blocks: SortedSet<BlockId>,
    pub pipe_blocks: SortedSet<BlockId>,
    pub chair_blocks: SortedSet<BlockId>,
}

impl BlockProperties {
    pub fn new() -> Self {
        let mut output = Self {
            blocks: FxHashMap::default(),
            command_blocks: SortedSet::new(),
            conductor_blocks: SortedSet::new(),
            pipe_blocks: SortedSet::new(),
            chair_blocks: SortedSet::new(),
        };
        output.load_manifest(include_str!("../../../../assets/blocks.txt"));
        output
    }

    pub fn load_manifest(&mut self, text: &str) {
        for line in text.split('\n') {
            if line.starts_with('#') { continue; }
            if line.is_empty() { continue; }
            let block = BlockInfo::from_line(line);
            
            if block.script.is_some() {
                self.command_blocks.push(block.id);
            }
            if block.is_conductor {
                self.conductor_blocks.push(block.id);
            }
            if block.is_pipe {
                self.pipe_blocks.push(block.id);
            }
            if block.is_chair {
                self.chair_blocks.push(block.id);
            }

            self.blocks.insert(block.id, block);
        }
    }
}

impl Index<BlockId> for BlockProperties {
    type Output = BlockInfo;

    fn index(&self, index: BlockId) -> &Self::Output {
        self.blocks.get(&index).unwrap()
    }
}