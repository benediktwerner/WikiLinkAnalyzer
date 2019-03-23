#!/usr/bin/env python3

import ast
import os
import pickle
import sys

from blist import blist

DATA_DIRECTORY = "data"

PAGES_IN_FILE = os.path.join(DATA_DIRECTORY, "pages.out")
REDIRECTS_IN_FILE = os.path.join(DATA_DIRECTORY, "redirects.out")
PAGELINKS_IN_FILE = os.path.join(DATA_DIRECTORY, "pagelinks.out")

PAGES_OUT_FILE = os.path.join(DATA_DIRECTORY, "pages_preprocessed.txt")
GRAPH_OUT_FILE = os.path.join(DATA_DIRECTORY, "graph.pickle")
GRAPH_REVERSED_OUT_FILE = os.path.join(DATA_DIRECTORY, "graph_reversed.pickle")


def decode_title(title):
    try:
        return ast.literal_eval("b" + title).decode().replace("_", " ")
    except Exception as e:
        print("Error when decoding title:")
        print(e)
        print("Title:", title)
        exit(1)


class Preprocessor:
    def load_redirects(self):
        self.redirects = {}

        with open(REDIRECTS_IN_FILE) as fp:
            for line in fp:
                page_id, to_namespace, to_title = line.strip().split("\t")
                page_id = int(page_id)

                if to_namespace == "0":
                    if to_title not in self.page_to_id:
                        continue
                    elif page_id not in self.redirect_from_id:
                        continue

                    title = self.redirect_from_id[page_id]
                    self.redirects[title] = self.page_to_id[to_title]

        print(f"Processed {len(self.redirects):,} redirects")

    def load_and_preprocess_pages(self):
        self.page_to_id = {}
        self.page_ids = set()
        self.redirect_from_id = {}
        self.max_id = 0

        with open(PAGES_IN_FILE) as fp, open(PAGES_OUT_FILE, "w") as out_fp:
            for line in fp:
                page_id, namespace, title, is_redirect = line.strip().split("\t")
                page_id = int(page_id)

                if namespace != "0":
                    continue

                if is_redirect == "0":
                    print(f"{page_id}\t{decode_title(title)}", file=out_fp)
                    self.page_to_id[title] = page_id
                    self.page_ids.add(page_id)

                    if page_id > self.max_id:
                        self.max_id = page_id
                else:
                    self.redirect_from_id[page_id] = title

        print(f"Filtered and indexed {len(self.page_to_id):,} pages")
        print(f"Highest ID {self.max_id:,}")

    def add_edge(self, from_id, to_id, reverse=False):
        if reverse:
            from_id, to_id = to_id, from_id

        targets = self.graph[from_id]
        if targets is None:
            self.graph[from_id] = set([to_id])
        else:
            targets.add(to_id)

    def create_graph(self, reverse=True):
        self.graph = blist([None])
        self.graph *= self.max_id + 1
        count = 0
        not_found = 0

        with open(PAGELINKS_IN_FILE) as fp:
            for line in fp:
                from_id, from_ns, to_title, to_ns = line.strip().split("\t")
                from_id = int(from_id)

                if from_ns == "0" == to_ns:
                    count += 1

                    if from_id not in self.page_ids:
                        continue

                    if to_title in self.page_to_id:
                        self.add_edge(from_id, self.page_to_id[to_title], reverse)
                    elif to_title in self.redirects:
                        self.add_edge(from_id, self.redirects[to_title], reverse)
                    else:
                        not_found += 1

        print(f"Parsed {count:,} links")
        print(f"Found {not_found:,} broken links")

        out_file = GRAPH_REVERSED_OUT_FILE if reverse else GRAPH_OUT_FILE
        with open(out_file, "wb") as fp:
            pickle.dump(self.graph, fp, pickle.HIGHEST_PROTOCOL)

        print(f"Graph written to '{out_file}'")

    def main(self, args):
        if len(args) > 2 or (len(args) == 2 and args[1] != "-r"):
            print("Usage:", args[0], "      # Filter pages and build graph")
            print("Usage:", args[0], "-r    # same, but reverse graph direction")
        self.load_and_preprocess_pages()
        self.load_redirects()

        self.create_graph(reverse=len(args) == 2)


if __name__ == "__main__":
    Preprocessor().main(sys.argv)
