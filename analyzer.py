#!/usr/bin/env python3

import ast
import datetime
import os
import pickle
import random
import sys
import time
from collections import defaultdict, deque

from blist import blist

DATA_DIRECTORY = "data"
PAGES_FILE = os.path.join(DATA_DIRECTORY, "pages_preprocessed.txt")
GRAPH_FILE = os.path.join(DATA_DIRECTORY, "graph.pickle")
GRAPH_REVERSED_FILE = os.path.join(DATA_DIRECTORY, "graph_reversed.pickle")


def user_input(text):
    result = input(text)

    if result in ("q", "quit", "exit"):
        exit(0)

    return result


def format_timedelta(start, end):
    return str(datetime.timedelta(seconds=int(end - start)))


class Analyzer:
    def __init__(self):
        self.graph = blist()
        self.id_to_title = {}
        self.title_to_id = {}

    def main(self, args):
        if len(args) > 4 or (len(args) == 2 and args[1] == "-h"):
            print("Usage:", args[0], "START_PAGE            # Find furthets page")
            print("Usage:", args[0], "START_PAGE END_PAGE   # Find shorest path")
            print("Usage:", args[0], "                      # Interactive mode")
            print("       Use '-r' to load the reversed graph in all modes")
            exit(1)

        self.load_pages()

        if "-r" in args[1:]:
            self.load_graph(reverse=True)
            args.remove("-r")
        else:
            self.load_graph()

        if len(args) == 2:
            self.find_furthest_page(args[1])
        elif len(args) == 3:
            self.find_shortest_path(args[1], args[2])
        else:
            self.loop()

    def loop(self):
        while True:
            print()
            print("Options:")
            print("0 - Find shortest path (default)")
            print("1 - Find furthest page")
            print("2 - Approximate graph diameter")

            action = user_input("> ")

            if not action or action == "0":
                start = user_input("Start page: ")
                end = user_input("End page: ")
                self.find_shortest_path(start, end)
            elif action == "1":
                start = user_input("Start page: ")
                self.find_furthest_page(start)
            elif action == "2":
                self.pseudo_diameter()
            else:
                print("Invalid option:", action)

    def load_pages(self):
        start_time = time.perf_counter()
        with open(PAGES_FILE) as fp:
            for line in fp:
                page_id, title = line.strip().split("\t")
                page_id = int(page_id)

                self.id_to_title[page_id] = title
                self.title_to_id[title] = page_id

        end_time = time.perf_counter()
        delta = format_timedelta(start_time, end_time)
        print(f"Loaded {len(self.id_to_title):,} pages in {delta}")

    def load_graph(self, reverse=False):
        start_time = time.perf_counter()

        graph_file = GRAPH_REVERSED_FILE if reverse else GRAPH_FILE
        with open(graph_file, "rb") as fp:
            self.graph = pickle.load(fp)

        end_time = time.perf_counter()
        delta = format_timedelta(start_time, end_time)
        print(f"Loaded links in {delta}")

    def get_random_id(self):
        max_index = len(self.graph)

        while True:
            rnd = random.randrange(max_index + 1)
            if self.graph[rnd] is not None:
                return rnd

    def pseudo_diameter(self):
        curr = self.get_random_id()
        no_change_count = 0
        max_start, max_end = curr, None
        max_dist = 0

        while no_change_count < 5:
            print(max_start, max_end, max_dist, no_change_count)
            nxt, dist = self.__find_furthest_path(curr)
            if dist > max_dist:
                max_start = curr
                max_end = nxt
                max_dist = dist
                no_change_count = 0
            else:
                no_change_count += 1
            
            curr = nxt

        print(f"Furthest distance: {max_dist}")
        print(f"From '{self.id_to_title[max_start]}' to '{self.id_to_title[max_end]}'")

    def __find_furthest_path(self, start_id):
        visited = set([start_id])
        todo = deque([(start_id, 0)])
        max_node, max_dist = start_id, 0

        while todo:
            node, dist = todo.popleft()
            if dist > max_dist:
                max_dist = dist
                max_node = node

            if self.graph[node] is None:
                # print("Found trapped node:", node, self.id_to_title[node])
                continue

            for neighbor in self.graph[node]:
                if neighbor not in visited:
                    todo.append((neighbor, dist + 1))
                    visited.add(neighbor)

        return max_node, max_dist

    def find_furthest_page(self, start):
        start_id = self.title_to_id.get(start)

        if start_id is None:
            print("Unknown start page:", start)
            return

        max_node, max_dist = self.__find_furthest_path(start_id)
        print(f"Furthest page '{self.id_to_title[max_node]}' with distance {max_dist}")

    def find_shortest_path(self, start, end):
        start = self.title_to_id.get(start)
        end = self.title_to_id.get(end)
        visited = set([start])
        came_from = {start: None}
        todo = deque([start])

        if start is None:
            print("Unknown start page")
            return
        if end is None:
            print("Unknown end page")
            return

        while todo:
            node = todo.popleft()

            if self.graph[node] is None:
                continue

            for neighbor in self.graph[node]:
                if neighbor not in visited:
                    todo.append(neighbor)
                    visited.add(neighbor)
                    came_from[neighbor] = node

                    if neighbor == end:
                        todo.clear()
                        break

        print("Checked", len(visited), "pages")

        if end not in came_from:
            print("No path found")
            return

        print("Shortest path:")

        path = []
        while end is not None:
            path.append(self.id_to_title[end])
            end = came_from[end]
        print(*reversed(path), sep="\n")


if __name__ == "__main__":
    Analyzer().main(sys.argv)
