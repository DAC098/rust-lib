use std::collections::BTreeMap;
use std::collections::btree_map::Iter;
use std::fmt;

//pub mod sync;

/// stores changes to a given value and applies a counted number to each update
///
/// values are stored in a BTreeMap and the counted version is a u64
pub struct Versioned<T> {
    store: BTreeMap<u64, T>,
    count: u64
}

impl<T> Versioned<T> {
    /// creates an empty versioned struct
    pub fn new() -> Self {
        Versioned {
            store: BTreeMap::new(),
            count: 0
        }
    }

    /// returns next version number to use
    pub fn count(&self) -> &u64 {
        &self.count
    }

    /// returns reference to current store
    pub fn store(&self) -> &BTreeMap<u64, T> {
        &self.store
    }

    /// returns total stored values in the store
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// updates the value returning the version number used
    pub fn update(&mut self, value: T) -> u64 {
        let version = self.count;
        self.count += 1;

        self.store.insert(version, value);

        version
    }

    /// drops the desired version returning the value found
    pub fn remove(&mut self, version: &u64) -> Option<T> {
        self.store.remove(version)
    }

    /// returns a reference to the desired version
    pub fn get(&self, version: &u64) -> Option<&T> {
        self.store.get(version)
    }

    /// returns the latest version of the value
    pub fn latest(&self) -> Option<&T> {
        self.store.last_key_value().map(|(_, v)| v)
    }

    /// returns the latest version of the value along with the version number
    pub fn latest_version(&self) -> Option<(&u64, &T)> {
        self.store.last_key_value()
    }

    /// returns a BTreeMap Iter
    pub fn iter(&self) -> Iter<'_, u64, T> {
        self.store.iter()
    }
}

impl<T> fmt::Debug for Versioned<T>
where
    T: fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Versioned")
            .field("store", &self.store)
            .field("count", &self.count)
            .finish()
    }
}

impl<T> Clone for Versioned<T>
where
    T: Clone
{
    fn clone(&self) -> Self {
        Versioned {
            store: self.store.clone(),
            count: self.count.clone(),
        }
    }
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
impl<T> Serialize for Versioned<T>
where
    T: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("Versioned", 2)?;
        state.serialize_field("store", &self.store)?;
        state.serialize_field("count", &self.count)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for Versioned<T>
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
            Count
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

        struct VersionedVisitor<T> {
            _type: std::marker::PhantomData<T>
        }

        impl<'de, T> Visitor<'de> for VersionedVisitor<T>
        where
            T: Deserialize<'de>
        {
            type Value = Versioned<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Versioned")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>
            {
                let store = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let count = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                Ok(Versioned { store, count })
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

                Ok(Versioned { store, count })
            }
        }

        deserializer.deserialize_struct(
            "Versioned",
            STRUCT_FIELDS,
            VersionedVisitor {
                _type: std::marker::PhantomData
            }
        )
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn serde_json() {
        let mut versioned: Versioned<u64> = Versioned::new();
        versioned.update(5);
        versioned.update(3);
        versioned.update(7);
        let drop = versioned.update(12);
        versioned.update(9);

        versioned.remove(&drop);

        let to_json = serde_json::to_string(&versioned)
            .expect("failed to serialize to json string");

        let and_back: Versioned<u64> = serde_json::from_str(&to_json)
            .expect("failed to deserialize from json string");

        assert_eq!(versioned.store, and_back.store, "store values are not equal");
        assert_eq!(versioned.count, and_back.count, "count values are not equal");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_bincode() {
        let mut versioned: Versioned<u64> = Versioned::new();
        versioned.update(5);
        versioned.update(3);
        versioned.update(7);
        let drop = versioned.update(12);
        versioned.update(9);

        versioned.remove(&drop);

        let to_vec = bincode::serialize(&versioned)
            .expect("failed to serialize to binary");

        let and_back: Versioned<u64> = bincode::deserialize(&to_vec)
            .expect("failed to deserialize from binary");

        assert_eq!(versioned.store, and_back.store, "store values are not equal");
        assert_eq!(versioned.count, and_back.count, "count values are not equal");
    }
}
