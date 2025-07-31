use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet, VecDeque};
use crate::ast::Expression;
use crate::error::CompilerError;

// HashMap implementation (already exists, but enhanced)
pub struct HashMap<K, V> {
    inner: StdHashMap<K, V>,
}

impl HashMap<String, Expression> {
    pub fn new() -> Self {
        Self {
            inner: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: Expression) -> Option<Expression> {
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&Expression> {
        self.inner.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Expression> {
        self.inner.remove(key)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<String, Expression> {
        self.inner.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<String, Expression> {
        self.inner.values()
    }
}

// Vector implementation
pub struct Vector<T> {
    inner: Vec<T>,
}

impl Vector<Expression> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: Vec::with_capacity(capacity) }
    }

    pub fn push(&mut self, item: Expression) {
        self.inner.push(item);
    }

    pub fn pop(&mut self) -> Option<Expression> {
        self.inner.pop()
    }

    pub fn get(&self, index: usize) -> Option<&Expression> {
        self.inner.get(index)
    }

    pub fn set(&mut self, index: usize, value: Expression) -> Result<(), CompilerError> {
        if index < self.inner.len() {
            self.inner[index] = value;
            Ok(())
        } else {
            Err(CompilerError::runtime_error(&format!("Index {} out of bounds", index)))
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn insert(&mut self, index: usize, item: Expression) -> Result<(), CompilerError> {
        if index <= self.inner.len() {
            self.inner.insert(index, item);
            Ok(())
        } else {
            Err(CompilerError::runtime_error(&format!("Index {} out of bounds", index)))
        }
    }

    pub fn remove(&mut self, index: usize) -> Result<Expression, CompilerError> {
        if index < self.inner.len() {
            Ok(self.inner.remove(index))
        } else {
            Err(CompilerError::runtime_error(&format!("Index {} out of bounds", index)))
        }
    }
}

// HashSet implementation
pub struct HashSet<T> {
    inner: StdHashSet<T>,
}

impl HashSet<String> {
    pub fn new() -> Self {
        Self { inner: StdHashSet::new() }
    }

    pub fn insert(&mut self, item: String) -> bool {
        self.inner.insert(item)
    }

    pub fn remove(&mut self, item: &str) -> bool {
        self.inner.remove(item)
    }

    pub fn contains(&self, item: &str) -> bool {
        self.inner.contains(item)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<String> {
        self.inner.iter()
    }
}

// LinkedList implementation
pub struct LinkedList<T> {
    inner: Vec<T>, // Using Vec for simplicity, could be replaced with actual linked list
}

impl LinkedList<Expression> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push_front(&mut self, item: Expression) {
        self.inner.insert(0, item);
    }

    pub fn push_back(&mut self, item: Expression) {
        self.inner.push(item);
    }

    pub fn pop_front(&mut self) -> Option<Expression> {
        if !self.inner.is_empty() {
            Some(self.inner.remove(0))
        } else {
            None
        }
    }

    pub fn pop_back(&mut self) -> Option<Expression> {
        self.inner.pop()
    }

    pub fn front(&self) -> Option<&Expression> {
        self.inner.first()
    }

    pub fn back(&self) -> Option<&Expression> {
        self.inner.last()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

// Stack implementation
pub struct Stack<T> {
    inner: Vec<T>,
}

impl Stack<Expression> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, item: Expression) {
        self.inner.push(item);
    }

    pub fn pop(&mut self) -> Option<Expression> {
        self.inner.pop()
    }

    pub fn peek(&self) -> Option<&Expression> {
        self.inner.last()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

// Queue implementation
pub struct Queue<T> {
    inner: VecDeque<T>,
}

impl Queue<Expression> {
    pub fn new() -> Self {
        Self { inner: VecDeque::new() }
    }

    pub fn enqueue(&mut self, item: Expression) {
        self.inner.push_back(item);
    }

    pub fn dequeue(&mut self) -> Option<Expression> {
        self.inner.pop_front()
    }

    pub fn front(&self) -> Option<&Expression> {
        self.inner.front()
    }

    pub fn back(&self) -> Option<&Expression> {
        self.inner.back()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

// Builtin functions for collections
pub fn create_hashmap() -> HashMap<String, Expression> {
    HashMap::new()
}

pub fn hashmap_insert(mut map: HashMap<String, Expression>, key: String, value: Expression) -> HashMap<String, Expression> {
    map.insert(key, value);
    map
}

pub fn hashmap_get<'a>(map: &'a HashMap<String, Expression>, key: &'a str) -> Option<&'a Expression> {
    map.get(key)
}

pub fn create_vector() -> Vector<Expression> {
    Vector::new()
}

pub fn vector_push(mut vec: Vector<Expression>, item: Expression) -> Vector<Expression> {
    vec.push(item);
    vec
}

pub fn create_hashset() -> HashSet<String> {
    HashSet::new()
}

pub fn hashset_insert(mut set: HashSet<String>, item: String) -> HashSet<String> {
    set.insert(item);
    set
}

pub fn create_linkedlist() -> LinkedList<Expression> {
    LinkedList::new()
}

pub fn linkedlist_push_back(mut list: LinkedList<Expression>, item: Expression) -> LinkedList<Expression> {
    list.push_back(item);
    list
}

pub fn create_stack() -> Stack<Expression> {
    Stack::new()
}

pub fn stack_push(mut stack: Stack<Expression>, item: Expression) -> Stack<Expression> {
    stack.push(item);
    stack
}

pub fn create_queue() -> Queue<Expression> {
    Queue::new()
}

pub fn queue_enqueue(mut queue: Queue<Expression>, item: Expression) -> Queue<Expression> {
    queue.enqueue(item);
    queue
} 