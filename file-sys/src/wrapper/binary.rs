use std::path::{PathBuf, Path};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::io::Error as IoError;
use std::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Bincode(bincode::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(_) => f.write_str("Io"),
            Error::Bincode(_) => f.write_str("Bincode"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Bincode(e) => Some(e),
        }
    }
}

pub struct Binary<T> {
    inner: T,
    path: Box<Path>,
}

impl<T> Binary<T> {
    pub fn new<P>(inner: T, path: P) -> Self
    where
        P: Into<PathBuf>
    {
        let buf = path.into();

        Binary {
            inner,
            path: buf.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn set_path<P>(&mut self, path: P)
    where
        P: Into<PathBuf>
    {
        self.path = path.into().into();
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Binary<T>
where
    T: Serialize
{
    pub fn save(&self) -> Result<(), Error> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| Error::Io(e))?;
        let writer = BufWriter::new(file);

        bincode::serialize_into(writer, &self.inner)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        Ok(())
    }
}

impl<T> Binary<T>
where
    T: DeserializeOwned
{
    pub fn load<P>(given: P) -> Result<Self, Error>
    where
        P: Into<PathBuf>
    {
        let path = given.into().into();
        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|e| Error::Io(e))?;
        let reader = BufReader::new(file);

        let inner = bincode::deserialize_from(reader)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        Ok(Binary {
            inner,
            path
        })
    }
}

impl<T> std::fmt::Debug for Binary<T>
where
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Binary")
            .field("inner", &self.inner)
            .field("path", &self.path)
            .finish()
    }
}

impl<T> std::convert::AsRef<T> for Binary<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> std::convert::AsMut<T> for Binary<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Clone for Binary<T>
where
    T: Clone
{
    fn clone(&self) -> Self {
        Binary {
            inner: self.inner.clone(),
            path: self.path.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wrapper;

    #[test]
    fn base() {
        let file_name = "test.binary";
        let inner = usize::MAX;

        wrapper::test::create_test_file(file_name);

        let wrapper = Binary::new(inner, file_name);

        wrapper.save().expect("failed to save to binary file");

        let and_back: Binary<usize> = Binary::load(PathBuf::from(file_name))
            .expect("failed to load binary file");

        assert_eq!(wrapper.inner(), and_back.inner());
    }
}
