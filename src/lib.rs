//! Vector of items with associated [isize] for ordering.
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]
use std::{
    mem,
    ops::{Index, IndexMut, RangeBounds},
    slice::{Iter, IterMut, SliceIndex},
    vec::{Drain, IntoIter},
};

/// Container for multiple elements sorted by a certain `isize` order.
///
/// Every element `T` is tagged with an associated `isize`. The `isize` value decides the relative
/// ordering of the elements. All manipulations keep the items `T` ordered according to the `isize`
/// values from lowest to highest.
///
/// ```
/// use isize_vec::IsizeVec;
///
/// let mut vector = IsizeVec::new();
///
/// vector.insert(10, 'a');
/// vector.insert(5, 'b');
///
/// println!("{:?}", vector);
///
/// for value in vector {
///     println!("{}", value);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct IsizeVec<T> {
    items: Vec<T>,
    order: Vec<isize>,
}

impl<T> Default for IsizeVec<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            order: Vec::new(),
        }
    }
}

impl<T> IsizeVec<T> {
    /// Create a new vector.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            order: Vec::new(),
        }
    }

    /// Get an iterator to the values.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        self.items.iter()
    }

    /// Get an iterator to the values (mutable).
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.items.iter_mut()
    }

    /// Push a value to the end of the vector, with `relative: isize::MAX`.
    pub fn push(&mut self, item: T) -> usize {
        self.items.push(item);
        self.order.push(isize::MAX);
        self.items.len() - 1
    }

    /// Remove the last element from this vector.
    pub fn pop(&mut self) -> Option<(T, isize)> {
        if !self.items.is_empty() {
            Some((self.items.pop().unwrap(), self.order.pop().unwrap()))
        } else {
            None
        }
    }

    /// Create a drain iterator.
    pub fn drain<R>(&mut self, range: R) -> Drain<T>
    where
        R: Clone + RangeBounds<usize>,
    {
        self.order.drain(range.clone());
        self.items.drain(range)
    }

    /// Returns the length of the container
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the container is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the item at a given index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// Get the item at a given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index)
    }

    /// Return the backing vector and clear this container.
    pub fn extract(&mut self) -> Vec<T> {
        self.order.clear();
        mem::replace(&mut self.items, Vec::new())
    }

    /// Find the index at which positive elements start.
    pub fn first_positive(&self) -> usize {
        self.first_right_of(-1)
    }

    /// Insert a value into this vector.
    ///
    /// The value `relative` indicates where the value will be put in the list relative to other
    /// values. If two values have the same `relative` value, then the value will be prepended if it
    /// is signed, and appended if unsigned.
    ///
    /// Returns the index of insertion.
    pub fn insert(&mut self, relative: isize, item: T) -> usize {
        match self.order.binary_search(&relative) {
            Ok(exact) => {
                if relative >= 0 {
                    match self.order[exact..]
                        .iter()
                        .enumerate()
                        .find(|(_, &x)| x != relative)
                    {
                        Some((idx, _)) => {
                            self.items.insert(exact + idx, item);
                            self.order.insert(exact + idx, relative);
                            exact + idx
                        }
                        None => {
                            self.items.push(item);
                            self.order.push(relative);
                            self.items.len() - 1
                        }
                    }
                } else {
                    match self.order[..exact]
                        .iter()
                        .rev()
                        .enumerate()
                        .find(|(_, &x)| x != relative)
                    {
                        Some((idx, _)) => {
                            self.items.insert(exact - idx, item);
                            self.order.insert(exact - idx, relative);
                            exact - idx
                        }
                        None => {
                            self.items.insert(0, item);
                            self.order.insert(0, relative);
                            0
                        }
                    }
                }
            }
            Err(order) => {
                self.items.insert(order, item);
                self.order.insert(order, relative);
                order
            }
        }
    }

    /// Remove the given index from the vector.
    pub fn remove(&mut self, index: usize) -> (T, isize) {
        (self.items.remove(index), self.order.remove(index))
    }

    /// Find the first index to the right of the relative list.
    pub fn first_right_of(&self, relative: isize) -> usize {
        match self.order.binary_search(&relative) {
            Ok(exact) => self.order[exact..]
                .iter()
                .enumerate()
                .find(|(_, &x)| x != relative)
                .map(|(index, _)| exact + index)
                .unwrap_or_else(|| self.order.len()),
            Err(order) => order,
        }
    }

    /// Swap two elements in the list. Associated order is swapped.
    pub fn swap(&mut self, a: usize, b: usize) {
        self.items.swap(a, b);
    }

    /// Same as [Vec::retain].
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let mut removed = 0;
        let mut idx = 0;
        let order = &mut self.order;
        self.items.retain(|x| {
            idx += 1;
            if (f)(x) {
                true
            } else {
                order.remove(idx - 1 - removed);
                removed += 1;
                false
            }
        });
    }
}

impl<T, I> Index<I> for IsizeVec<T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;
    fn index(&self, index: I) -> &<Vec<T> as Index<I>>::Output {
        &self.items[index]
    }
}

impl<T, I> IndexMut<I> for IsizeVec<T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut <Vec<T> as Index<I>>::Output {
        &mut self.items[index]
    }
}

impl<T> IntoIterator for IsizeVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a IsizeVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.items.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut IsizeVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> {
        self.items.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::IsizeVec;

    #[quickcheck_macros::quickcheck]
    fn inserting_appends_or_prepends(relative: isize, values: usize) {
        let mut vector = IsizeVec::new();

        for value in 0..values {
            vector.insert(relative, value);
        }

        if relative >= 0 {
            let mut value = 0;
            for item in vector.iter() {
                assert_eq!(value, *item);
                value += 1;
            }
            assert_eq!(value, values);
        } else {
            let mut value = values;
            for item in vector.iter() {
                value -= 1;
                assert_eq!(value, *item);
            }
            assert_eq!(value, 0);
        }
    }

    #[quickcheck_macros::quickcheck]
    fn insertion_behaves_as_ordering(orders: Vec<(u8, isize)>) {
        let mut vector = IsizeVec::new();

        for (item, order) in &orders {
            vector.insert(*order, item);
        }

        let mut unsigned = orders
            .iter()
            .cloned()
            .filter(|(_, o)| o >= &0)
            .collect::<Vec<_>>();

        let mut signed = orders
            .iter()
            .cloned()
            .filter(|(_, o)| o < &0)
            .collect::<Vec<_>>();

        unsigned.sort_by(|a, b| a.1.cmp(&b.1));
        signed.reverse();
        signed.sort_by(|a, b| a.1.cmp(&b.1));

        for (idx, (item, _)) in signed.iter().chain(unsigned.iter()).enumerate() {
            assert_eq!(vector[idx], item);
        }
    }

    #[quickcheck_macros::quickcheck]
    fn first_right_of_size(orders: Vec<isize>) {
        let mut vector = IsizeVec::new();

        for order in &orders {
            vector.insert(*order, ());
        }

        assert_eq!(vector.first_right_of(isize::MAX), vector.len());
    }

    #[test]
    fn basic() {
        let mut vector = IsizeVec::new();

        let value = 0;
        vector.insert(0, 0);
        vector.insert(1, 1);

        let mut number = 0;
        for item in vector.iter() {
            assert_eq!(number, *item);
            number += 1;
        }
        assert_eq!(number, 2);

        vector.retain(|&x| x != value);

        let mut number = 1;
        for item in vector.iter() {
            assert_eq!(number, *item);
            number += 1;
        }
        assert_eq!(number, 2);
    }

    #[test]
    fn haystack() {
        let mut vector = IsizeVec::new();

        let value = 0;
        for _ in 0..10 {
            vector.insert(0, value);
        }
        for _ in 0..10 {
            vector.insert(2, value);
        }

        vector.insert(1, 1);

        vector.retain(|&x| x != value);

        let mut count = 0;
        for item in vector.iter() {
            assert_eq!(*item, 1);
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn find_first_positive() {
        let mut vector = IsizeVec::new();
        vector.insert(-1, 'a');
        vector.insert(0, 'b');
        vector.insert(1, 'c');

        assert!(matches!(vector.first_positive(), 1));

        let mut vector = IsizeVec::new();
        vector.insert(-2, 'a');
        vector.insert(-1, 'b');

        assert!(matches!(vector.first_positive(), 2));

        let mut vector = IsizeVec::new();
        vector.insert(-2, 'a');
        vector.insert(0, 'b');

        assert!(matches!(vector.first_positive(), 1));

        let mut vector = IsizeVec::new();
        vector.insert(0, 'a');
        vector.insert(1, 'b');

        assert!(matches!(vector.first_positive(), 0));

        let mut vector = IsizeVec::new();
        vector.insert(0, 'a');
        vector.insert(0, 'b');
        vector.insert(0, 'c');

        assert!(matches!(vector.first_positive(), 0));
    }

    #[test]
    fn find_first_right_of() {
        let mut vector = IsizeVec::new();
        vector.insert(0, 'a');
        vector.insert(1, 'a');

        assert!(matches!(vector.first_right_of(1), 2));

        let mut vector = IsizeVec::new();
        vector.insert(0, 'a');
        vector.insert(1, 'a');
        vector.insert(2, 'a');

        assert!(matches!(vector.first_right_of(1), 2));

        let mut vector = IsizeVec::new();
        vector.insert(0, 'a');
        vector.insert(2, 'a');

        assert!(matches!(vector.first_right_of(1), 1));

        let mut vector = IsizeVec::new();
        vector.insert(-101, 'a');
        vector.insert(-100, 'b');

        assert!(matches!(vector.first_right_of(0), 2));
        assert!(matches!(vector.first_right_of(-100), 2));
        assert!(matches!(vector.first_right_of(-101), 1));
        assert!(matches!(vector.first_right_of(-1000), 0));

        let mut vector = IsizeVec::new();
        vector.insert(0, 'a');
        vector.insert(0, 'b');

        assert!(matches!(vector.first_right_of(0), 2));
    }
}
