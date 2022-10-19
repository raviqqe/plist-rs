use std::{borrow::Borrow, sync::Arc};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct List<T> {
    cons: Option<Arc<Cons<T>>>,
    size: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
struct Cons<T> {
    head: T,
    tail: Option<Arc<Cons<T>>>,
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
            cons: Arc::new(Cons {
                head,
                tail: self.cons.clone(),
            })
            .into(),
            size: self.size + 1,
        }
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
        self.into_iter().any(|element| element.borrow() == value)
    }
}

pub struct ListIterator<'a, T>(&'a Option<Arc<Cons<T>>>);

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
}
