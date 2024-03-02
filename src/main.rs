use std::{env, fs, process};

use color_eyre::{
    eyre::{bail, Ok},
    Result,
};
use dict::DictEntry;
use pdfium_render::prelude::*;
use regex::Regex;

use crate::dict::Dict;

mod dict;

fn main() -> Result<()> {
    let Some(book_path) = env::args().nth(1) else {
        bail!("Please specify the path of the book")
    };

    let word_reg = Regex::new(r"^(?P<name>[a-zA-Z]+)(\s(?P<trans>[a-z]{1,4}\..*))?$").unwrap();
    let phrase_reg =
        Regex::new(r"^(?P<phrase>(\(?(([a-zA-Z/.]+\s)*[a-zA-Z/.]+)\)?\s?)+)(?P<trans>.*)").unwrap();
    let part_of_speech_reg = Regex::new(r"^[a-z]{1,4}\.").unwrap();
    let pronounciation_reg = Regex::new(r"\s?(/[^/,]+/|(\p{Co})+)").unwrap();

    let lib_path = if cfg!(debug_assertions) {
        env::current_dir()
    } else {
        env::current_exe()
    }?;

    let pdfium = Pdfium::new(Pdfium::bind_to_library(
        Pdfium::pdfium_platform_library_name_at_path(lib_path.as_path()),
    )?);

    let doc = pdfium.load_pdf_from_file(&book_path, None)?;

    let page = doc.pages().get(111).unwrap();

    let text = page.text()?.all();

    let mut entries: Vec<DictEntry> = vec![];

    let mut head = 0usize;

    for ref line in text
        .lines()
        .skip(2)
        .map(|line| pronounciation_reg.replace(line, "").to_string())
    {
        println!("{} \"{}\"", word_reg.is_match(line), line);

        if let Some(caps) = word_reg.captures(line) {
            let mut entry = DictEntry {
                name: caps["name"].to_string(),
                trans: vec![],
            };

            if let Some(trans) = caps.name("trans") {
                entry.trans.push(trans.as_str().to_string())
            }

            entries.push(entry);

            head += 1;
            continue;
        }

        let has_part_of_speech = part_of_speech_reg.is_match(line);
        let phrase_caps = phrase_reg.captures(line);

        if !has_part_of_speech && phrase_caps.is_some() {
            let caps = phrase_caps.unwrap();
            let entry = DictEntry {
                name: caps["phrase"].trim_end().to_string(),
                trans: vec![caps["trans"].to_string()],
            };

            entries.push(entry);

            head += 1;
            continue;
        }

        assert!(!entries.is_empty() && head > 0, "No entries found");

        let prev = head - 1;

        let trans = &mut entries[prev].trans;

        if has_part_of_speech || trans.is_empty() {
            trans.push(line.to_string())
        } else {
            trans.last_mut().unwrap().push_str(line)
        }
    }

    let count = entries
        .iter()
        .rev()
        .skip_while(|x| x.name.chars().any(|c| c.is_ascii_uppercase()))
        .count();
    entries.drain(count..);

    println!("{:#?}", entries);
    println!("{}", entries.len());
    println!("{}", text);

    // let dict = Dict(entries);
    // fs::write("./dict.json", serde_json::to_string_pretty(&dict)?)?;

    Ok(())
}
