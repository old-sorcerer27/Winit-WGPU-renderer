use std::marker::PhantomData;

use slotmap::SlotMap;

use super::{Handle, Resource};

pub struct Storage<T: Resource> {
    slotmap: SlotMap<T::Key, T>,
}

impl<T: Resource> Default for Storage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Resource> Storage<T> {
    pub fn new() -> Self {
        Self {
            slotmap: SlotMap::with_key(),
        }
    }

    pub fn debug_stats(&self) -> String {
        format!(
            "Storage<{}>: {} resources, {} capacity",
            std::any::type_name::<T>(),
            self.slotmap.len(),
            self.slotmap.capacity()
        )
    }
    
    pub fn load(&mut self, params: T::LoadParams) -> Result<Handle<T>, Box<dyn std::error::Error>> {
        let resource = T::load(params)?;
        let key = self.slotmap.insert(resource);
        Ok(Handle {
            key,
            _phantom: PhantomData,
        })
    }

    pub fn load_all<I>(&mut self, params_iter: I) -> Result<Vec<Handle<T>>, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = T::LoadParams>
    {
        params_iter.into_iter().map(|p| self.load(p)).collect()
    }
    
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.slotmap.get(handle.key)
    }
    
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.slotmap.get_mut(handle.key)
    }
    
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        self.slotmap.remove(handle.key)
    }
    
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.slotmap.contains_key(handle.key)
    }
}



