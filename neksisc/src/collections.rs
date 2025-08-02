// Collections module for Neksis 2025
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque};
use std::hash::Hash;
use crate::modern_stdlib::{NeksisError, NeksisResult};

/// Dynamic Array (Vector)
#[derive(Debug, Clone, PartialEq)]
pub struct NeksisVec<T> {
    data: Vec<T>,
}

impl<T> NeksisVec<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }
    
    pub fn push(&mut self, item: T) {
        self.data.push(item);
    }
    
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }
    
    pub fn get(&self, index: usize) -> NeksisResult<&T> {
        self.data.get(index)
            .ok_or_else(|| NeksisError::IndexOutOfBounds(
                format!("Index {} out of bounds for vector of length {}", index, self.data.len())
            ))
    }
    
    pub fn get_mut(&mut self, index: usize) -> NeksisResult<&mut T> {
        let len = self.data.len();
        self.data.get_mut(index)
            .ok_or_else(|| NeksisError::IndexOutOfBounds(
                format!("Index {} out of bounds for vector of length {}", index, len)
            ))
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.data.contains(item)
    }
    
    pub fn insert(&mut self, index: usize, item: T) -> NeksisResult<()> {
        if index > self.data.len() {
            return Err(NeksisError::IndexOutOfBounds(
                format!("Index {} out of bounds for insert on vector of length {}", index, self.data.len())
            ));
        }
        self.data.insert(index, item);
        Ok(())
    }
    
    pub fn remove(&mut self, index: usize) -> NeksisResult<T> {
        if index >= self.data.len() {
            return Err(NeksisError::IndexOutOfBounds(
                format!("Index {} out of bounds for vector of length {}", index, self.data.len())
            ));
        }
        Ok(self.data.remove(index))
    }
    
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }
    
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.data.sort();
    }
    
    pub fn reverse(&mut self) {
        self.data.reverse();
    }
}

impl<T> From<Vec<T>> for NeksisVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self { data: vec }
    }
}

impl<T> Into<Vec<T>> for NeksisVec<T> {
    fn into(self) -> Vec<T> {
        self.data
    }
}

/// Hash Map
#[derive(Debug, Clone)]
pub struct NeksisHashMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    data: HashMap<K, V>,
}

impl<K, V> PartialEq for NeksisHashMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<K, V> NeksisHashMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.data.insert(key, value)
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }
    
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.data.get_mut(key)
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }
    
    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    pub fn keys(&self) -> std::collections::hash_map::Keys<K, V> {
        self.data.keys()
    }
    
    pub fn values(&self) -> std::collections::hash_map::Values<K, V> {
        self.data.values()
    }
    
    pub fn iter(&self) -> std::collections::hash_map::Iter<K, V> {
        self.data.iter()
    }
}

/// Hash Set
#[derive(Debug, Clone)]
pub struct NeksisHashSet<T>
where
    T: Eq + Hash,
{
    data: HashSet<T>,
}

impl<T> NeksisHashSet<T>
where
    T: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            data: HashSet::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashSet::with_capacity(capacity),
        }
    }
    
    pub fn insert(&mut self, value: T) -> bool {
        self.data.insert(value)
    }
    
    pub fn remove(&mut self, value: &T) -> bool {
        self.data.remove(value)
    }
    
    pub fn contains(&self, value: &T) -> bool {
        self.data.contains(value)
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    pub fn iter(&self) -> std::collections::hash_set::Iter<T> {
        self.data.iter()
    }
    
    pub fn union(&self, other: &NeksisHashSet<T>) -> NeksisHashSet<T>
    where
        T: Clone,
    {
        let union_set: HashSet<T> = self.data.union(&other.data).cloned().collect();
        NeksisHashSet { data: union_set }
    }
    
    pub fn intersection(&self, other: &NeksisHashSet<T>) -> NeksisHashSet<T>
    where
        T: Clone,
    {
        let intersection_set: HashSet<T> = self.data.intersection(&other.data).cloned().collect();
        NeksisHashSet { data: intersection_set }
    }
    
    pub fn difference(&self, other: &NeksisHashSet<T>) -> NeksisHashSet<T>
    where
        T: Clone,
    {
        let difference_set: HashSet<T> = self.data.difference(&other.data).cloned().collect();
        NeksisHashSet { data: difference_set }
    }
}

/// Deque (Double-ended queue)
#[derive(Debug, Clone, PartialEq)]
pub struct NeksisDeque<T> {
    data: VecDeque<T>,
}

impl<T> NeksisDeque<T> {
    pub fn new() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
        }
    }
    
    pub fn push_front(&mut self, item: T) {
        self.data.push_front(item);
    }
    
    pub fn push_back(&mut self, item: T) {
        self.data.push_back(item);
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        self.data.pop_front()
    }
    
    pub fn pop_back(&mut self) -> Option<T> {
        self.data.pop_back()
    }
    
    pub fn front(&self) -> Option<&T> {
        self.data.front()
    }
    
    pub fn back(&self) -> Option<&T> {
        self.data.back()
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    pub fn get(&self, index: usize) -> NeksisResult<&T> {
        self.data.get(index)
            .ok_or_else(|| NeksisError::IndexOutOfBounds(
                format!("Index {} out of bounds for deque of length {}", index, self.data.len())
            ))
    }
}

/// Ordered Map (BTreeMap)
#[derive(Debug, Clone, PartialEq)]
pub struct NeksisOrderedMap<K, V> {
    data: BTreeMap<K, V>,
}

impl<K, V> NeksisOrderedMap<K, V>
where
    K: Ord,
{
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.data.insert(key, value)
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }
    
    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    pub fn keys(&self) -> std::collections::btree_map::Keys<K, V> {
        self.data.keys()
    }
    
    pub fn values(&self) -> std::collections::btree_map::Values<K, V> {
        self.data.values()
    }
    
    pub fn iter(&self) -> std::collections::btree_map::Iter<K, V> {
        self.data.iter()
    }
    
    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        self.data.first_key_value()
    }
    
    pub fn last_key_value(&self) -> Option<(&K, &V)> {
        self.data.last_key_value()
    }
}

/// Convenience functions for creating collections
pub fn vec<T>() -> NeksisVec<T> {
    NeksisVec::new()
}

pub fn hashmap<K, V>() -> NeksisHashMap<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    NeksisHashMap::new()
}

pub fn hashset<T>() -> NeksisHashSet<T>
where
    T: Eq + Hash,
{
    NeksisHashSet::new()
}

pub fn deque<T>() -> NeksisDeque<T> {
    NeksisDeque::new()
}

pub fn ordered_map<K, V>() -> NeksisOrderedMap<K, V>
where
    K: Ord,
{
    NeksisOrderedMap::new()
}

/// Collection utilities
pub fn range(start: i64, end: i64) -> NeksisVec<i64> {
    let mut vec = NeksisVec::new();
    for i in start..end {
        vec.push(i);
    }
    vec
}

pub fn range_inclusive(start: i64, end: i64) -> NeksisVec<i64> {
    let mut vec = NeksisVec::new();
    for i in start..=end {
        vec.push(i);
    }
    vec
}

pub fn repeat<T: Clone>(item: T, count: usize) -> NeksisVec<T> {
    let mut vec = NeksisVec::new();
    for _ in 0..count {
        vec.push(item.clone());
    }
    vec
}
