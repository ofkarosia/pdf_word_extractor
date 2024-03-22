use std::{env, ops::RangeInclusive, path::PathBuf};

use color_eyre::eyre::Result;
use pdfium_render::pdfium::Pdfium;
use regex::Regex;

use crate::{cli::Args, dict::{ Dict, Entry, Kind}, lazy};

// ^[a-z]+\ ?([\p{Co}/]+)(.*$)
// Any word starts with uppercase will be ignored

// "[a-z]{1,4}\.( & [a-z]{1,4}\.| \[pl\.\])? ([a-z \uff08\u3001\uff1b\u2026]?[\u4e00-\u9fff]+[\uff09\u3001\uff1b\u2026\uff0c]?)*"gm

lazy! {
    static WORD_RE: Regex = Regex::new(r"^(?P<word>[a-z]+)\s?([\p{Co}/]+)(?P<rest>.*)").unwrap();
    static PRONOUNCIATION_REG: Regex = Regex::new(r"\s?(/[^/,]+/|(\p{Co})+)").unwrap();
    static WORD_REG: Regex = Regex::new(r"^(?P<name>[a-zA-Z]+)(\s(?P<trans>[a-z]{1,4}\..*))?$").unwrap();
    static PHRASE_REG: Regex = Regex::new(r"^(?P<phrase>(\(?(([a-zA-Z/.]+\s)*[a-zA-Z/.]+)\)?\s?)+)(?P<trans>.*)").unwrap();
    static PART_OF_SPEECH_REG: Regex = Regex::new(r"^[a-z]{1,4}\.").unwrap();
}

fn extract_page(text: &str, entries: &mut Vec<Entry>, head: &mut usize) {
    for line in text.lines() {
        let is_match = WORD_RE.is_match(line);

        // println!("{} {}", is_match, line)
        if is_match {
            println!("{}", line)
        }
    }
}

pub fn extract(args: Args) -> Result<()> {
    let lib_path = if cfg!(debug_assertions) {
        env::current_dir()
    } else {
        env::current_exe()
    }?;

    let core = Pdfium::new(Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(lib_path.as_path()))?);

    let doc = core.load_pdf_from_file(args.path.as_path(), None)?;
    let pages = doc.pages();

    let mut entries: Vec<Entry> = vec![];
    let mut head = 0usize;

    for i in args.start..=args.end {
        let text = pages.get((i as u16).saturating_sub(1))?.text()?.all();
        extract_page(&text, &mut entries, &mut head)
    }

    Ok(())
}

pub struct Extractor {
    core: Pdfium,
    args: Args
}

impl Extractor {
    fn extract_page(text: &str, entries: &mut Vec<Entry>, head: &mut usize) {
        for line in text
            .lines()
            .skip(2)
        {
            // println!("\"{:?}\"", line);
            println!("{}", line);

            let line = &PRONOUNCIATION_REG.replace(line, "").to_string();

            //println!("{} \"{:?}\"", WORD_REG.is_match(line), line);

            if let Some(caps) = WORD_REG.captures(line) {
                let mut entry = Entry {
                    name: caps["name"].to_string(),
                    trans: vec![],
                    kind: Kind::Word
                };

                if let Some(trans) = caps.name("trans") {
                    entry.trans.push(trans.as_str().to_string())
                }

                entries.push(entry);

                *head += 1;
                continue;
            }

            let has_part_of_speech = PART_OF_SPEECH_REG.is_match(line);
            let phrase_caps = PHRASE_REG.captures(line);

            if !has_part_of_speech && phrase_caps.is_some() {
                let caps = phrase_caps.unwrap();
                let entry = Entry {
                    name: caps["phrase"].trim_end().to_string(),
                    trans: vec![caps["trans"].to_string()],
                    kind: Kind::Phrase
                };

                entries.push(entry);

                *head += 1;
                continue;
            }

            assert!(!entries.is_empty() && *head > 0, "No entries found");

            let prev = *head - 1;

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
    }
}
