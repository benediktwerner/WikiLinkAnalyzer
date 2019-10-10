use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::BufReader;

pub struct Graph {
    pub graph: HashMap<usize, HashSet<usize>>,
}

impl Graph {
    pub fn load(reverse: bool) -> Graph {
        let graph_file = if reverse {
            println!("Loading reverse graph ..");
                crate::GRAPH_REVERSE
        } else {
            println!("Loading graph ..");
            crate::GRAPH
        };
        let graph_file = File::open(graph_file).unwrap();
        let progress = crate::progress::progress_bar(&graph_file);
        let reader = BufReader::new(progress.wrap_read(graph_file));
        let graph = bincode::deserialize_from(reader).unwrap();
        progress.finish();

        Graph { graph }
    }

    pub fn find_shortest_path(&self, start: usize, end: usize) -> Vec<usize> {
        let mut visited = HashSet::new();
        let mut came_from = HashMap::new();
        let mut todo = VecDeque::new();

        todo.push_back(start);
        visited.insert(start);

        while let Some(curr) = todo.pop_front() {
            if let Some(neigbhors) = self.graph.get(&curr) {
                for &n in neigbhors {
                    if !visited.contains(&n) {
                        todo.push_back(n);
                        visited.insert(n);
                        came_from.insert(n, curr);

                        if n == end {
                            todo.clear();
                            break;
                        }
                    }
                }
            }
        }

        let mut path = vec![end];
        let mut curr = end;

        while let Some(&from) = came_from.get(&curr) {
            path.push(from);
            curr = from;
        }

        path.reverse();
        path
    }

    pub fn find_furthest(&self, start: usize) -> (usize, usize) {
        let mut visited = HashSet::new();
        let mut todo = VecDeque::new();
        let mut max_node = start;
        let mut max_dist = 0;
        let progress = crate::progress::progress_bar(self.graph.len());

        visited.insert(start);
        todo.push_back((start, 0));

        while let Some((curr, dist)) = todo.pop_front() {
            let count = visited.len() - todo.len();
            if count % 10_000 == 0 {
                progress.set_position(count as u64);
            }
            if dist > max_dist {
                max_dist = dist;
                max_node = curr;
            }
            if let Some(neigbhors) = self.graph.get(&curr) {
                for &n in neigbhors {
                    if !visited.contains(&n) {
                        todo.push_back((n, dist + 1));
                        visited.insert(n);
                    }
                }
            }
        }

        progress.finish();
        (max_node, max_dist)
    }

    pub fn estimate_diameter(&self) -> (usize, usize, usize) {
        let mut curr = self.random_node();
        let mut max_start = curr;
        let mut max_end = curr;
        let mut max_dist = 0;
        let mut no_changes_count = 0;

        while no_changes_count < 5 {
            let (next, dist) = self.find_furthest(curr);
            if dist > max_dist {
                max_start = curr;
                max_end = next;
                max_dist = dist;
                no_changes_count = 0;
            } else {
                no_changes_count += 1;
            }
            curr = next;
        }

        (max_start, max_end, max_dist)
    }

    fn random_node(&self) -> usize {
        let max = self.graph.len();
        loop {
            let rnd = rand::thread_rng().gen_range(0, max);
            if self.graph.contains_key(&rnd) {
                return rnd;
            }
        }
    }
}
