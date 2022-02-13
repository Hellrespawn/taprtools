#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
//! Iterator with buffered lookahead.
use std::collections::VecDeque;
use std::iter::Fuse;

/// Turns any `IntoIterator` into a `BufferedIterator`
pub fn buffered<I>(iterable: I) -> BufferedIterator<I::IntoIter>
where
    I: IntoIterator,
{
    BufferedIterator {
        iter: iterable.into_iter().fuse(),
        buffer: VecDeque::new(),
    }
}

/// Iterator that buffers lookahead.
#[derive(Clone, Debug)]
pub struct BufferedIterator<I: Iterator> {
    iter: Fuse<I>,
    buffer: VecDeque<I::Item>,
}

impl<I> Iterator for BufferedIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop_front().or_else(|| self.iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let buffered = self.buffer.len();
        let (lower, upper) = self.iter.size_hint();
        (lower + buffered, upper.map(|n| n + buffered))
    }
}

impl<I> ExactSizeIterator for BufferedIterator<I> where I: ExactSizeIterator {}

impl<I> BufferedIterator<I>
where
    I: Iterator,
{
    /// Peeks at the front of the iterator.
    pub fn peek(&mut self) -> Option<&I::Item> {
        if self.buffer.is_empty() {
            if let Some(t) = self.iter.next() {
                self.buffer.push_back(t);
            }
        }

        self.buffer.front()
    }

    /// Peeks at the front `n` elements of the iterator.
    pub fn peekn(&mut self, n: usize) -> &[I::Item] {
        while self.buffer.len() < n {
            if let Some(t) = self.iter.next() {
                self.buffer.push_back(t);
            } else {
                break;
            }
        }

        self.buffer.make_contiguous();

        let (output, _) = self.buffer.as_slices();

        &output[..std::cmp::min(n, output.len())]
    }

    /// Peeks at the `i`th element of the iterator.

    pub fn peeki(&mut self, i: usize) -> Option<&I::Item> {
        self.peekn(i + 1).get(i)
    }

    /// Tries to find the first index that matches `predicate`.

    pub fn findi<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: Fn(&I::Item) -> bool,
    {
        let mut index = None;

        for i in 0.. {
            let opt = self.peeki(i);

            match opt {
                None => return None,
                Some(item) => {
                    if predicate(item) {
                        index = Some(i);
                        break;
                    }
                }
            }
        }

        index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffered_iterator_peek_test() {
        let numbers = [0, 1, 2, 3, 4];
        let mut buf = buffered(numbers);

        assert_eq!(buf.peek(), Some(&0));
        assert_eq!(buf.peek(), Some(&0));

        assert_eq!(buf.next(), Some(0));
        assert_eq!(buf.peek(), Some(&1));
        assert_eq!(buf.peekn(2), &[1, 2]);

        assert_eq!(buf.next(), Some(1));
        assert_eq!(buf.peekn(3), &[2, 3, 4]);
        assert_eq!(buf.peekn(3), &[2, 3, 4]);

        assert_eq!(buf.next(), Some(2));

        assert_eq!(buf.peekn(2), &[3, 4]);
        assert_eq!(buf.peekn(4), &[3, 4]);
        assert_eq!(buf.peekn(5), &[3, 4]);

        assert_eq!(buf.next(), Some(3));
        assert_eq!(buf.next(), Some(4));

        assert_eq!(buf.peek(), None);
        assert_eq!(buf.peekn(8), &[]);
    }

    #[test]
    fn buffered_iterator_peeki_test() {
        let numbers = [0, 1, 2, 3, 4];
        let mut buf = buffered(numbers);

        assert_eq!(buf.peeki(0), Some(&0));

        assert_eq!(buf.peeki(3), Some(&3));

        assert_eq!(buf.next(), Some(0));
        assert_eq!(buf.next(), Some(1));
        assert_eq!(buf.next(), Some(2));
        assert_eq!(buf.next(), Some(3));
        assert_eq!(buf.next(), Some(4));
    }

    #[test]
    fn buffered_iterator_findi_test() {
        let numbers = [0, 1, 2, 3, 4];
        let mut buf = buffered(numbers);

        let i = buf.findi(|s| s == &3);

        assert_eq!(i, Some(3));

        let i = i.unwrap();

        assert_eq!(i, 3);
        assert_eq!(buf.peeki(i), Some(&3));

        for _ in 0..i {
            buf.next();
        }

        assert_eq!(buf.peek(), Some(&3));
    }
}
