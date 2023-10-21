/// a fixed value cirular buffer
///
/// made with the intention of being used to store previous versions of a
/// value. when the list has been filled and a new value is pushed the
/// oldest value will be returned
pub struct Fixed<T, const N: usize> {
    /// array to store provided values
    list: [Option<T>; N],
    /// the index of the next to write to. the "newest" value is next - 1 and
    /// loops back to 0 if next == 0
    next: usize,
    /// the index of the currently known oldest value
    oldest: usize,
    /// total number of values stored
    stored: usize,
}

impl<T, const N: usize> Fixed<T, N> {
    /// creates empty Fixed
    pub fn new() -> Self {
        Fixed {
            list: std::array::from_fn(|_| None),
            next: 0,
            oldest: 0,
            stored: 0,
        }
    }

    /// creates a Fixed list with then provided list
    ///
    /// newest index will be the N - 1 and the oldest index will be 0
    pub fn with_list(given: [T; N]) -> Self {
        let list = given.map(|v| Some(v));

        Fixed {
            list,
            next: 0,
            oldest: 0,
            stored: N,
        }
    }

    /// creates a Fixed list with the provided list and given newest index
    ///
    /// the provided index will be considered the newest value and the oldest
    /// will be calculated from that index.
    ///
    /// a check will be made to ensure that the given index is with in the
    /// bounds of the array. if it is not then None will be returned
    pub fn with_index(given: [T; N], index: usize) -> Option<Self> {
        if index >= N {
            return None;
        }

        let oldest = if index == N - 1 {
            0
        } else {
            index + 1
        };

        Some(Fixed {
            list: given.map(|v| Some(v)),
            next: oldest,
            oldest,
            stored: N,
        })
    }

    /// pushes a new value to the next newest position
    ///
    /// if a value existed in the new position then it will be returned.
    pub fn push(&mut self, v: T) -> Option<T> {
        let rtn = self.list[self.next].replace(v);

        self.next = (self.next + 1) % N;

        if self.stored == N {
            self.oldest = self.next;
        } else {
            self.stored += 1;
        }

        rtn
    }

    /// pops the oldest value from the list
    pub fn pop(&mut self) -> Option<T> {
        if self.stored == 0 {
            return None;
        }

        let rtn = self.list[self.oldest].take();

        self.oldest = (self.oldest + 1) % N;
        self.stored -= 1;

        rtn
    }

    #[inline]
    fn newest_index(&self) -> usize {
        if self.next == 0 {
            N - 1
        } else {
            self.next - 1
        }
    }

    /// returns the current newest value
    pub fn newest(&self) -> Option<&T> {
        self.list[self.newest_index()].as_ref()
    }

    /// returns the current oldest value
    pub fn oldest(&self) -> Option<&T> {
        self.list[self.oldest].as_ref()
    }

    /// total amount of stored values
    pub fn stored(&self) -> usize {
        self.stored
    }

    /// retrieves the a value at the given index
    ///
    /// value returned is calculated from the newest index
    pub fn get(&self, given: usize) -> Result<Option<&T>, ()> {
        if given >= N {
            return Err(())
        }

        // index will be the current newest index
        let mut index = self.newest_index();

        // override it with the index that is requested
        index = if index < given {
            N - (given - index)
        } else {
            index - given
        };

        Ok(self.list[index].as_ref())
    }

    /// returns an iterator for the Fixed list
    pub fn iter(&self) -> FixedIter<T, N> {
        FixedIter {
            working: self,
            backward: self.newest_index(),
            backward_count: 0,
            forward: self.oldest,
            forward_count: 0,
        }
    }
}

impl<T, const N: usize> std::default::Default for Fixed<T, N> {
    #[inline]
    fn default() -> Self {
        Fixed::new()
    }
}

impl<T, const N: usize> Clone for Fixed<T, N>
where
    T: Clone
{
    fn clone(&self) -> Self {
        Fixed {
            list: self.list.clone(),
            next: self.next,
            oldest: self.oldest,
            stored: self.stored,
        }
    }
}

impl<T, const N: usize> std::fmt::Debug for Fixed<T, N>
where
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fixed")
            .field("list", &self.list)
            .field("next", &self.next)
            .field("oldest", &self.oldest)
            .field("stored", &self.stored)
            .finish()
    }
}

/// iterator for Fixed
///
/// this implements Iterator and DoubleEndedIterator. Iterator starts with the
/// newest value and goes to the oldest. DoubleEndedIterator starts with the
/// oldest value and goes to the newest.
pub struct FixedIter<'a, T, const N: usize> {
    working: &'a Fixed<T, N>,
    backward: usize,
    backward_count: usize,
    forward: usize,
    forward_count: usize,
}

impl<'a, T, const N: usize> Iterator for FixedIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.backward_count == self.working.stored {
            return None;
        }

        let rtn = self.working.list[self.backward].as_ref();

        if self.backward == 0 {
            self.backward = N - 1
        } else {
            self.backward -= 1
        }

        self.backward_count += 1;

        rtn
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for FixedIter<'a, T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.forward_count == self.working.stored {
            return None;
        }

        let rtn = self.working.list[self.forward].as_ref();

        if self.forward == N - 1 {
            self.forward = 0;
        } else {
            self.forward += 1;
        }

        self.forward_count += 1;

        rtn
    }
}

impl<'a, T, const N: usize> Clone for FixedIter<'a, T, N> {
    fn clone(&self) -> Self {
        FixedIter {
            working: self.working,
            backward: self.backward,
            backward_count: self.backward_count,
            forward: self.forward,
            forward_count: self.forward_count,
        }
    }
}

impl<'a, T, const N: usize> std::fmt::Debug for FixedIter<'a, T, N>
where
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FixedIter")
            .field("working", &self.working)
            .field("backward", &self.backward)
            .field("backward_count", &self.backward_count)
            .field("forward", &self.forward)
            .field("forward_count", &self.forward_count)
            .finish()
    }
}

#[cfg(feature = "serde")]
use serde::{
    ser::{
        Serialize,
        Serializer,
        SerializeStruct
    },
    de::{
        self,
        Deserialize,
        Deserializer,
        Visitor,
        MapAccess,
        SeqAccess
    }
};

#[cfg(feature = "serde")]
impl<T, const N: usize> Serialize for Fixed<T, N>
where
    T: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("Fixed", 4)?;
        state.serialize_field("list", &self.list.as_slice())?;
        state.serialize_field("next", &self.next)?;
        state.serialize_field("oldest", &self.oldest)?;
        state.serialize_field("stored", &self.stored)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T, const N: usize> Deserialize<'de> for Fixed<T, N>
where
    T: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        const STRUCT_FIELDS: &'static [&'static str] = &["list", "next", "oldest", "stored"];

        enum KeyField {
            List,
            Next,
            Oldest,
            Stored,
        }

        impl<'de> Deserialize<'de> for KeyField {
            fn deserialize<D>(deserializer: D) -> Result<KeyField, D::Error>
            where
                D: Deserializer<'de>
            {
                struct KeyFieldVisitor;

                impl<'de> Visitor<'de> for KeyFieldVisitor {
                    type Value = KeyField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("'list' for 'index'")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error
                    {
                        match value {
                            "list" => Ok(KeyField::List),
                            "next" => Ok(KeyField::Next),
                            "oldest" => Ok(KeyField::Oldest),
                            "stored" => Ok(KeyField::Stored),
                            _ => Err(de::Error::unknown_field(value, STRUCT_FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(KeyFieldVisitor)
            }
        }

        struct FixedVisitor<T, const N: usize> {
            _type: std::marker::PhantomData<T>
        }

        impl<'de, T, const N: usize> Visitor<'de> for FixedVisitor<T, N>
        where
            T: Deserialize<'de>
        {
            type Value = Fixed<T, N>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Fixed")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>
            {
                let list = {
                    let t: Vec<Option<T>> = seq.next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                    t.try_into().map_err(|_| de::Error::invalid_length(N, &self))?
                };

                let next = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let oldest = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let stored = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                Ok(Fixed { list, next, oldest, stored })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>
            {
                let mut list = None;
                let mut next = None;
                let mut oldest = None;
                let mut stored = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        KeyField::List => {
                            if list.is_some() {
                                return Err(de::Error::duplicate_field("list"));
                            }

                            let t: Vec<Option<T>> = map.next_value()?;

                            list = Some(t.try_into().map_err(|_| de::Error::invalid_length(N, &self))?)
                        }
                        KeyField::Next => {
                            if next.is_some() {
                                return Err(de::Error::duplicate_field("next"));
                            }

                            next = Some(map.next_value()?);
                        }
                        KeyField::Oldest => {
                            if oldest.is_some() {
                                return Err(de::Error::duplicate_field("oldest"));
                            }

                            oldest = Some(map.next_value()?);
                        }
                        KeyField::Stored => {
                            if stored.is_some() {
                                return Err(de::Error::duplicate_field("stored"));
                            }

                            stored = Some(map.next_value()?);
                        }
                    }
                }

                let list = list.ok_or_else(|| de::Error::missing_field("list"))?;
                let next = next.ok_or_else(|| de::Error::missing_field("next"))?;
                let oldest = oldest.ok_or_else(|| de::Error::missing_field("oldest"))?;
                let stored = stored.ok_or_else(|| de::Error::missing_field("stored"))?;

                Ok(Fixed { list, next, oldest, stored })
            }
        }

        deserializer.deserialize_struct(
            "Fixed",
            STRUCT_FIELDS,
            FixedVisitor {
                _type: std::marker::PhantomData
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push() {
        let mut list: Fixed<u8, 3> = Fixed::new();

        assert_eq!(list.push(1), None);
        assert_eq!(list.push(2), None);
        assert_eq!(list.push(3), None);
        assert_eq!(list.push(4), Some(1));
        assert_eq!(list.push(5), Some(2));
    }

    #[test]
    fn pop() {
        let mut list = Fixed::with_index([3u8,4,5,1,2], 2).unwrap();

        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn newest() {
        let values: Fixed<u8, 5> = Fixed::with_list([1u8,2,3,4,5]);

        assert_eq!(values.newest(), Some(&5));
    }

    #[test]
    fn oldest() {
        let values: Fixed<u8, 5> = Fixed::with_list([1,2,3,4,5]);

        assert_eq!(values.oldest(), Some(&1));
    }

    #[test]
    fn get() {
        let original = [1u8,2,3,4,5];
        let values = Fixed::with_list(original);

        assert_eq!(values.get(0), Ok(Some(&5)));
        assert_eq!(values.get(1), Ok(Some(&4)));
        assert_eq!(values.get(2), Ok(Some(&3)));
        assert_eq!(values.get(3), Ok(Some(&2)));
        assert_eq!(values.get(4), Ok(Some(&1)));
        assert_eq!(values.get(5), Err(()));
    }

    #[test]
    fn comprehensive() {
        let mut list: Fixed<u8, 5> = Fixed::new();

        assert_eq!(list.push(1), None, "push value. {:#?}", list);
        assert_eq!(list.push(2), None, "push value. {:#?}", list);
        assert_eq!(list.push(3), None, "push value. {:#?}", list);
        assert_eq!(list.pop(), Some(1), "pop value. {:#?}", list);
        assert_eq!(list.stored(), 2, "stored value. {:#?}", list);
        assert_eq!(list.push(4), None, "push value. {:#?}", list);
        assert_eq!(list.push(5), None, "push value. {:#?}", list);
        assert_eq!(list.push(6), None, "push value. {:#?}", list);
        assert_eq!(list.push(7), Some(2), "push value. {:#?}", list);
        assert_eq!(list.oldest(), Some(&3), "oldest value. {:#?}", list);
        assert_eq!(list.stored(), 5, "stored value. {:#?}", list);
        assert_eq!(list.pop(), Some(3), "pop value. {:#?}", list);
        assert_eq!(list.pop(), Some(4), "pop value. {:#?}", list);
        assert_eq!(list.pop(), Some(5), "pop value. {:#?}", list);
        assert_eq!(list.oldest(), Some(&6), "oldest value. {:#?}", list);
        assert_eq!(list.newest(), Some(&7), "newest value. {:#?}", list);
        assert_eq!(list.pop(), Some(6), "pop value. {:#?}", list);
        assert_eq!(list.oldest(), Some(&7), "oldest value. {:#?}", list);
        assert_eq!(list.newest(), Some(&7), "newest value. {:#?}", list);
        assert_eq!(list.pop(), Some(7), "pop value. {:#?}", list);
        assert_eq!(list.stored(), 0, "stored value. {:#?}", list);
        assert_eq!(list.newest(), None, "newest value. {:#?}", list);
        assert_eq!(list.oldest(), None, "oldest value. {:#?}", list);
    }

    #[test]
    fn iterator_full() {
        let values = Fixed::with_index([6u8,7,8,9,4,5], 3).unwrap();
        let mut values_iter = values.iter();

        assert_eq!(values_iter.next(), Some(&9));
        assert_eq!(values_iter.next(), Some(&8));
        assert_eq!(values_iter.next(), Some(&7));
        assert_eq!(values_iter.next(), Some(&6));
        assert_eq!(values_iter.next(), Some(&5));
        assert_eq!(values_iter.next(), Some(&4));
        assert_eq!(values_iter.next(), None);
    }

    #[test]
    fn iterator_backward_full() {
        let values = Fixed::with_index([6u8,7,8,9,4,5], 3).unwrap();
        let mut values_iter = values.iter().rev();

        assert_eq!(values_iter.next(), Some(&4));
        assert_eq!(values_iter.next(), Some(&5));
        assert_eq!(values_iter.next(), Some(&6));
        assert_eq!(values_iter.next(), Some(&7));
        assert_eq!(values_iter.next(), Some(&8));
        assert_eq!(values_iter.next(), Some(&9));
        assert_eq!(values_iter.next(), None);
    }

    #[test]
    fn iterator_partial() {
        let mut values: Fixed<u8, 5> = Fixed::new();

        for v in 0..3 {
            values.push(v);
        }

        let mut values_iter = values.iter();

        assert_eq!(values_iter.next(), Some(&2));
        assert_eq!(values_iter.next(), Some(&1));
        assert_eq!(values_iter.next(), Some(&0));
        assert_eq!(values_iter.next(), None);
    }

    #[test]
    fn iterator_backward_partial() {
        let mut values: Fixed<u8, 5> = Fixed::new();

        for v in 0..3 {
            values.push(v);
        }

        let mut values_iter = values.iter().rev();

        assert_eq!(values_iter.next(), Some(&0));
        assert_eq!(values_iter.next(), Some(&1));
        assert_eq!(values_iter.next(), Some(&2));
        assert_eq!(values_iter.next(), None);
    }

    #[test]
    fn iterator_single() {
        let mut values: Fixed<u8, 5> = Fixed::new();
        values.push(0);
        let mut values_iter = values.iter();

        assert_eq!(values_iter.next(), Some(&0));
        assert_eq!(values_iter.next(), None);
    }

    #[test]
    fn iterator_backward_single() {
        let mut values: Fixed<u8, 5> = Fixed::new();
        values.push(0);
        let mut values_iter = values.iter().rev();

        assert_eq!(values_iter.next(), Some(&0));
        assert_eq!(values_iter.next(), None);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_json() {
        const SIZE: usize = 5;

        let original: Fixed<u8, SIZE> = Fixed {
            list: [Some(1), Some(2), Some(3), None, None],
            next: 3,
            oldest: 0,
            stored: 3,
        };

        let to_json = serde_json::to_string(&original)
            .expect("failed to serialize to json string");

        let and_back: Fixed<u8, SIZE> = serde_json::from_str(&to_json)
            .expect("failed to deserialize from json string");

        assert_eq!(original.list, and_back.list, "list values are not equal");
        assert_eq!(original.next, and_back.next, "next values are not equal");
        assert_eq!(original.oldest, and_back.oldest, "oldest values are not equal");
        assert_eq!(original.stored, and_back.stored, "stored values are not equal");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_bincode() {
        const SIZE: usize = 5;

        let original: Fixed<u8, SIZE> = Fixed {
            list: [Some(1), Some(2), Some(3), None, None],
            next: 3,
            oldest: 0,
            stored: 3,
        };

        let to_vec = bincode::serialize(&original)
            .expect("failed to serialize to binary");

        let and_back: Fixed<u8, SIZE> = bincode::deserialize(&to_vec)
            .expect("failed to deserialize from binary");

        assert_eq!(original.list, and_back.list, "list values are not equal");
        assert_eq!(original.next, and_back.next, "next values are not equal");
        assert_eq!(original.oldest, and_back.oldest, "oldest values are not equal");
        assert_eq!(original.stored, and_back.stored, "stored values are not equal");
    }
}
