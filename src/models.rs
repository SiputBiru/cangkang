#[derive(Debug, Default)]
pub struct PageMetadata {
    pub title: String,
    pub date: String,
    pub description: String,
    pub keywords: String,
    pub pinned: bool,
}

#[derive(Debug)]
pub struct PageInfo {
    pub title: String,
    pub url: String,
    pub date: String,
    pub description: String,
    pub pinned: bool,
}
