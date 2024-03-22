use argh::FromArgs;
use std::{ops::RangeInclusive, path::PathBuf};

#[derive(FromArgs, Debug)]
/// Application arguments
pub struct Args {
    /// path to the book
    #[argh(positional)]
    pub path: PathBuf,
    #[argh(positional)]
    pub start: u8,
    #[argh(positional)]
    pub end: u8,
}
