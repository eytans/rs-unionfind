use std::{fmt::Debug, hash::Hash, cell::RefCell, rc::Rc};
use indexmap::IndexMap;


type Rank = usize;

/// A type that can be used as an id in a union-find data structure.
/// 
/// This trait is implemented for hashable types, as a way to have a single object unionfind on complex data.
/// 
/// # Examples
/// 
/// ```
/// use hash_unionfind::UnionFind;
/// 
/// let mut uf = UnionFind::default();
/// let a = uf.insert("a");
/// let b = uf.insert("b");
/// let c = uf.insert("c");
/// let d = uf.insert("d");
/// let e = uf.insert("e");
/// 
/// uf.union(&"a", &"b");
/// uf.union(&"b", &"c");
/// 
/// uf.union(&"d", &"e");
/// 
/// assert_eq!(uf.find(&"a"), uf.find(&"c"));
/// assert_ne!(uf.find(&"a"), uf.find(&"d"));
/// 
/// uf.union(&"a", &"d");
/// 
/// assert_eq!(uf.find(&"a"), uf.find(&"e"));
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnionFind<T: Hash + Eq + Clone + Debug> {
    // The parents of each node. The index is T and we keep the maybe updated leader + rank.
    parents: RefCell<IndexMap<T, (Rc<T>, Rank)>>,
}

impl<T: Hash + Eq + Clone + Debug> UnionFind<T> {
    pub fn new() -> Self {
        Self {
            parents: RefCell::new(IndexMap::new()),
        }
    }

    pub fn size(&self) -> usize {
        self.parents.borrow().len()
    }

    // Create a new set from the element t.
    pub fn insert(&mut self, t: T) {
        if self.parents.borrow().contains_key(&t) {
            return;
        }
        let rc_t = Rc::new(t.clone());
        self.parents.borrow_mut().insert(t, (rc_t, 1));
    }

    fn inner_find(&self, current: &T) -> Option<(Rc<T>, Rank)> {
        // If the current node is not in the map, it is not in the union-find.
        // All other cases node will point to parent or itself.
        if !self.parents.borrow().contains_key(current) {
            return None;
        }

        let mut ps = self.parents.borrow_mut();
        let mut old = current;
        let mut current = &ps[old].0;
        let mut current_rank = &ps[old].1;
        let mut to_update = vec![];
        while current.as_ref() != old {
            to_update.push(old.clone());
            old = current.as_ref();
            current = &ps[old].0;
            current_rank = &ps[old].1;
        }
        
        let current = current.clone();
        let current_rank = *current_rank;
        for u in to_update {
            // It is actually unneccessary to update rank
            ps.insert(u.clone(), (current.clone(), current_rank));
        }
        
        Some((current, current_rank))
    }

    // Find the leader of the set that t is in. This is amortized to O(log*(n))
    // This uses [RefCell], and is therefore unsafe to call concurrently.
    // TODO: Make this safe to call concurrently using atomic keys.
    pub fn find(&self, current: &T) -> Option<Rc<T>> {
        self.inner_find(current).map(|(leader, _)| leader)
    }

    /// Given two ids, unions the two eclasses making the bigger class the leader.
    /// If one of the items is missing returns None.
    pub fn union(&mut self, x: &T, y: &T) -> Option<Rc<T>> {
        let (mut x, x_rank) = self.inner_find(x)?;
        let (mut y, y_rank) = self.inner_find(y)?;
        if x == y {
            return Some(x);
        }
        if y_rank > x_rank {
            std::mem::swap(&mut x, &mut y);
        }
        let x = x;
        let y = y;
        let mut ps = self.parents.borrow_mut();
        let new_x_res = ps[x.as_ref()].0.clone();
        *ps.get_mut(y.as_ref()).unwrap() = (new_x_res.clone(), x_rank + y_rank);
        *ps.get_mut(x.as_ref()).unwrap() = (new_x_res.clone(), x_rank + y_rank);
        Some(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_find() {
        let n = 10;

        let mut uf = UnionFind::default();
        for i in 0..n {
            uf.insert(i);
        }

        // build up one set
        uf.union(&0, &1);
        uf.union(&0, &2);
        uf.union(&0, &3);

        // build up another set
        uf.union(&6, &7);
        uf.union(&6, &8);
        uf.union(&6, &9);

        // indexes:         0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        let expected = vec![0, 0, 0, 0, 4, 5, 6, 6, 6, 6];
        for i in 0..n {
            assert_eq!(uf.find(&i).unwrap().as_ref(), &expected[i]);
        }
    }

    #[test]
    fn test_on_str() {
        let mut uf = UnionFind::new();
        uf.insert("a");
        uf.insert("b");
        uf.insert("c");
        uf.insert("d");
        uf.insert("e");
            
        uf.union(&"a", &"b");
        uf.union(&"b", &"c");
            
        uf.union(&"d", &"e");
        
        assert_eq!(None, uf.union(&"x", &"a"));
        assert_eq!(None, uf.union(&"a", &"x"));
        assert_eq!(None, uf.find(&"x"));

        assert_eq!(uf.find(&"a"), uf.find(&"c"));
        assert_ne!(uf.find(&"a"), uf.find(&"d"));
            
        uf.union(&"a", &"d");
            
        assert_eq!(uf.find(&"a"), uf.find(&"e"));
        assert_eq!(&"a", uf.find(&"a").unwrap().as_ref());
    }
}
