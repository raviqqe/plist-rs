use std::{borrow::Borrow, rc::Rc};

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct List<T> {
    cons: Option<Rc<Cons<T>>>,
    size: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
struct Cons<T> {
    head: T,
    tail: Option<Rc<Cons<T>>>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            cons: None,
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn push_front(&self, head: T) -> Self {
        Self {
            cons: Rc::new(Cons {
                head,
                tail: self.cons.clone(),
            })
            .into(),
            size: self.size + 1,
        }
    }

    pub fn push_front_many(&self, iterator: impl IntoIterator<Item = T>) -> Self {
        let mut list = self.clone();

        for value in iterator {
            list = list.push_front(value);
        }

        list
    }

    pub fn pop_front(&self) -> Self {
        if let Some(cons) = &self.cons {
            Self {
                cons: cons.tail.clone(),
                size: self.size - 1,
            }
        } else {
            Self::new()
        }
    }

    pub fn contains<S: Eq + ?Sized>(&self, value: &S) -> bool
    where
        T: Borrow<S>,
    {
        self.into_iter().any(|other| other.borrow() == value)
    }
}

impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        Self {
            cons: self.cons.clone(),
            size: self.size,
        }
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iterator: I) -> Self {
        let mut list = List::new();

        for value in iterator {
            list = list.push_front(value);
        }

        list
    }
}

pub struct ListIterator<'a, T>(&'a Option<Rc<Cons<T>>>);

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = ListIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ListIterator(&self.cons)
    }
}

impl<'a, T> Iterator for ListIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cons) = self.0 {
            self.0 = &cons.tail;

            Some(&cons.head)
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
        List::<()>::new();
    }

    #[test]
    fn equal() {
        assert_ne!(List::new(), List::new().push_front(42));
        assert_eq!(List::<()>::new(), List::new());
        assert_eq!(List::new().push_front(42), List::new().push_front(42));
    }

    #[test]
    fn ord() {
        assert!(List::new() < List::new().push_front(1));
        assert!(List::new().push_front(1) < List::new().push_front(2));
        assert!(List::new().push_front(1) < List::new().push_front(1).push_front(1));
        assert!(List::new().push_front(1).push_front(1) < List::new().push_front(2).push_front(1));
    }

    #[test]
    fn len() {
        assert_eq!(List::<()>::new().len(), 0);
        assert_eq!(List::new().push_front(42).len(), 1);
        assert_eq!(List::new().push_front(42).push_front(42).len(), 2);
    }

    #[test]
    fn is_empty() {
        assert!(List::<()>::new().is_empty());
        assert!(!List::new().push_front(42).is_empty());
    }

    #[test]
    fn contains() {
        assert!(List::new().push_front(1).push_front(2).contains(&2),);
    }

    #[test]
    fn into_iter() {
        assert_eq!(
            List::new()
                .push_front(1)
                .push_front(2)
                .into_iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![2, 1]
        );
    }

    #[test]
    fn from_iter() {
        assert_eq!(
            [1, 2].into_iter().collect::<List<_>>(),
            List::new().push_front(1).push_front(2)
        );
    }
}
