#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Id {
    pub index: u32,
    pub generation: u32,
}

pub type SignalId = Id;
pub type NodeId = Id;
pub type ScopeId = Id;

struct ArenaEntry<T> {
    value: Option<T>,
    generation: u32,
}

pub struct Arena<T> {
    entries: Vec<ArenaEntry<T>>,
    free: Vec<u32>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> Id {
        if let Some(index) = self.free.pop() {
            let entry = &mut self.entries[index as usize];
            entry.value = Some(value);
            Id {
                index,
                generation: entry.generation,
            }
        } else {
            let index = self.entries.len() as u32;
            self.entries.push(ArenaEntry {
                value: Some(value),
                generation: 0,
            });
            Id {
                index,
                generation: 0,
            }
        }
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        let entry = self.entries.get(id.index as usize)?;
        if entry.generation != id.generation {
            return None;
        }
        entry.value.as_ref()
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        let entry = self.entries.get_mut(id.index as usize)?;
        if entry.generation != id.generation {
            return None;
        }
        entry.value.as_mut()
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let entry = self.entries.get_mut(id.index as usize)?;
        if entry.generation != id.generation {
            return None;
        }

        let value = entry.value.take()?;
        entry.generation = entry.generation.wrapping_add(1);
        self.free.push(id.index);

        Some(value)
    }
}
