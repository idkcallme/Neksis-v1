use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub enum MemoryValue {
    Owned(Box<VMValueCore>),
    Shared(Arc<VMValueCore>),
    Borrowed(Arc<RefCell<VMValueCore>>),
    Weak(std::sync::Weak<VMValueCore>),
}

#[derive(Debug, Clone)]
pub enum VMValueCore {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<MemoryValue>),
    Object(HashMap<String, MemoryValue>),
    Null,
}

pub struct MemoryManager {
    heap: Vec<MemoryValue>,
    free_slots: Vec<usize>,
    allocation_count: usize,
    deallocation_count: usize,
    peak_memory: usize,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            free_slots: Vec::new(),
            allocation_count: 0,
            deallocation_count: 0,
            peak_memory: 0,
        }
    }
    
    pub fn allocate(&mut self, value: VMValueCore) -> usize {
        self.allocation_count += 1;
        
        let memory_value = MemoryValue::Owned(Box::new(value));
        
        if let Some(slot) = self.free_slots.pop() {
            self.heap[slot] = memory_value;
            slot
        } else {
            self.heap.push(memory_value);
            let index = self.heap.len() - 1;
            self.peak_memory = self.peak_memory.max(self.heap.len());
            index
        }
    }
    
    pub fn deallocate(&mut self, index: usize) {
        if index < self.heap.len() {
            self.heap[index] = MemoryValue::Owned(Box::new(VMValueCore::Null));
            self.free_slots.push(index);
            self.deallocation_count += 1;
        }
    }
    
    pub fn get(&self, index: usize) -> Option<&MemoryValue> {
        self.heap.get(index)
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut MemoryValue> {
        self.heap.get_mut(index)
    }
    
    pub fn create_shared(&mut self, value: VMValueCore) -> Arc<VMValueCore> {
        Arc::new(value)
    }
    
    pub fn create_borrowed(&mut self, value: VMValueCore) -> Arc<RefCell<VMValueCore>> {
        Arc::new(RefCell::new(value))
    }
    
    // Automatic memory optimization
    pub fn optimize_memory(&mut self) {
        // Compact heap by moving values to fill free slots
        let mut write_index = 0;
        let mut read_index = 0;
        
        while read_index < self.heap.len() {
            if !matches!(self.heap[read_index], MemoryValue::Owned(ref boxed) 
                        if matches!(**boxed, VMValueCore::Null)) {
                if write_index != read_index {
                    self.heap[write_index] = self.heap[read_index].clone();
                }
                write_index += 1;
            }
            read_index += 1;
        }
        
        self.heap.truncate(write_index);
        self.free_slots.clear();
    }
    
    pub fn get_stats(&self) -> String {
        format!(
            "Memory Stats: {} allocated, {} deallocated, {} active, peak: {}",
            self.allocation_count,
            self.deallocation_count,
            self.heap.len() - self.free_slots.len(),
            self.peak_memory
        )
    }
    
    // Smart garbage collection
    pub fn collect_garbage(&mut self) {
        // Simple mark-and-sweep for now
        let mut marked = vec![false; self.heap.len()];
        
        // Mark phase - mark all reachable objects
        for (i, value) in self.heap.iter().enumerate() {
            match value {
                MemoryValue::Shared(arc) => {
                    if Arc::strong_count(arc) > 1 {
                        marked[i] = true;
                    }
                }
                MemoryValue::Borrowed(arc) => {
                    if Arc::strong_count(arc) > 1 {
                        marked[i] = true;
                    }
                }
                MemoryValue::Owned(_) => {
                    marked[i] = true; // Keep owned values
                }
                MemoryValue::Weak(weak) => {
                    if weak.strong_count() > 0 {
                        marked[i] = true;
                    }
                }
            }
        }
        
        // Sweep phase - deallocate unmarked objects
        for (i, is_marked) in marked.iter().enumerate() {
            if !is_marked && i < self.heap.len() {
                self.deallocate(i);
            }
        }
    }
}

// Convert between our new memory system and the existing VM system
impl From<crate::vm::VMValue> for VMValueCore {
    fn from(vm_value: crate::vm::VMValue) -> Self {
        match vm_value {
            crate::vm::VMValue::Int(i) => VMValueCore::Int(i),
            crate::vm::VMValue::Float(f) => VMValueCore::Float(f),
            crate::vm::VMValue::String(s) => VMValueCore::String(s),
            crate::vm::VMValue::Bool(b) => VMValueCore::Bool(b),
            crate::vm::VMValue::Array(arr) => {
                let converted: Vec<MemoryValue> = arr.into_iter()
                    .map(|v| MemoryValue::Owned(Box::new(VMValueCore::from(v))))
                    .collect();
                VMValueCore::Array(converted)
            }
            crate::vm::VMValue::Object(map) => {
                let converted: HashMap<String, MemoryValue> = map.into_iter()
                    .map(|(k, v)| (k, MemoryValue::Owned(Box::new(VMValueCore::from(v)))))
                    .collect();
                VMValueCore::Object(converted)
            }
            _ => VMValueCore::Null,
        }
    }
}

impl From<VMValueCore> for crate::vm::VMValue {
    fn from(core_value: VMValueCore) -> Self {
        match core_value {
            VMValueCore::Int(i) => crate::vm::VMValue::Int(i),
            VMValueCore::Float(f) => crate::vm::VMValue::Float(f),
            VMValueCore::String(s) => crate::vm::VMValue::String(s),
            VMValueCore::Bool(b) => crate::vm::VMValue::Bool(b),
            VMValueCore::Array(arr) => {
                let converted: Vec<crate::vm::VMValue> = arr.into_iter()
                    .map(|mem_val| match mem_val {
                        MemoryValue::Owned(boxed) => crate::vm::VMValue::from(*boxed),
                        _ => crate::vm::VMValue::Null,
                    })
                    .collect();
                crate::vm::VMValue::Array(converted)
            }
            VMValueCore::Object(map) => {
                let converted: HashMap<String, crate::vm::VMValue> = map.into_iter()
                    .map(|(k, mem_val)| (k, match mem_val {
                        MemoryValue::Owned(boxed) => crate::vm::VMValue::from(*boxed),
                        _ => crate::vm::VMValue::Null,
                    }))
                    .collect();
                crate::vm::VMValue::Object(converted)
            }
            VMValueCore::Null => crate::vm::VMValue::Null,
        }
    }
}
