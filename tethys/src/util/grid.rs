use std::{mem::MaybeUninit, ops::{Index, IndexMut}};

pub const CHUNK_SIZE: u32 = 32;

pub struct Grid<T> {
    data: [T; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize],
}

impl<T: Default + Copy> Grid<T> {
    pub fn new() -> Self {
        let data = [T::default(); (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize];

        Self {
            data
        }
    }
}
impl<T> Grid<T> {
    pub fn from_data(data: [T; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize]) -> Self {
        Self {
            data
        }
    }
    
    pub fn from_fn(func: impl Fn(u32, u32, u32) -> T) -> Grid<T> {
        let mut data: [MaybeUninit<T>; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize] = [const { MaybeUninit::uninit() }; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize];
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    data[(i + j*CHUNK_SIZE + k*CHUNK_SIZE*CHUNK_SIZE) as usize].write(func(i,j,k));
                }
            }
        }
        let data = unsafe { (&data as *const _ as *const [T; (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) as usize]).read() };
        Self {
            data,
        }
    }
}

impl<T: Clone> Grid<T> {
    pub fn fill(&mut self, item: T) {
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    self[(i,j,k)] = item.clone();
                }
            }
        }
    }
}

impl<T> Index<(u32, u32, u32)> for Grid<T> {
    type Output = T;

    fn index(&self, index: (u32, u32, u32)) -> &Self::Output {
        &self.data[(index.0 + index.1*CHUNK_SIZE + index.2*CHUNK_SIZE*CHUNK_SIZE) as usize]
    }
}

impl<T> IndexMut<(u32, u32, u32)> for Grid<T> {
    fn index_mut(&mut self, index: (u32, u32, u32)) -> &mut Self::Output {
        &mut self.data[(index.0 + index.1*CHUNK_SIZE + index.2*CHUNK_SIZE*CHUNK_SIZE) as usize]
    }
}