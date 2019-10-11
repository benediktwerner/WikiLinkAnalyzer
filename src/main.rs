use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

mod extractor;
mod generator;
mod graph;
mod progress;

use graph::Graph;

const PAGE_TABLE: &str = "data/page.csv";
const REDIRECTS_TABLE: &str = "data/redirect.csv";
const LINKS_TABLE: &str = "data/pagelinks.csv";

pub const PAGES: &str = "data/pages.csv";
pub const GRAPH: &str = "data/graph.bin";
pub const GRAPH_REVERSE: &str = "data/graph_reverse.bin";

struct Main<'a> {
    title_to_id: HashMap<String, usize>,
    id_to_title: HashMap<usize, String>,
    graph: Option<Graph>,
    reverse_graph: Option<Graph>,
    stdin_lock: std::io::StdinLock<'a>,
}

impl Main<'_> {
    fn new() -> Self {
        Main {
            title_to_id: HashMap::new(),
            id_to_title: HashMap::new(),
            graph: None,
            reverse_graph: None,
            stdin_lock: Box::leak(Box::new(std::io::stdin())).lock(),
        }
    }

    fn read_line(&mut self) -> String {
        std::io::stdout().flush().unwrap();
        let mut result = String::new();
        self.stdin_lock.read_line(&mut result).unwrap();
        result
    }

    fn read_page(&mut self, prompt: &str) -> usize {
        print!("{} ", prompt);
        loop {
            let title = self.read_line();
            let title = title.trim();
            if let Some(&id) = self.title_to_id.get(title) {
                return id;
            }
            print!("Invalid page. Try again: ");
        }
    }

    fn load(&mut self) -> Result<(), ()> {
        if !file_exists(PAGES) || !file_exists(GRAPH) || !file_exists(GRAPH_REVERSE) {
            extractor::ensure_extracted()?;
            generator::generate();
        }

        let _progress = progress::msg("Loading pages");

        let reader = BufReader::new(File::open(PAGES).unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            let mut iter = line.split('\t');
            let id: usize = iter.next().unwrap().parse().unwrap();
            let title = iter.next().unwrap();
            self.id_to_title.insert(id, title.to_owned());
            self.title_to_id.insert(title.to_owned(), id);
        }

        Ok(())
    }

    fn ensure_graph(&mut self) {
        if self.graph.is_none() {
            self.graph = Some(Graph::load(false));
        }
    }

    fn ensure_reverse_graph(&mut self) {
        if self.reverse_graph.is_none() {
            self.reverse_graph = Some(Graph::load(true));
        }
    }

    fn graph(&self) -> &Graph {
        self.graph.as_ref().unwrap()
    }

    fn reverse_graph(&self) -> &Graph {
        self.reverse_graph.as_ref().unwrap()
    }

    fn run(&mut self) {
        if self.load().is_err() {
            return;
        }

        loop {
            print!("What do you want to do? ");
            match self.read_line().trim() {
                "help" => {
                    println!("links     - List all the links on a page");
                    println!("path      - Find the shortest path from one page to another");
                    println!("furthest  - Find the page furthest away from a starting point");
                    println!("max       - Find the maximal number of steps needed to get to a page from anywhere");
                    println!("diameter  - Approximate the diameter of the link graph (i.e. how far the furthest two pages are apart)");
                    println!("exit");
                }
                "links" => {
                    self.ensure_graph();
                    let page = self.read_page("Page:");
                    let links = &self.graph().graph[&page];
                    println!("{} links:", links.len());
                    for link in links {
                        println!("{}", self.id_to_title[link]);
                    }
                }
                "path" => {
                    self.ensure_graph();
                    let start = self.read_page("Start page:");
                    let end = self.read_page("Target page:");
                    println!();

                    let path = self.graph().find_shortest_path(start, end);

                    if path.is_empty() {
                        println!("No path found.");
                    } else {
                        println!("Reachable in {} steps:", path.len() - 1);
                        for n in path {
                            println!("{}", self.id_to_title[&n]);
                        }
                    }
                }
                "furthest" => {
                    self.ensure_graph();
                    let start = self.read_page("Start page:");
                    println!();

                    let (end, dist) = self.graph().find_furthest(start);
                    let title = &self.id_to_title[&end];
                    println!("The furthest page is '{}' at {} steps.", title, dist);
                }
                "max" => {
                    self.ensure_reverse_graph();
                    let start = self.read_page("Target page:");
                    println!();

                    let (end, dist) = self.reverse_graph().find_furthest(start);
                    let title = &self.id_to_title[&end];
                    println!(
                        "The maximal number of steps needed is {} from page '{}'.",
                        dist, title
                    );
                }
                "diameter" => {
                    self.ensure_graph();
                    let (start, end, dist) = self.graph().estimate_diameter();
                    let title_start = &self.id_to_title[&start];
                    let title_end = &self.id_to_title[&end];
                    println!("The estimated diameter is {}.", dist);
                    println!("when going from '{}'", title_start);
                    println!("to '{}'.", title_end);
                }
                "exit" | "quit" => return,
                _ => println!("Invalid command. Try 'help' for help."),
            }
            println!();
        }
    }
}

fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn main() {
    Main::new().run();
}
