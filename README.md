# Wikipedia link-chain analyzer
This is a program to find the shortest link chains between two wikipedia pages.

It can also be used to find pages that need the longest link-chains to a given page and more.

## Building and Installing
Building and installing requires a [Rust Installation](https://www.rust-lang.org/).

To install:

```
$ git clone https://github.com/benediktwerner/WikiLinkAnalyzer
$ cargo install --path WikiLinkAnalyzer
$ wiki-analyzer --version
```

To build:
```
$ git clone https://github.com/benediktwerner/WikiLinkAnalyzer
$ cd WikiLinkAnalyzer
$ cargo build
$ ./target/debug/wiki-analyzer
```

## Setup
- Goto <https://dumps.wikimedia.org/>
- Choose a wiki e.g. "enwiki" for English Wikipedia or "dewiki" for German Wikipedia
- Download the table dumps for the tables `page`, `pagelinks` and `redirect` as `.sql.gz` archives
- Place them in the `data` directory (relative to the directory you run the command in)
- The analyzer will automatically extract and preprocess these files on the first run

## Debugging tips
You can use the Wikipedia API to find an article from it's ID: <https://de.wikipedia.org/w/api.php?action=query&prop=info&pageids=3034015&inprop=url>.

Table documentations:
- [Page](https://www.mediawiki.org/wiki/Manual:Page_table)
- [Pagelinks](https://www.mediawiki.org/wiki/Manual:Pagelinks_table)
- [Redirect](https://www.mediawiki.org/wiki/Manual:Redirect_table)
