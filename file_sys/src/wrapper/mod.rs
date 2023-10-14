#[cfg(all(feature = "binary", feature = "serde"))]
pub mod binary;

#[cfg(all(feature = "binary", feature = "serde"))]
pub use binary::Binary;

#[cfg(all(feature = "json", feature = "serde"))]
pub mod json;

#[cfg(all(feature = "json", feature = "serde"))]
pub use json::Json;

#[cfg(all(feature = "crypto", feature = "binary", feature = "serde"))]
pub mod encrypted;

#[cfg(all(feature = "crypto", feature = "binary", feature = "serde"))]
pub use encrypted::Encrypted;

#[cfg(test)]
pub(crate) mod test {
    pub fn create_test_file<P>(path: P) -> std::fs::File
    where
        P: AsRef<std::path::Path>
    {
        std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .expect("failed to create test file")
    }
}
