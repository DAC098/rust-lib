#[cfg(all(feature = "binary", feature = "serde"))]
pub mod binary;

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
