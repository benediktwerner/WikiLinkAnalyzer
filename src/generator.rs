use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter};

#[derive(Default)]
struct GraphBuilder {
    graph: HashMap<usize, HashSet<usize>>,
    reverse_graph: HashMap<usize, HashSet<usize>>,
    page_ids: HashSet<usize>,
    title_to_id: HashMap<String, usize>,
    redirect_from_id: HashMap<usize, String>,
    redirects: HashMap<String, usize>,
}

pub fn generate() {
    let _progress = crate::progress::msg("Generating link graph");

    let mut builder = GraphBuilder::default();
    builder.load_and_preprocess_pages();
    builder.load_redirects();
    builder.build_graph();

    let writer = BufWriter::new(File::create(crate::GRAPH).unwrap());
    bincode::serialize_into(writer, &builder.graph).unwrap();

    let writer = BufWriter::new(File::create(crate::GRAPH_REVERSE).unwrap());
    bincode::serialize_into(writer, &builder.reverse_graph).unwrap();
}

impl GraphBuilder {
    fn load_and_preprocess_pages(&mut self) {
        let file = File::open(crate::PAGE_TABLE).unwrap();
        let reader = BufReader::new(file);
        let mut writer = BufWriter::new(File::create(crate::PAGES).unwrap());

        for line in reader.lines() {
            let line = line.unwrap();
            let mut iter = line.trim().split('\t');
            let page_id: usize = iter.next().unwrap().parse().unwrap();

            let title = iter.next().unwrap();
            let is_redirect = iter.next().unwrap();

            if is_redirect == "0" {
                let title_nice = cleanup_name(&title[1..title.len() - 1]);
                writeln!(writer, "{}\t{}", page_id, title_nice).unwrap();
                self.page_ids.insert(page_id);
                self.title_to_id.insert(title.to_owned(), page_id);
            } else {
                self.redirect_from_id.insert(page_id, title.to_owned());
            }
        }
    }

    fn load_redirects(&mut self) {
        let file = File::open(crate::REDIRECTS_TABLE).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let mut iter = line.trim().split('\t');
            let page_id: usize = iter.next().unwrap().parse().unwrap();

            let to_title = iter.next().unwrap();
            if !self.title_to_id.contains_key(to_title) {
                continue;
            }
            if let Some(title) = self.redirect_from_id.get(&page_id) {
                if let Some(&to_id) = self.title_to_id.get(title) {
                    self.redirects.insert(title.to_owned(), to_id);
                }
            }
        }
    }

    fn build_graph(&mut self) {
        let file = File::open(crate::LINKS_TABLE).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let mut iter = line.trim().split('\t');
            let from_id: usize = iter.next().unwrap().parse().unwrap();
            let to_title = iter.next().unwrap();

            if !self.page_ids.contains(&from_id) {
                continue;
            }
            if let Some(&to_id) = self.title_to_id.get(to_title) {
                self.add_edge(from_id, to_id);
            } else if let Some(&to_id) = self.redirects.get(to_title) {
                self.add_edge(from_id, to_id);
            }
        }
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.graph.entry(from).or_default().insert(to);
        self.reverse_graph.entry(to).or_default().insert(from);
    }
}

fn cleanup_name(s: &str) -> String {
    let mut result = Vec::new();
    let mut iter = s.chars();
    let mut buf = [0u8; 4];

    while let Some(c) = iter.next() {
        if c == '_' {
            result.push(b' ');
        } else if c != '\\' {
            result.extend(c.encode_utf8(&mut buf).bytes())
        } else {
            match iter.next().unwrap() {
                'b' => result.push(8),
                'f' => result.push(12),
                'n' => result.push(b'\n'),
                'r' => result.push(b'\r'),
                't' => result.push(b'\t'),
                '\\' => result.push(b'\\'),
                '\'' => result.push(b'\''),
                '"' => result.push(b'"'),
                'x' => {
                    let mut s = String::new();
                    s.push(iter.next().unwrap());
                    s.push(iter.next().unwrap());
                    result.push(u8::from_str_radix(&s, 16).unwrap());
                }
                c => panic!("unimplemented escape char: '{}'", c),
            }
        }
    }

    String::from_utf8(result).unwrap()
}
