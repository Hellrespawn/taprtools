use std::collections::VecDeque;

pub trait Buffered: Iterator {
    fn buffered(self) -> BufferedIterator<Self>
    where
        Self: Sized,
    {
        BufferedIterator::new(self)
    }
}

impl<I> Buffered for I where I: Iterator {}

pub struct BufferedIterator<I>
where
    I: Iterator,
{
    iter: I,
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
}

impl<I> BufferedIterator<I>
where
    I: Iterator,
{
    pub fn new(iter: I) -> Self {
        BufferedIterator {
            iter,
            buffer: VecDeque::new(),
        }
    }

    pub fn with_capacity(iter: I, capacity: usize) -> Self {
        BufferedIterator {
            iter,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn unget(&mut self, value: I::Item) {
        self.buffer.push_front(value)
    }

    /// Peeks at the front of the iterator.
    pub fn peek(&mut self) -> Option<&I::Item> {
        if self.buffer.is_empty() {
            if let Some(t) = self.iter.next() {
                self.buffer.push_back(t)
            }
        }

        self.buffer.front()
    }

    pub fn peekn(&mut self, n: usize) -> &[I::Item] {
        while self.buffer.len() < n {
            if let Some(t) = self.iter.next() {
                self.buffer.push_back(t)
            } else {
                break;
            }
        }

        self.buffer.make_contiguous();

        let (output, _) = self.buffer.as_slices();

        &output[..std::cmp::min(n, output.len())]
    }

    pub fn peeki(&mut self, i: usize) -> Option<&I::Item> {
        self.peekn(i + 1).get(i)
    }

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
        let mut buf = numbers.iter().buffered();

        assert_eq!(buf.peek(), Some(&&0));
        assert_eq!(buf.peek(), Some(&&0));

        assert_eq!(buf.next(), Some(&0));
        assert_eq!(buf.peek(), Some(&&1));
        assert_eq!(buf.peekn(2), [&1, &2]);

        assert_eq!(buf.next(), Some(&1));
        assert_eq!(buf.peekn(3), [&2, &3, &4]);
        assert_eq!(buf.peekn(3), [&2, &3, &4]);

        assert_eq!(buf.next(), Some(&2));

        assert_eq!(buf.peekn(2), [&3, &4]);
        assert_eq!(buf.peekn(4), [&3, &4]);
        assert_eq!(buf.peekn(5), [&3, &4]);

        assert_eq!(buf.next(), Some(&3));
        assert_eq!(buf.next(), Some(&4));

        assert_eq!(buf.peek(), None);
        assert_eq!(buf.peekn(8), [] as [&i32; 0]);

        let n = 5;
        buf.unget(&n);
        assert_eq!(buf.peek(), Some(&&5));
        assert_eq!(buf.peekn(8), [&5]);
        assert_eq!(buf.next(), Some(&5));
    }

    #[test]
    fn buffered_iterator_peeki_test() {
        let numbers = [0, 1, 2, 3, 4];
        let mut buf = numbers.iter().buffered();

        assert_eq!(buf.peeki(0), Some(&&0));

        assert_eq!(buf.peeki(3), Some(&&3));

        assert_eq!(buf.next(), Some(&0));
        assert_eq!(buf.next(), Some(&1));
        assert_eq!(buf.next(), Some(&2));
        assert_eq!(buf.next(), Some(&3));
        assert_eq!(buf.next(), Some(&4));
    }

    #[test]
    fn buffered_iterator_findi_test() {
        let numbers = [0, 1, 2, 3, 4];
        let mut buf = numbers.iter().buffered();

        let i = buf.findi(|s| **s == 3);

        assert_eq!(i, Some(3));

        let i = i.unwrap();

        assert_eq!(i, 3);
        assert_eq!(buf.peeki(i), Some(&&3));

        for _ in 0..i {
            buf.next();
        }

        assert_eq!(buf.peek(), Some(&&3));
    }
}
