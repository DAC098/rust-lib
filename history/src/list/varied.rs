// this is not as efficient as Fixed and you should probably use that instead
// if your buffer is a fixed size
pub struct Varied<T> {
    list: Vec<T>,
    index: usize,
}

impl<T> Varied<T> {
    pub fn new() -> Self {
        Varied {
            list: Vec::new(),
            index: 0,
        }
    }

    pub fn with_list(list: Vec<T>) -> Self {
        Varied {
            list,
            index: 0
        }
    }

    pub fn with_index(given: Vec<T>, index: usize) -> Option<Self> {
        if index > given.capacity() {
            return None;
        }

        if given.len() == given.capacity() {
            Some(Varied {
                list: given,
                index
            })
        } else {
            Some(Varied{
                list: given,
                index: 0
            })
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Varied {
            list: Vec::with_capacity(capacity),
            index: 0
        }
    }

    pub fn push(&mut self, mut v: T) -> Option<T> {
        if self.list.len() == self.list.capacity() {
            std::mem::swap(&mut self.list[self.index], &mut v);

            self.index = (self.index + 1) % self.list.len();

            Some(v)
        } else {
            self.list.push(v);

            None
        }
    }

    pub fn newest(&self) -> Option<&T> {
        if self.list.len() == 0 {
            None
        } else if self.list.len() == self.list.capacity() {
            if self.index == 0 {
                Some(&self.list[self.list.len() - 1])
            } else {
                Some(&self.list[self.index - 1])
            }
        } else {
            Some(&self.list[self.list.len() - 1])
        }
    }

    pub fn oldest(&self) -> Option<&T> {
        if self.list.len() == 0 {
            None
        } else if self.list.len() == self.list.capacity() {
            Some(&self.list[self.index])
        } else {
            Some(&self.list[0])
        }
    }

    pub fn iter(&self) -> VariedIter<T> {
        VariedIter {
            working: self,
            count: self.list.len()
        }
    }

    pub fn grow(&mut self, amount: usize) {
        let len = self.list.len();

        if len == self.list.capacity() {
            if self.index < len / 2 {
                self.list.rotate_left(self.index);
            } else {
                self.list.rotate_right(len - self.index);
            }
        }

        self.list.reserve_exact(amount);
    }

    pub fn shrink(&mut self, amount: usize) -> Vec<T> {
        let new_capacity = self.list.capacity() - amount;

        let rtn = if new_capacity < self.list.len() {
            let mut dropping = self.list.len() - new_capacity;
            let mut popped = Vec::with_capacity(dropping);

            rotate_to_position(&mut self.list, self.index, new_capacity);

            while dropping != 0 {
                popped.push(self.list.pop().unwrap());
                dropping -= 1;
            }

            popped
        } else {
            Vec::new()
        };

        rtn
    }
}

fn diff(a: usize, b: usize) -> (usize, bool) {
    if a > b {
        (a - b, true)
    } else {
        (b - a, false)
    }
}

fn rotate_to_position<T>(given: &mut [T], index: usize, position: usize) {
    let (diff, to_left) = diff(index, position);

    if diff < given.len() / 2 {
        if to_left {
            given.rotate_left(diff);
        } else {
            given.rotate_right(diff);
        }
    } else {
        if to_left {
            given.rotate_right(given.len() - diff);
        } else {
            given.rotate_left(given.len() - diff);
        }
    }
}

pub struct VariedIter<'a, T> {
    working: &'a Varied<T>,
    count: usize,
}

impl<'a, T> Iterator for VariedIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;

            let index = (self.working.index + self.count) % self.working.list.len();

            Some(&self.working.list[index])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let values = [1u8,2,3,4,5,6,7,8,9,10,11,12];

        let mut size: usize = 3;
        let mut expected = (&values[..(values.len() - size)]).into_iter();

        let mut list: Varied<u8> = Varied::with_capacity(size);

        for v in &values {
            if size > 0 {
                size -= 1;

                list.push(*v);
            } else {
                let old = list.push(*v).expect("oldest value was not provided");
                let check = expected.next().expect("end of expected values");

                assert_eq!(*check, old, "invalid old value. expected: {} given: {}", check, old);
            }
        }
    }

    #[test]
    fn newest() {
        let values = Varied::with_list(vec![1,2,3,4,5]);

        let newest = values.newest().expect("newest value was not provided");

        assert_eq!(*newest, 5, "unexpected newest value. expected: 5, given: {}", newest);
    }

    #[test]
    fn oldest() {
        let values = Varied::with_list(vec![1,2,3,4,5]);

        let oldest = values.oldest().expect("oldest value was not provided");

        assert_eq!(*oldest, 1, "unexpected oldest value. expected: 1, given: {}", oldest);
    }

    #[test]
    fn iterator() {
        let values = Varied::with_index(vec![6u8,7,8,9,1,2,3,4,5], 4).unwrap();
        let expected = [9u8,8,7,6,5,4,3,2,1];

        assert!(values.iter().eq(&expected), "iterator values in unexpected order");
    }
}
