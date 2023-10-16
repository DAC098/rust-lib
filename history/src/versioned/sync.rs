use std::collections::BTreeMap;
use std::sync::{Mutex, RwLock};
use std::sync::RwLockReadGuard;
//use std::ptr::NonNull;
use std::fmt;

/*
/// reference struct for the stored value
///
/// contains the read guard from the rwlock in RwVersioned
pub struct Value<'a, T> {
    reader: RwLockReadGuard<'a, BTreeMap<u64, T>>,
    value: NonNull<T>
}

impl<'a, T> Value<'a, T> {
    /// returns reference to value
    pub fn value(&self) -> &'a T {
        unsafe { self.value.as_ref() }
    }
}

impl<'a, T> std::ops::Deref for Value<'a, T> {
    type Target = T;

    fn deref(&self) -> &'a Self::Target {
        unsafe { self.value.as_ref() }
    }
}

/// reference struct for the stored key and value
///
/// contains the read guard from the rwlock in RwVersioned
pub struct KeyValue<'a, T> {
    reader: RwLockReadGuard<'a, BTreeMap<u64, T>>,
    key: NonNull<u64>,
    value: NonNull<T>,
}

impl<'a, T> KeyValue<'a, T> {
    /// returns reference to key
    pub fn key(&self) -> &'a u64 {
        unsafe { self.key.as_ref() }
    }

    /// returns reference to value
    pub fn value(&self) -> &'a T {
        unsafe { self.value.as_ref() }
    }
}
*/

/// possible errors from methods in RwVersioned
pub enum Error {
    /// the mutex containing count has been poisoned
    CountPoisoned,
    /// the rwlock containing known versions has been poisoned
    StorePoisoned,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CountPoisoned => f.write_str("CountPoisoned"),
            Error::StorePoisoned => f.write_str("StorePoisoned"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CountPoisoned => f.write_str("CountPoisoned"),
            Error::StorePoisoned => f.write_str("StorePoisoned"),
        }
    }
}

impl std::error::Error for Error {}

/// stores changes to a given value and applies a counted number to each update
///
/// values are stored in an RwLock that contains a BTreeMap and the counted
/// version is a u64 stored behind a Mutex
pub struct RwVersioned<T> {
    store: RwLock<BTreeMap<u64, T>>,
    count: Mutex<u64>,
}

impl<T> RwVersioned<T> {
    /// creates an empty versioned struct
    pub fn new() -> Self {
        RwVersioned {
            store: RwLock::new(BTreeMap::new()),
            count: Mutex::new(0)
        }
    }

    /// retuns the next version number to use
    ///
    /// locks the count aand returns a copied value
    pub fn count(&self) -> Result<u64, Error> {
        let count_lock = self.count.lock()
            .map_err(|_| Error::CountPoisoned)?;

        Ok(*count_lock)
    }

    /// returns read guard to current store
    pub fn store(&self) -> Result<RwLockReadGuard<'_, BTreeMap<u64, T>>, Error> {
        self.store.read().map_err(|_| Error::StorePoisoned)
    }

    /// updates the value returning the version number used
    ///
    /// count will be locked first and incremented once the store has been
    /// updated
    pub fn update(&self, value: T) -> Result<u64, Error> {
        let mut count_lock = self.count.lock()
            .map_err(|_| Error::CountPoisoned)?;
        let new_version = *count_lock;

        {
            let mut store_writer = self.store.write()
                .map_err(|_| Error::StorePoisoned)?;

            store_writer.insert(new_version, value);
        }

        *count_lock += 1;

        Ok(new_version)
    }

    /// drops the desired version returning the value found
    ///
    /// only locks the store
    pub fn drop(&self, version: &u64) -> Result<Option<T>, Error> {
        let mut store_writer = self.store.write()
            .map_err(|_| Error::StorePoisoned)?;

        Ok(store_writer.remove(version))
    }

    /*
    /// returns a reference to the desired version
    ///
    /// the struct returned contains the value and RwLockReadGuard used to
    /// retrieve the value
    pub fn get(&self, version: &u64) -> Result<Option<Value<'_, T>>, Error> {
        let store_reader = self.store.read()
            .map_err(|_| Error::StorePoisoned)?;

        let mut rtn = Value {
            reader: store_reader,
            value: NonNull::dangling(),
        };

        let Some(value) = rtn.reader.get(version) else {
            return Ok(None);
        };

        rtn.value = NonNull::from(value);

        Ok(Some(rtn))
    }

    /// returns the latest version of the value
    ///
    /// similar to get in that both the value and guard are returned in the
    /// struct
    pub fn latest(&self) -> Result<Option<Value<'_, T>>, Error> {
        let store_reader = self.store.read()
            .map_err(|_| Error::StorePoisoned)?;

        let mut rtn = Value {
            reader: store_reader,
            value: NonNull::dangling(),
        };

        let Some((_, value)) = rtn.reader.last_key_value() else {
            return Ok(None);
        };

        rtn.value = NonNull::from(value);

        Ok(Some(rtn))
    }

    /// returns the latest version of the value along with the version number
    ///
    /// similar to get in that both the value and guard are returned in the
    /// struct along with the version associated with the value
    pub fn latest_version(&self) -> Result<Option<KeyValue<'_, T>>, Error> {
        let store_reader = self.store.read()
            .map_err(|_| Error::StorePoisoned)?;

        let mut rtn = KeyValue {
            reader: store_reader,
            key: NonNull::dangling(),
            value: NonNull::dangling(),
        };

        let Some((key, value)) = rtn.reader.last_key_value() else {
            return Ok(None);
        };

        rtn.key = NonNull::from(key);
        rtn.value = NonNull::from(value);

        Ok(Some(rtn))
    }
    */
}

#[cfg(feature = "serde")]
use serde::{
    ser::{
        Serialize,
        Serializer,
        SerializeStruct,
    },
    de::{
        self,
        Deserialize,
        Deserializer,
        Visitor,
        MapAccess,
        SeqAccess,
    }
};

#[cfg(feature = "serde")]
impl<T> Serialize for RwVersioned<T>
where
    T: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("RwVersioned", 2)?;
        state.serialize_field("store", &self.store)?;
        state.serialize_field("count", &self.count)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for RwVersioned<T>
where
    T: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        const STRUCT_FIELDS: &'static [&'static str] = &["store", "count"];

        enum StructField {
            Store,
            Count,
        }

        impl<'de> Deserialize<'de> for StructField {
            fn deserialize<D>(deserializer: D) -> Result<StructField, D::Error>
            where
                D: Deserializer<'de>
            {
                struct StructFieldVisitor;

                impl<'de> Visitor<'de> for StructFieldVisitor {
                    type Value = StructField;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("'store' or 'count'")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error
                    {
                        match value {
                            "store" => Ok(StructField::Store),
                            "count" => Ok(StructField::Count),
                            _ => Err(de::Error::unknown_field(value, STRUCT_FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(StructFieldVisitor)
            }
        }

        struct RwVersionedVisitor<T> {
            _type: std::marker::PhantomData<T>
        }

        impl<'de, T> Visitor<'de> for RwVersionedVisitor<T>
        where
            T: Deserialize<'de>
        {
            type Value = RwVersioned<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct RwVersioned")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>
            {
                let store = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let count = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                Ok(RwVersioned { store, count })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>
            {
                let mut store = None;
                let mut count = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        StructField::Store => {
                            if store.is_some() {
                                return Err(de::Error::duplicate_field("store"));
                            }

                            store = Some(map.next_value()?);
                        }
                        StructField::Count => {
                            if count.is_some() {
                                return Err(de::Error::duplicate_field("count"));
                            }

                            count = Some(map.next_value()?);
                        }
                    }
                }

                let store = store.ok_or_else(|| de::Error::missing_field("store"))?;
                let count = count.ok_or_else(|| de::Error::missing_field("count"))?;

                Ok(RwVersioned { store, count })
            }
        }

        deserializer.deserialize_struct(
            "RwVersioned",
            STRUCT_FIELDS,
            RwVersionedVisitor {
                _type: std::marker::PhantomData
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get() {
        let store: RwVersioned<u64> = RwVersioned::new();
        store.update(1).unwrap();
        store.update(2).unwrap();
        store.update(3).unwrap();

        let reader = store.store()
            .expect("poisoned rw lock");

        let v = reader.get(&1)
            .expect("failed to find version");

        assert_eq!(*v, 2);
    }

    #[allow(dead_code)]
    #[inline]
    fn rw_versioned_eq<T>(a: &RwVersioned<T>, b: &RwVersioned<T>)
    where
        T: PartialEq + std::fmt::Debug
    {
        {
            let a_store = a.store.read().unwrap();
            let b_store = b.store.read().unwrap();

            assert_eq!(*a_store, *b_store, "store values are not equal");
        }

        {
            let a_count = a.count.lock().unwrap();
            let b_count = b.count.lock().unwrap();

            assert_eq!(*a_count, *b_count, "count values are not equal");
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_json() {
        let versioned: RwVersioned<u64> = RwVersioned::new();
        versioned.update(5).unwrap();
        versioned.update(3).unwrap();
        versioned.update(7).unwrap();
        let drop = versioned.update(12).unwrap();
        versioned.update(9).unwrap();

        versioned.drop(&drop).unwrap();

        let to_json = serde_json::to_string(&versioned)
            .expect("failed to serialize to json string");

        let and_back: RwVersioned<u64> = serde_json::from_str(&to_json)
            .expect("failed to deserialize from json string");

        rw_versioned_eq(&versioned, &and_back);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_bincode() {
        let versioned: RwVersioned<u64> = RwVersioned::new();
        versioned.update(5).unwrap();
        versioned.update(3).unwrap();
        versioned.update(7).unwrap();
        let drop = versioned.update(12).unwrap();
        versioned.update(9).unwrap();

        versioned.drop(&drop).unwrap();

        let to_vec = bincode::serialize(&versioned)
            .expect("failed to serialize to binary");

        let and_back: RwVersioned<u64> = bincode::deserialize(&to_vec)
            .expect("failed to deserialize from binary");

        rw_versioned_eq(&versioned, &and_back);
    }
}
