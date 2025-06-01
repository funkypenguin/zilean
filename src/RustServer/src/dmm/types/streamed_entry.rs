#[derive(Debug, Clone)]
pub struct StreamedEntry {
    pub info_hash: String,
    pub name: String,
    pub size: u64,
}