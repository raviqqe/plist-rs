use crate::Map;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Formatter},
    hash::Hash,
    ops::Index,
    rc::Rc,
};

pub struct ChainMap<K, V> {
    chain: Map<K, V>,
    head: Rc<HashMap<K, V>>,
}

impl<K, V> ChainMap<K, V> {
    pub fn new(head: HashMap<K, V>) -> Self {
        Self {
            chain: Default::default(),
            head: head.into(),
        }
    }

    pub fn get<Q: Eq + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        self.chain.get(key).or_else(|| self.head.get(key))
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

impl<K: Eq + Hash, V> ChainMap<K, V> {
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

impl<Q: Eq + ?Sized, K: Eq, V> Index<&Q> for ChainMap<K, V>
where
    K: Borrow<Q>,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("existent key")
    }
}

impl<K, V> Clone for ChainMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            chain: self.chain.clone(),
            head: self.head.clone(),
        }
    }
}

impl<K, V> Default for ChainMap<K, V> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<K: Debug + Eq + Hash, V: Debug> Debug for ChainMap<K, V> {
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

impl<K: Eq + Hash, V: PartialEq> PartialEq for ChainMap<K, V> {
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

impl<K: Eq + Hash, V: Eq> Eq for ChainMap<K, V> {}

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
