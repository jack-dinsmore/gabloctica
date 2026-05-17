mod interp;

use std::{cell::RefCell, rc::Rc};

pub use interp::SphericalInterpolator;
use rustc_hash::FxHashMap;
pub type RcCell<T> = Rc<RefCell<T>>;
pub type Vendor<D> = Box<TagVendor<D>>;

pub fn new_vendor<D>() -> Vendor<D> {
    Box::new(TagVendor::new())
}

#[derive(Debug)]
pub struct Tagged<D> {
    vendor: *mut TagVendor<D>,
    tag: u32,
}

pub struct TagVendor<D> {
    data: FxHashMap<u32, D>,
    borrows: FxHashMap<u32, u32>,
}
impl<D> TagVendor<D> {
    pub fn new() -> Self {
        Self {
            data: FxHashMap::default(),
            borrows: FxHashMap::default(),
        }
    }

    pub fn insert(&mut self, data: D) -> Tagged<D> {
        let tag = self.new_tag();
        self.data.insert(tag, data);
        self.borrows.insert(tag, 1);
        Tagged {
            vendor: self as *mut Self,
            tag
        }
    }

    fn remove(&mut self, tag: u32) {
        self.data.remove(&tag);
        self.borrows.remove(&tag);
    }

    fn new_tag(&self) -> u32 {
        let mut tag = 0;
        loop {
            if self.data.contains_key(&tag) {
                tag += 1;
            } else {
                break tag;
            }
        }
    }

    pub fn iter<'a>(&'a mut self) -> TagVendorIterator<D> {
        let pointer = self as *mut _;
        TagVendorIterator {
            tags: self.data.keys().copied().collect(),
            index: 0,
            pointer,
        }
    }
}

pub struct TagVendorIterator<D> {
    tags: Vec<u32>,
    index: usize,
    pointer: *mut TagVendor<D>,
}
impl<D> Iterator for TagVendorIterator<D> {
    type Item = Tagged<D>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tags.len() {
            None
        } else {
            let tag = self.tags[self.index];
            {
                let vendor = unsafe { &mut (*self.pointer) };
                *vendor.borrows.get_mut(&tag).unwrap() += 1;
            }
            let output = Some(Tagged { vendor: self.pointer, tag });
            self.index += 1;
            output
        }
    }
}

impl<D> Clone for Tagged<D> {
    fn clone(&self) -> Self {
        let vendor = unsafe { &mut (*self.vendor) };
        *vendor.borrows.get_mut(&self.tag).unwrap() += 1;
        Self {
            vendor: self.vendor.clone(),
            tag: self.tag.clone()
        }
    }
}
impl<D> Drop for Tagged<D> {
    fn drop(&mut self) {
        let vendor = unsafe { &mut (*self.vendor) };
        *vendor.borrows.get_mut(&self.tag).unwrap() -= 1;
        if vendor.borrows[&self.tag] == 0 {
            vendor.remove(self.tag)
        }
    }
}

impl<D> std::ops::Deref for Tagged<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        let vendor = unsafe { &(*self.vendor) };
        vendor.data.get(&self.tag).unwrap()
    }
}
impl<D> std::ops::DerefMut for Tagged<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let vendor = unsafe { &mut (*self.vendor) };
        vendor.data.get_mut(&self.tag).unwrap()
    }
}
impl<D> PartialEq for Tagged<D> {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}
impl<D> Eq for Tagged<D> {}
impl<D> PartialOrd for Tagged<D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag.partial_cmp(&other.tag)
    }
}