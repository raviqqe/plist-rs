use crate::{map, Map};
use std::{
    borrow::Borrow,
    collections::{hash_map, HashMap, HashSet},
    fmt::{self, Debug, Formatter},
    hash::Hash,
    ops::Index,
    rc::Rc,
};

pub struct FlailMap<K, V> {
    chain: Map<K, V>,
    head: Rc<HashMap<K, V>>,
}

impl<K, V> FlailMap<K, V> {
    pub fn new(head: HashMap<K, V>) -> Self {
        Self {
            chain: Default::default(),
            head: head.into(),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Self {
        Self {
            chain: self.chain.insert(key, value),
            head: self.head.clone(),
        }
    }

    pub fn insert_many(&self, iterator: impl IntoIterator<Item = (K, V)>) -> Self {
        Self {
            chain: self.chain.insert_many(iterator),
            head: self.head.clone(),
        }
    }
}

impl<K: Eq + Hash, V> FlailMap<K, V> {
    pub fn len(&self) -> usize {
        let mut set = HashSet::new();

        for key in self.keys() {
            set.insert(key);
        }

        set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chain.is_empty() && self.head.is_empty()
    }

    pub fn get<Q: Eq + Hash + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        self.chain.get(key).or_else(|| self.head.get(key))
    }

    pub fn contains_key<Q: Eq + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.keys().any(|other| other.borrow() == key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.into_iter().map(|(key, _)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.into_iter().map(|(_, value)| value)
    }
}

impl<Q: Eq + Hash + ?Sized, K: Eq + Hash, V> Index<&Q> for FlailMap<K, V>
where
    K: Borrow<Q>,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("existent key")
    }
}

impl<K, V> Clone for FlailMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            chain: self.chain.clone(),
            head: self.head.clone(),
        }
    }
}

impl<K, V> Default for FlailMap<K, V> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<K: Debug + Eq + Hash, V: Debug> Debug for FlailMap<K, V> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{{")?;

        for (index, (key, value)) in self.into_iter().enumerate() {
            write!(formatter, "{:?}: {:?}", key, value)?;

            if index < self.len() - 1 {
                write!(formatter, ", ")?;
            }
        }

        write!(formatter, "}}")?;

        Ok(())
    }
}

impl<K: Eq + Hash, V: PartialEq> PartialEq for FlailMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        let set = self.into_iter().collect::<HashMap<_, _>>();

        for (key, value) in other {
            if let Some(&other_value) = set.get(key) {
                if value != other_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl<K: Eq + Hash, V: Eq> Eq for FlailMap<K, V> {}

impl<K: Eq + Hash, V> FromIterator<(K, V)> for FlailMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iterator: I) -> Self {
        Self::new(iterator.into_iter().collect())
    }
}

pub struct FlailMapIterator<'a, K: Eq + Hash, V> {
    chain_iterator: map::MapIterator<'a, K, V>,
    head_iterator: hash_map::Iter<'a, K, V>,
    set: HashSet<&'a K>,
}

impl<'a, K: Eq + Hash, V> IntoIterator for &'a FlailMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = FlailMapIterator<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        FlailMapIterator {
            chain_iterator: self.chain.into_iter(),
            head_iterator: self.head.iter(),
            set: Default::default(),
        }
    }
}

impl<'a, K: Eq + Hash, V> Iterator for FlailMapIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((key, value)) = self.chain_iterator.next() {
            if self.set.contains(key) {
                return self.next();
            }

            self.set.insert(key);

            Some((key, value))
        } else if let Some((key, value)) = self.head_iterator.next() {
            if self.set.contains(key) {
                return self.next();
            }

            self.set.insert(key);

            Some((key, value))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn new() {
        FlailMap::<(), ()>::new(Default::default());
    }

    #[test]
    fn equal() {
        assert_eq!(
            FlailMap::<(), ()>::new(Default::default()),
            FlailMap::new(Default::default())
        );
        assert_ne!(
            FlailMap::new(Default::default()),
            FlailMap::new(Default::default()).insert(42, 42)
        );
        assert_eq!(
            FlailMap::new(Default::default()).insert(42, 42),
            FlailMap::new(Default::default()).insert(42, 42)
        );
        assert_eq!(
            FlailMap::new(Default::default()).insert(42, 42),
            FlailMap::new(Default::default())
                .insert(42, 42)
                .insert(42, 42)
        );
        assert_ne!(
            FlailMap::new(Default::default()).insert(1, 1),
            FlailMap::new(Default::default()).insert(1, 1).insert(2, 2)
        );
        assert_eq!(
            FlailMap::new(Default::default()).insert(2, 2).insert(1, 1),
            FlailMap::new(Default::default()).insert(1, 1).insert(2, 2)
        );
        assert_eq!(
            FlailMap::new([(2, 2)].into_iter().collect()).insert(1, 1),
            FlailMap::new(Default::default()).insert(1, 1).insert(2, 2)
        );
        assert_eq!(
            FlailMap::new([(1, 1), (2, 2)].into_iter().collect()),
            FlailMap::new(Default::default()).insert(1, 1).insert(2, 2)
        );
    }

    #[test]
    fn len() {
        assert_eq!(FlailMap::<(), ()>::new(Default::default()).len(), 0);
        assert_eq!(FlailMap::new(Default::default()).insert(1, 1).len(), 1);
        assert_eq!(
            FlailMap::new(Default::default())
                .insert(1, 1)
                .insert(1, 1)
                .len(),
            1
        );
        assert_eq!(
            FlailMap::new(Default::default())
                .insert(1, 1)
                .insert(2, 2)
                .len(),
            2
        );
        assert_eq!(
            FlailMap::new([(1, 1)].into_iter().collect())
                .insert(1, 1)
                .len(),
            1
        );
        assert_eq!(
            FlailMap::new([(1, 1)].into_iter().collect())
                .insert(2, 2)
                .len(),
            2
        );
    }

    #[test]
    fn is_empty() {
        assert!(FlailMap::<(), ()>::new(Default::default()).is_empty());
        assert!(!FlailMap::new(Default::default()).insert(1, 1).is_empty());
        assert!(!FlailMap::new([(1, 1)].into_iter().collect()).is_empty());
    }

    #[test]
    fn get() {
        let map = FlailMap::new(Default::default()).insert(1, 2).insert(3, 4);

        assert_eq!(map.get(&1), Some(&2));
        assert_eq!(map.get(&3), Some(&4));
        assert_eq!(map.get(&4), None);
    }

    #[test]
    fn get_from_head() {
        let map = FlailMap::new([(1, 2)].into_iter().collect()).insert(3, 4);

        assert_eq!(map.get(&1), Some(&2));
        assert_eq!(map.get(&3), Some(&4));
        assert_eq!(map.get(&4), None);
    }

    #[test]
    fn contains() {
        assert!(FlailMap::new(Default::default())
            .insert(1, 1)
            .insert(2, 2)
            .contains_key(&2));
        assert!(FlailMap::new([(1, 1)].into_iter().collect())
            .insert(1, 1)
            .contains_key(&1));
        assert!(FlailMap::new([(1, 1)].into_iter().collect())
            .insert(2, 2)
            .contains_key(&2));
    }

    #[test]
    fn insert_many() {
        assert_eq!(
            FlailMap::new(Default::default())
                .insert(1, 1)
                .insert(2, 2)
                .into_iter()
                .collect::<Vec<_>>(),
            FlailMap::new(Default::default())
                .insert_many([(1, 1), (2, 2)])
                .into_iter()
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn into_iter() {
        assert_eq!(
            FlailMap::new(Default::default())
                .insert(1, 1)
                .insert(2, 2)
                .into_iter()
                .collect::<HashSet<_>>(),
            [(&1, &1), (&2, &2)].into_iter().collect()
        );
    }

    #[test]
    fn into_iter_duplicates() {
        assert_eq!(
            FlailMap::new(Default::default())
                .insert(1, 1)
                .insert(1, 1)
                .into_iter()
                .count(),
            1
        );
        assert_eq!(
            FlailMap::new([(1, 1)].into_iter().collect())
                .insert(1, 1)
                .insert(1, 1)
                .into_iter()
                .count(),
            1
        );
    }

    #[test]
    fn from_iter() {
        assert_eq!(
            FlailMap::from_iter([(1, 1), (2, 2)]),
            FlailMap::from_iter([(1, 1), (2, 2)]),
        );
    }

    #[test]
    fn debug() {
        assert_eq!(
            format!("{:?}", FlailMap::<(), ()>::new(Default::default())),
            "{}"
        );
        assert_eq!(
            format!("{:?}", FlailMap::new(Default::default()).insert(1, 2)),
            "{1: 2}"
        );
        assert_eq!(
            format!(
                "{:?}",
                FlailMap::new(Default::default()).insert_many([(1, 2), (3, 4)])
            ),
            "{3: 4, 1: 2}"
        );
        assert_eq!(
            format!(
                "{:?}",
                FlailMap::new(Default::default()).insert_many([(1, 2), (3, 4), (5, 6)])
            ),
            "{5: 6, 3: 4, 1: 2}"
        );

        assert_eq!(
            format!(
                "{:?}",
                FlailMap::new([(5, 6)].into_iter().collect()).insert_many([(3, 4), (1, 2)])
            ),
            format!(
                "{:?}",
                BTreeMap::<_, _>::from_iter([(1, 2), (3, 4), (5, 6)])
            )
        );
    }
}
