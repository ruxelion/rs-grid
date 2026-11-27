//! O(1) least-recently-used order tracking.
//!
//! The idea ported from World42's CBT terrain engine: a fixed pool of
//! index-addressed slots recycled via a free list, instead of a structure
//! that reallocates or shifts elements. Applied here to LRU order instead
//! of GPU triangle bisectors — `touch`/`remove`/`pop_lru` are all O(1),
//! unlike a `VecDeque` + linear `retain` scan.

use std::{collections::HashMap, hash::Hash};

const NIL: u32 = u32::MAX;

#[derive(Debug, Clone, Copy)]
struct Node<K> {
    key: K,
    prev: u32,
    next: u32,
}

/// O(1) most-recently-used order tracker over keys of type `K`.
#[derive(Debug)]
pub struct LruRing<K> {
    nodes: Vec<Node<K>>,
    free: Vec<u32>,
    index: HashMap<K, u32>,
    head: u32,
    tail: u32,
}

impl<K: Copy + Eq + Hash> Default for LruRing<K> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            free: Vec::new(),
            index: HashMap::new(),
            head: NIL,
            tail: NIL,
        }
    }
}

impl<K: Copy + Eq + Hash> LruRing<K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    fn unlink(&mut self, i: u32) {
        let (prev, next) = {
            let n = &self.nodes[i as usize];
            (n.prev, n.next)
        };
        if prev != NIL {
            self.nodes[prev as usize].next = next;
        } else {
            self.head = next;
        }
        if next != NIL {
            self.nodes[next as usize].prev = prev;
        } else {
            self.tail = prev;
        }
    }

    fn push_front(&mut self, i: u32) {
        let old_head = self.head;
        self.nodes[i as usize].prev = NIL;
        self.nodes[i as usize].next = old_head;
        if old_head != NIL {
            self.nodes[old_head as usize].prev = i;
        } else {
            self.tail = i;
        }
        self.head = i;
    }

    /// Mark `key` as most-recently-used, inserting it if new.
    pub fn touch(&mut self, key: K) {
        if let Some(&i) = self.index.get(&key) {
            self.unlink(i);
            self.push_front(i);
            return;
        }
        let i = if let Some(i) = self.free.pop() {
            self.nodes[i as usize] = Node {
                key,
                prev: NIL,
                next: NIL,
            };
            i
        } else {
            self.nodes.push(Node {
                key,
                prev: NIL,
                next: NIL,
            });
            (self.nodes.len() - 1) as u32
        };
        self.index.insert(key, i);
        self.push_front(i);
    }

    /// Remove `key` if present.
    pub fn remove(&mut self, key: K) {
        if let Some(i) = self.index.remove(&key) {
            self.unlink(i);
            self.free.push(i);
        }
    }

    /// Evict and return the least-recently-used key.
    pub fn pop_lru(&mut self) -> Option<K> {
        if self.tail == NIL {
            return None;
        }
        let i = self.tail;
        let key = self.nodes[i as usize].key;
        self.unlink(i);
        self.index.remove(&key);
        self.free.push(i);
        Some(key)
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.free.clear();
        self.index.clear();
        self.head = NIL;
        self.tail = NIL;
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_new_keys_orders_lru_to_mru() {
        let mut r = LruRing::new();
        r.touch(1u64);
        r.touch(2);
        r.touch(3);
        assert_eq!(r.len(), 3);
        assert_eq!(r.pop_lru(), Some(1));
        assert_eq!(r.pop_lru(), Some(2));
        assert_eq!(r.pop_lru(), Some(3));
        assert_eq!(r.pop_lru(), None);
    }

    #[test]
    fn re_touch_moves_to_most_recently_used() {
        let mut r = LruRing::new();
        r.touch(1u64);
        r.touch(2);
        r.touch(3);
        r.touch(1); // 1 is now MRU; LRU order is 2, 3, 1
        assert_eq!(r.pop_lru(), Some(2));
        assert_eq!(r.pop_lru(), Some(3));
        assert_eq!(r.pop_lru(), Some(1));
    }

    #[test]
    fn remove_evicts_from_middle() {
        let mut r = LruRing::new();
        r.touch(1u64);
        r.touch(2);
        r.touch(3);
        r.remove(2);
        assert_eq!(r.len(), 2);
        assert_eq!(r.pop_lru(), Some(1));
        assert_eq!(r.pop_lru(), Some(3));
    }

    #[test]
    fn freed_slots_are_recycled() {
        let mut r = LruRing::new();
        for k in 0..5u64 {
            r.touch(k);
        }
        for _ in 0..5u64 {
            r.pop_lru();
        }
        assert!(r.is_empty());
        // Reuses freed node slots rather than growing unbounded.
        r.touch(100u64);
        assert_eq!(r.nodes.len(), 5);
        assert_eq!(r.pop_lru(), Some(100));
    }

    #[test]
    fn clear_resets_everything() {
        let mut r = LruRing::new();
        r.touch(1u64);
        r.touch(2);
        r.clear();
        assert!(r.is_empty());
        assert_eq!(r.pop_lru(), None);
    }
}
