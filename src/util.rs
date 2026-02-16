/// # Example
/// ```
/// let size = mb!(100); // 100 MB.
/// ```
#[macro_export]
macro_rules! mb {
    ($x:expr) => {
        ($x as u64) * 1024 * 1024
    };
}