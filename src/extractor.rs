use flate2::read::GzDecoder;
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

macro_rules! _table_regex {
    ($head:ident, $($tail:ident),+) => {
        concat!(_table_regex!($head), "),(", _table_regex!($($tail),+))
    };
    (int) => {
        "-?[0-9]+"
    };
    (uint) => {
        "[0-9]+"
    };
    (string) => {
        r"(?:'.*?')|(?:NULL)"
    };
    (float) => {
        r"[0-9]+\.[0-9]+"
    };
}

macro_rules! table_regex {
    ($($rows:ident),+) => {
        concat!("^(", _table_regex!($($rows),+), ")$")
    };
}

const PAGE_REGEX: &str = table_regex!(
    uint, int, string, string, int, int, float, string, string, uint, int, string, string
);
const REDIRECT_REGEX: &str = table_regex!(uint, int, string, string, string);
const PAGELINKS_REGEX: &str = table_regex!(uint, int, string, int);

#[derive(Clone, Copy)]
enum Table {
    Page,
    Pagelinks,
    Redirect,
}

impl Table {
    fn name(self) -> &'static str {
        match self {
            Table::Page => "page",
            Table::Pagelinks => "pagelinks",
            Table::Redirect => "redirect",
        }
    }
    fn target_file(self) -> &'static str {
        match self {
            Table::Page => crate::PAGE_TABLE,
            Table::Pagelinks => crate::LINKS_TABLE,
            Table::Redirect => crate::REDIRECTS_TABLE,
        }
    }
    fn regex(self) -> Regex {
        Regex::new(match self {
            Table::Page => PAGE_REGEX,
            Table::Pagelinks => PAGELINKS_REGEX,
            Table::Redirect => REDIRECT_REGEX,
        })
        .unwrap()
    }
    fn rows(self) -> Vec<usize> {
        match self {
            Table::Page => vec![0, 2, 4],
            Table::Pagelinks => vec![0, 2],
            Table::Redirect => vec![0, 2],
        }
    }
    fn namespace_rows(self) -> Vec<usize> {
        match self {
            Table::Page => vec![1],
            Table::Pagelinks => vec![1, 3],
            Table::Redirect => vec![1],
        }
    }
}

fn extract(table: Table, path: &str) {
    println!("Extracting table '{}' ...", table.name());

    let file = File::open(path).unwrap();
    let progress = crate::progress::progress_bar(file.metadata().unwrap().len());
    let decoder = GzDecoder::new(progress.wrap_read(file));
    let reader = BufReader::new(decoder);

    let out_file = File::create(table.target_file()).unwrap();
    let mut writer = BufWriter::new(out_file);

    let line_start = format!("INSERT INTO `{}` VALUES ", table.name());
    let regex = table.regex();
    let rows = table.rows();
    let namespace_rows = table.namespace_rows();

    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with(&line_start) {
            let mut iter = line.split(" VALUES ");
            let values = iter.nth(1).unwrap();
            let iter = values[1..values.len() - 2].split("),("); // Problem: (362495,0,'Seesterne_(Klasse),(Art),(Gattung)',101)
            for val in iter {
                let val = val.replace("\\xe2\\x80\\x93", "-");
                let captures = regex.captures(&val);
                let captures = match captures {
                    Some(c) => c,
                    None => {
                        println!("Failed to parse: '{}'", val);
                        continue;
                    }
                };
                if namespace_rows.iter().any(|i| &captures[i + 1] != "0") {
                    continue;
                }
                for &i in &rows {
                    if i == 0 {
                        write!(writer, "{}", &captures[i + 1]).unwrap();
                    } else {
                        write!(writer, "\t{}", &captures[i + 1]).unwrap();
                    }
                }
                writeln!(writer).unwrap();
            }
        }
    }
}

pub fn ensure_extracted() -> Result<(), ()> {
    for table in &[Table::Page, Table::Redirect, Table::Pagelinks] {
        if !crate::file_exists(table.target_file()) {
            let name = format!("-{}.sql.gz", table.name());
            let data_dir = std::fs::read_dir("data").unwrap();
            let mut files = data_dir
                .map(|entry| entry.unwrap().file_name().into_string().unwrap())
                .filter(|fname| fname.ends_with(&name));

            if let Some(file) = files.next() {
                if files.next().is_some() {
                    println!("Multiple table dumps for table '{}'.", table.name());
                    println!("Please move or delete the others and try again.");
                    return Err(());
                }
                extract(*table, &file);
            } else {
                println!("Missing database dumps.");
                println!("Please downloade the 3 tables 'page', 'pagelinks' and 'redirects'");
                println!("from https://dumps.wikimedia.org/ as .sql.gz files,");
                println!("place them in the 'data' directory and try again.");
                return Err(());
            }
        }
    }
    Ok(())
}
