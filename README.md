# Wikipedia link-chain analyzer
This is a program to find the shortest link chains between two wikipedia pages.

It can also be used to find pages that need the longest link-chains to a given page.

## Setup
- Install requirements with `pip3 install -r requirements` (may require root)
- Get the `pages`, `redirect` and `pagelinks` tables from [here](https://dumps.wikimedia.org/).
- Extract the data from the `*.sql.gz` files using [WikiUtils](https://github.com/napsternxg/WikiUtils/).
  - *Note:* `WikiUtils` currently doesn't handle pages that end in a comma, like `Kagrra,` properly.
  - `python3 parse_mysqldump.py <pages-dump>.sql.gz page pages.out -c 0124`
  - `python3 parse_mysqldump.py <redirect-dump>.sql.gz redirect redirect.out`
  - `python3 parse_mysqldump.py <pagelinks-dump>.sql.gz pagelinks pagelinks.out`
- Filter out non-article pages and create the link-graph: `./preprocess_files.py`
- Now you can run `./analyzer.py`

## Usage
Run `./analyzer.py -h` for help.

If you aren't just showing the help it will most likely take some time (a minute or more, depending on the wiki)
to load the graph into memory.

## Debugging tips
You can use the Wikipedia API to find an article from it's ID: <https://de.wikipedia.org/w/api.php?action=query&prop=info&pageids=3034015&inprop=url>.

## Files
### WikiUtils output
#### `pagelinks.out`
Database of all intrawiki/internal links.

Columns:
| Name               | Meaning                                                                      |
|--------------------|------------------------------------------------------------------------------|
|`pl_from`           | `page_id` of the page containing the link                                    |
|`pl_from_namespace` | `page_namespace` of the page containing the link. 0 means it's an article.   |
|`pl_namespace`      | `page_namespace` of the target page. 0 means it's an article                 |
|`pl_title`          | Title of the target page                                                     |

Table documentation: <https://www.mediawiki.org/wiki/Manual:Pagelinks_table>

#### `redirects.out`
Database of all redirect pages.

Columns:
| Name               | Meaning                                                          |
|--------------------|------------------------------------------------------------------|
|`rd_from`           | `page_id` of the page redirect page                              |
|`rd_namespace`      | `page_namespace` of the target page. 0 means it's an article.    |
|`rd_title`          | Title of the target page                                         |

Table documentation: <https://www.mediawiki.org/wiki/Manual:Redirect_table>

#### `pages.out`
Database of all wiki pages.

Columns:
| Name               | Meaning                                          |
|--------------------|--------------------------------------------------|
|`page_id`           | ID of the page                                   |
|`page_namespace`    | Namespace key of the page. 0=article             |
|`page_title`        | Title of the page                                |

Table documentation: <https://www.mediawiki.org/wiki/Manual:Page_table>

### Prerpoccessed files
#### `pages_preprocessed.out`
Preprocessed `pages.out` file containing only article pages (with namespace 0) and only
the page id and name. Also doesn't contain redirecting pages.

Built from `preproccess_files.py`.

#### `graph.pickle`
The graph representing links between pages. This is a pickle of a [blist](https://pypi.org/project/blist/)
where `list[ID]` ist a `set` containing the `IDs` of all the pages to which the page with `ID` links.

Built from `preproccess_files.py`.

