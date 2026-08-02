use rustc_hash::FxHashMap;

#[derive(Clone)]
pub struct Memory {
    vector_map: FxHashMap<u32, Vec<f64>>,
    next_address: u32,
}
impl Memory {
    pub fn new() -> Self {
        Self {
            vector_map: FxHashMap::default(),
            next_address: 0,
        }
    }

    pub fn allocate(&mut self) -> u32 {
        let address = self.next_address;
        self.next_address += 1;
        self.vector_map.insert(address, Vec::new());
        address
    }

    pub fn drop(&mut self, address: u32) -> Option<()> {
        self.vector_map.remove(&(address as u32)).map(|_| ())
    }

    pub fn load(&self, address: u32, index: u32) -> Option<f64> {
        let index = index as usize;
        match self.vector_map.get(&address) {
            Some(v) => v.get(index).copied(),
            None => None,
        }
    }

    pub fn access<'a>(&'a self, address: u32) -> Option<&'a [f64]> {
        match self.vector_map.get(&address) {
            Some(v) => Some(v),
            None => None,
        }
    }

    pub fn store(&mut self, address: u32, index: u32, item: f64) -> Option<()> {
        let index = index as usize;
        match self.vector_map.get_mut(&address) {
            Some(v) => {
                if v.len() > index {
                    v[index] = item;
                } else if v.len() == index {
                    v.push(item);
                } else {
                    return None
                }
                Some(())
            },
            None => None,
        }
    }

    pub fn store_back(&mut self, address: u32, item: f64) -> Option<()> {
        match self.vector_map.get_mut(&address) {
            Some(v) => {
                v.push(item);
                Some(())
            },
            None => None,
        }
    }
}