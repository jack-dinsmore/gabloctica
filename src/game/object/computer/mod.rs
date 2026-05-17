use sorted_vec::SortedSet;

pub struct BlockProperties {
    pub command_blocks: SortedSet<u8>,
    pub conductor_blocks: SortedSet<u8>,
    pub pipe_blocks: SortedSet<u8>,
}

impl BlockProperties {
    pub fn new() -> Self {
        Self {
            command_blocks: SortedSet::new(),
            conductor_blocks: SortedSet::new(),
            pipe_blocks: SortedSet::new(),
        }
    }
    pub fn load_manifest(&mut self, text: &str) {
        for line in text.split('\n') {
            if line.starts_with('#') { continue; }
            if line.is_empty() { continue; }
            let mut arguments = line.split_whitespace();
            let id: u8 = arguments.next().unwrap().parse().unwrap();
            let is_conductor = arguments.next().unwrap() == "T";
            let is_pipe = arguments.next().unwrap() == "T";
            let is_command = arguments.next().unwrap() == "T";

            if is_conductor { self.conductor_blocks.push(id); }
            if is_pipe { self.pipe_blocks.push(id); }
            if is_command { self.command_blocks.push(id); }
        }
    }
}