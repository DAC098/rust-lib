use std::sync::{RwLock, RwLockReadGuard};
use std::ptr::NonNull;
use std::fmt;

pub enum Error {
    Poisoned,
    Index,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Poisoned => f.write_str("Poisoned"),
            Error::Index => f.write_str("Index"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Poisoned => f.write_str("Poisoned"),
            Error::Index => f.write_str("Index"),
        }
    }
}

impl std::error::Error for Error {}

struct Inner<T, const N: usize> {
    list: [Option<T>; N],
    next: usize,
    oldest: usize,
    stored: usize,
}

impl<T, const N: usize> Inner<T, N> {
    #[inline]
    fn newest_index(&self) -> usize {
        if self.next == 0 {
            N - 1
        } else {
            self.next - 1
        }
    }
}

pub struct Value<'a, T, const N: usize> {
    guard: RwLockReadGuard<'a, Inner<T, N>>,
    value: NonNull<T>
}

impl<'a, T, const N: usize> Value<'a, T, N> {
    pub fn value(&self) -> &'a T {
        unsafe { self.value.as_ref() }
    }
}

impl<'a, T, const N: usize> std::ops::Deref for Value<'a, T, N> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        unsafe { self.value.as_ref() }
    }
}

pub struct RwFixed<T, const N: usize> {
    guard: RwLock<Inner<T, N>>
}

impl<T, const N: usize> RwFixed<T, N> {
    pub fn new() -> Self {
        RwFixed {
            guard: RwLock::new(Inner {
                list: std::array::from_fn(|_| None),
                next: 0,
                oldest: 0,
                stored: 0,
            })
        }
    }

    pub fn with_list(given: [T; N]) -> Self {
        let list = given.map(|v| Some(v));

        RwFixed {
            guard: RwLock::new(Inner {
                list,
                next: 0,
                oldest: 0,
                stored: N
            })
        }
    }

    pub fn with_index(given: [T; N], index: usize) -> Option<Self> {
        if index >= N {
            return None;
        }

        let oldest = if index == N - 1 {
            0
        } else {
            index + 1
        };

        Some(RwFixed {
            guard: RwLock::new(Inner {
                list: given.map(|v| Some(v)),
                next: oldest,
                oldest,
                stored: N
            })
        })
    }

    pub fn push(&self, v: T) -> Result<Option<T>, Error> {
        let mut inner = self.guard.write()
            .map_err(|_| Error::Poisoned)?;

        let rtn = {
            let index = inner.next;

            inner.list[index].replace(v)
        };

        inner.next = (inner.next + 1) % N;

        if inner.stored == N {
            inner.oldest == inner.next;
        } else {
            inner.stored += 1;
        }

        Ok(rtn)
    }

    pub fn pop(&self) -> Result<Option<T>, Error> {
        let mut inner = self.guard.write()
            .map_err(|_| Error::Poisoned)?;

        if inner.stored == 0 {
            return Ok(None);
        }

        let rtn = {
            let index = inner.oldest;

            inner.list[index].take()
        };

        inner.oldest = (inner.oldest + 1) % N;
        inner.stored -= 1;

        Ok(rtn)
    }

    pub fn newest(&self) -> Result<Option<Value<'_, T, N>>, Error> {
        let inner = self.guard.read()
            .map_err(|_| Error::Poisoned)?;

        let mut rtn = Value {
            guard: inner,
            value: NonNull::dangling()
        };

        let Some(v) = rtn.guard.list[rtn.guard.newest_index()].as_ref() else {
            return Ok(None);
        };

        rtn.value = NonNull::from(v);

        Ok(Some(rtn))
    }

    pub fn oldest(&self) -> Result<Option<Value<'_, T, N>>, Error> {
        let inner = self.guard.read()
            .map_err(|_| Error::Poisoned)?;

        let mut rtn = Value {
            guard: inner,
            value: NonNull::dangling()
        };

        let Some(v) = rtn.guard.list[rtn.guard.oldest].as_ref() else {
            return Ok(None);
        };

        rtn.value = NonNull::from(v);

        Ok(Some(rtn))
    }

    pub fn stored(&self) -> Result<usize, Error> {
        let inner = self.guard.read()
            .map_err(|_| Error::Poisoned)?;

        Ok(inner.stored)
    }

    pub fn get(&self, given: usize) -> Result<Option<Value<'_, T, N>>, Error> {
        if given >= N {
            return Err(Error::Index);
        }

        let inner = self.guard.read()
            .map_err(|_| Error::Poisoned)?;

        let mut index = inner.newest_index();

        index = if index < given {
            N - (given - index)
        } else {
            index - given
        };

        let mut rtn = Value {
            guard: inner,
            value: NonNull::dangling()
        };

        let Some(v) = rtn.guard.list[index].as_ref() else {
            return Ok(None);
        };

        rtn.value = NonNull::from(v);

        Ok(Some(rtn))
    }

    pub fn iter(&self) -> Result<Iter<'_, T, N>, Error> {
        let inner = self.guard.read()
            .map_err(|_| Error::Poisoned)?;

        let backward = inner.newest_index();
        let forward = inner.oldest;

        Ok(Iter {
            guard: inner,
            backward,
            backward_count: 0,
            forward,
            forward_count: 0
        })
    }
}

pub struct Iter<'a, T, const N: usize> {
    guard: RwLockReadGuard<'a, Inner<T, N>>,
    backward: usize,
    backward_count: usize,
    forward: usize,
    forward_count: usize,
}

// there are lifetime errors for the below iter implementations. not sure how
// best to solve this since the read guard is owned but the data underneath
// is referenced
impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.backward_count == self.guard.stored {
            return None;
        }

        let rtn = self.guard.list[self.backward].as_ref();

        if self.backward == 0 {
            self.backward = N - 1
        } else {
            self.backward -= 1
        }

        self.backward_count += 1;

        rtn
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for Iter<'a, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.forward_count == self.guard.stored {
            return None;
        }

        let rtn = self.guard.list[self.forward].as_ref();

        if self.forward == N - 1 {
            self.forward = 0;
        } else {
            self.forward += 1;
        }

        self.forward_count += 1;

        rtn
    }
}

