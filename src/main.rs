use std::{env, fs, io::{stdout, Write}, path::PathBuf, process};

use color_eyre::{
    eyre::{bail, Ok},
    Result,
};
use pdfium_render::prelude::*;
use regex::Regex;

use crate::{cli::Args, dict::Dict, extractor::Extractor};

mod cli;
mod dict;
mod extractor;
mod macros;

fn main() -> Result<()> {
    // panic!("{:X}", '，' as u32);
    let args: Args = argh::from_env();

    println!("{:?}", args);

    extractor::extract(args)
    // let dict = Extractor::init_dynamic(args)?.extract()?;

    // fs::write("./dict.json", serde_json::to_string_pretty(&dict)?)?;
}
