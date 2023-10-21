use std::path::{PathBuf, Path};
use std::fs::OpenOptions;
use std::io::{Read, Write, BufReader, BufWriter};
use std::io::Error as IoError;
use std::fmt;
use std::default::Default;

use serde::{Serialize, de::DeserializeOwned};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce
};
pub use chacha20poly1305::Key;

const NONCE_LEN: usize = 24;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Bincode(bincode::Error),
    Crypto,
    InvalidEncoding,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => fmt::Display::fmt(e, f),
            Error::Bincode(e) => fmt::Display::fmt(e, f),
            Error::Crypto => f.write_str("Crypto"),
            Error::InvalidEncoding => f.write_str("InvalidEncoding"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Bincode(e) => Some(e),
            _ => None
        }
    }
}

fn encode_data(nonce: XNonce, data: Vec<u8>) -> Vec<u8> {
    let mut rtn: Vec<u8> = Vec::with_capacity(NONCE_LEN + data.len());
    rtn.extend(nonce);
    rtn.extend(data);

    rtn
}

fn decode_data(data: Vec<u8>) -> Result<(XNonce, Vec<u8>), Error> {
    if data.len() < 24 {
        return Err(Error::InvalidEncoding);
    }

    let mut nonce = [0; NONCE_LEN];
    let mut encrypted = Vec::with_capacity(data.len() - NONCE_LEN);
    let mut iter = data.into_iter();

    for i in 0..24 {
        if let Some(b) = iter.next() {
            nonce[i] = b;
        } else {
            return Err(Error::InvalidEncoding);
        }
    }

    while let Some(b) = iter.next() {
        encrypted.push(b);
    }

    Ok((nonce.into(), encrypted))
}

fn encrypt_data(key: &Key, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let cipher = XChaCha20Poly1305::new(&key);

    let encrypted = cipher.encrypt(&nonce, data.as_slice())
        .map_err(|_| Error::Crypto)?;

    Ok(encode_data(nonce, encrypted))
}

fn decrypt_data(key: &Key, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let (nonce, encrypted) = decode_data(data)?;

    let cipher = XChaCha20Poly1305::new(&key);
    let decrypted = cipher.decrypt(&nonce, encrypted.as_slice())
        .map_err(|_| Error::Crypto)?;

    Ok(decrypted)
}

pub struct Encrypted<T> {
    inner: T,
    path: Box<Path>,
    key: Key,
}

impl<T> Encrypted<T> {
    /// creates a new Encrypted with the provided data
    ///
    /// no checks are made on the path to ensure that the file exists
    pub fn new<P, K>(inner: T, path: P, key: K) -> Self
    where
        P: Into<PathBuf>,
        K: Into<Key>
    {
        Encrypted {
            inner,
            path: path.into().into(),
            key: key.into(),
        }
    }

    #[inline]
    fn touch_file(path: &Path) -> Result<(), Error> {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }

    /// creates a new Encrypted with the provided data and makes the file
    ///
    /// will attempt to create a new file and throw an error if a file already
    /// exists
    pub fn create<P, K>(inner: T, path: P, key: K) -> Result<Self, Error>
    where
        P: Into<PathBuf>,
        K: Into<Key>
    {
        let path: Box<Path> = path.into().into();
        let key = key.into();

        Self::touch_file(&path)?;

        Ok(Encrypted {
            inner,
            path,
            key
        })
    }

    /// returns the current path for the wrapper
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// updates the current path to the provided value
    pub fn set_path<P>(&mut self, given: P)
    where
        P: Into<PathBuf>
    {
        self.path = given.into().into()
    }

    /// returns the current key for encrypting the file data
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// updates the current key for encrypting the file data
    pub fn set_key<K>(&mut self, key: K)
    where
        K: Into<Key>
    {
        self.key = key.into();
    }

    /// returns the inner value
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// returns a mutable inner value
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// consumes the struct returning the inner value
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Encrypted<T>
where
    T: Serialize
{
    /// saves the inner value to the provided file path
    ///
    /// data will be encrypted using the key stored and the file will be
    /// truncated when written to
    pub fn save(&self) -> Result<(), Error> {
        let serialize = bincode::serialize(&self.inner)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        let encrypted = encrypt_data(&self.key, serialize)?;

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| Error::Io(e))?;
        let mut writer = BufWriter::new(file);

        writer.write_all(encrypted.as_slice())
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }

    /// saves the inner value to the provided file path using tokio fs
    ///
    /// similar operation as the blocking save
    #[cfg(feature = "tokio")]
    pub async fn save_async(&self) -> Result<(), Error> {
        use tokio::io::AsyncWriteExt;

        let serialize = bincode::serialize(&self.inner)
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e)
            })?;

        let encrypted = encrypt_data(&self.key, serialize)?;

        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await
            .map_err(|e| Error::Io(e))?;
        let mut writer = tokio::io::BufWriter::new(file);

        writer.write_all(encrypted.as_slice())
            .await
            .map_err(|e| Error::Io(e))?;
        writer.flush()
            .await
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }
}

impl<T> Encrypted<T>
where
    T: DeserializeOwned
{
    fn read_to_buffer(path: &Path) -> Result<Vec<u8>, Error> {
        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|e| Error::Io(e))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        reader.read_to_end(&mut buffer)
            .map_err(|e| Error::Io(e))?;

        Ok(buffer)
    }

    fn decrypt_deserialize(key: &Key, buffer: Vec<u8>) -> Result<T, Error> {
        let decrypted = decrypt_data(&key, buffer)?;

        bincode::deserialize(decrypted.as_slice())
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e),
            })
    }

    /// loads the specified file using the master key provided
    ///
    /// assumes that the file already exists and is propperly encoded with the
    /// encrypted data
    pub fn load<P, K>(given: P, master_key: K) -> Result<Self, Error>
    where
        P: Into<PathBuf>,
        K: Into<Key>,
    {
        let path: Box<Path> = given.into().into();
        let key = master_key.into();

        let buffer = Self::read_to_buffer(&path)?;
        let inner = Self::decrypt_deserialize(&key, buffer)?;

        Ok(Encrypted {
            inner,
            path,
            key
        })
    }

    /// loads or creates the specified file using the master key provided
    ///
    /// if the file already exits it will follow the same operation as load
    /// otherwise it will attempt to create an empty file.
    pub fn load_create<P, K>(path: P, master_key: K) -> Result<Self, Error>
    where
        T: Default,
        P: Into<PathBuf>,
        K: Into<Key>,
    {
        let path: Box<Path> = path.into().into();
        let key = master_key.into();
        let check = path.try_exists()
            .map_err(|e| Error::Io(e))?;

        if check {
            let buffer = Self::read_to_buffer(&path)?;
            let inner = Self::decrypt_deserialize(&key, buffer)?;

            Ok(Encrypted {
                inner,
                path,
                key
            })
        } else {
            Self::touch_file(&path)?;

            Ok(Encrypted {
                inner: Default::default(),
                path,
                key
            })
        }
    }

    /// loads the specified file using the master key provided using tokio fs
    ///
    /// similar to the blocking load
    #[cfg(feature = "tokio")]
    pub async fn load_async<P, K>(given: P, master_key: K) -> Result<Self, Error>
    where
        P: Into<PathBuf>,
        K: Into<Key>,
    {
        use tokio::io::AsyncReadExt;

        let path = given.into().into();
        let key = master_key.into();

        let file = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&path)
            .await
            .map_err(|e| Error::Io(e))?;
        let mut reader = tokio::io::BufReader::new(file);
        let mut buffer = Vec::new();

        reader.read_to_end(&mut buffer)
            .await
            .map_err(|e| Error::Io(e))?;

        let decrypted = decrypt_data(&key, buffer)?;

        let inner = bincode::deserialize(decrypted.as_slice())
            .map_err(|e| match *e {
                bincode::ErrorKind::Io(io) => Error::Io(io),
                _ => Error::Bincode(e),
            })?;

        Ok(Encrypted {
            inner,
            path,
            key
        })
    }
}

impl<T> std::fmt::Debug for Encrypted<T>
where
    T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Encrypted")
            .field("inner", &self.inner)
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl<T> std::convert::AsRef<T> for Encrypted<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> std::convert::AsMut<T> for Encrypted<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Clone for Encrypted<T>
where
    T: Clone
{
    fn clone(&self) -> Self {
        Encrypted {
            inner: self.inner.clone(),
            path: self.path.clone(),
            key: self.key.clone()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wrapper;

    #[test]
    fn base() {
        let file_name = "test.encrypted";
        let inner = usize::MAX;
        let key = [0; 32];

        wrapper::test::create_test_file(file_name);

        let wrapper = Encrypted::new(inner, file_name, key);

        wrapper.save().expect("failed to save to encrypted file");

        let and_back: Encrypted<usize> = Encrypted::load(
            PathBuf::from(file_name),
            key
        ).expect("failed to load encrypted file");

        assert_eq!(wrapper.inner(), and_back.inner());
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn tokio() {
        let file_name = "test.tokio.encrypted";
        let inner = usize::MAX;
        let key = [0; 32];

        wrapper::test::create_test_file(file_name);

        let wrapper = Encrypted::new(inner, file_name, key);

        wrapper.save_async()
            .await
            .expect("failed to save to tokio encrypted file");

        let and_back: Encrypted<usize> = Encrypted::load_async(file_name, key)
            .await
            .expect("failed to load tokio encrypted file");

        assert_eq!(wrapper.inner(), and_back.inner());
    }
}
