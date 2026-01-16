#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowseMode {
    #[default]
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    #[default]
    Csv,
    Parquet,
    Txt,
    Excel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadMode {
    #[default]
    Lazy,
    ToMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}
