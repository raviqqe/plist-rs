use crate::{List, ListIterator};
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::Index,
};

#[derive(Debug)]
pub struct Map<K, V>(List<(K, V)>);

impl<K, V> Map<K, V> {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn get<Q: Eq + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        self.0.into_iter().find_map(|(other_key, value)| {
            if other_key.borrow() == key {
                Some(value)
            } else {
                None
            }
        })
    }

    pub fn insert(&self, key: K, value: V) -> Self {
        Self(self.0.push_front((key, value)))
    }

    pub fn insert_many(&self, iterator: impl IntoIterator<Item = (K, V)>) -> Self {
        Self(self.0.push_front_many(iterator))
    }
}

impl<K: Eq + Hash, V> Map<K, V> {
    pub fn len(&self) -> usize {
        let mut set = HashSet::new();

        for key in self.keys() {
            set.insert(key);
        }

        set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

impl<Q: Eq + ?Sized, K: Eq, V> Index<&Q> for Map<K, V>
where
    K: Borrow<Q>,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("existent key")
    }
}

impl<K, V> Clone for Map<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash, V: Eq> Eq for Map<K, V> {}

impl<K: Eq + Hash, V: PartialEq> PartialEq for Map<K, V> {
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

impl<K, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iterator: I) -> Self {
        Self::new().insert_many(iterator)
    }
}

pub struct MapIterator<'a, K: Eq + Hash, V> {
    iterator: ListIterator<'a, (K, V)>,
    set: HashSet<&'a K>,
}

impl<'a, K: Eq + Hash, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = MapIterator<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        MapIterator {
            set: Default::default(),
            iterator: self.0.into_iter(),
        }
    }
}

impl<'a, K: Eq + Hash, V> Iterator for MapIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((key, value)) = self.iterator.next() {
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

    #[test]
    fn new() {
        Map::<(), ()>::new();
    }

    #[test]
    fn equal() {
        assert_eq!(Map::<(), ()>::new(), Map::new());
        assert_ne!(Map::new(), Map::new().insert(42, 42));
        assert_eq!(Map::new().insert(42, 42), Map::new().insert(42, 42));
        assert_eq!(
            Map::new().insert(42, 42),
            Map::new().insert(42, 42).insert(42, 42)
        );
        assert_ne!(
            Map::new().insert(1, 1),
            Map::new().insert(1, 1).insert(2, 2)
        );
        assert_eq!(
            Map::new().insert(2, 2).insert(1, 1),
            Map::new().insert(1, 1).insert(2, 2)
        );
    }

    #[test]
    fn len() {
        assert_eq!(Map::<(), ()>::new().len(), 0);
        assert_eq!(Map::new().insert(1, 1).len(), 1);
        assert_eq!(Map::new().insert(1, 1).insert(1, 1).len(), 1);
        assert_eq!(Map::new().insert(1, 1).insert(2, 2).len(), 2);
    }

    #[test]
    fn is_empty() {
        assert!(Map::<(), ()>::new().is_empty());
        assert!(!Map::new().insert(1, 1).is_empty());
    }

    #[test]
    fn get() {
        let map = Map::new().insert(1, 2).insert(3, 4);

        assert_eq!(map.get(&1), Some(&2));
        assert_eq!(map.get(&3), Some(&4));
        assert_eq!(map.get(&4), None);
    }

    #[test]
    fn contains() {
        assert!(Map::new().insert(1, 1).insert(2, 2).contains_key(&2),);
    }

    #[test]
    fn insert_many() {
        assert_eq!(
            Map::new()
                .insert(1, 1)
                .insert(2, 2)
                .into_iter()
                .collect::<Vec<_>>(),
            Map::new()
                .insert_many([(1, 1), (2, 2)])
                .into_iter()
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn into_iter() {
        assert_eq!(
            Map::new()
                .insert(1, 1)
                .insert(2, 2)
                .into_iter()
                .collect::<HashSet<_>>(),
            [(&1, &1), (&2, &2)].into_iter().collect()
        );
    }

    #[test]
    fn into_iter_duplicates() {
        assert_eq!(Map::new().insert(1, 1).insert(1, 1).into_iter().count(), 1);
    }

    #[test]
    fn from_iter() {
        assert_eq!(
            Map::from_iter([(1, 1), (2, 2)]),
            Map::from_iter([(1, 1), (2, 2)]),
        );
    }

    #[test]
    fn from_iter_duplicates() {
        assert_eq!(
            Map::from_iter([(1, 1), (2, 2)]),
            Map::from_iter([(1, 1), (2, 2), (1, 1)]),
        );
    }
}
