use clap::ValueEnum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OrderByEnum {
    /// ascending
    Asc,
    /// descending
    Desc,
}
