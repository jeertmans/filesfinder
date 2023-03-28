<p align="center">
  <img src="https://raw.githubusercontent.com/jeertmans/filesfinder/main/static/logo.svg" width="200" height="200"> </img>
</p>

# FilesFinder

> Find files matching patterns while respecting `.gitignore`

[![Crates.io](https://img.shields.io/crates/v/filesfinder)](https://crates.io/crates/filesfinder)

1. [About](#about)
2. [Installation](#installation)
3. [Examples](#examples)
4. [GitHub Action](#github-action)
5. [Contributing](#contributing)

## About

FilesFinder (FF) is a command-line tool that aims to search for files within a given repository.
As such, it respects your `.gitignore` files and exclude the same files from the output.

FF is a **faster** and **simpler-to-use** alternative to other tools such as `find` from [Findutils](https://www.gnu.org/software/findutils/manual/html_mono/find.html).

> **NOTE:** FF is generally faster than `find` (or else), mainly because it uses parallel processing. If you find a scenario in which FF is slower than `find` or any other tool, please report it to me :-)

## Installation

You can install the latest released version with `cargo`:

```bash
> cargo install filesfinder
```

After that, FilesFinder can be used via the `ff` alias.

```text
USAGE:
    ff [OPTIONS] <PATTERN>...
    ff [OPTIONS] <PATTERN> [OPTIONS] <PATTERN> ...

ARGS:
    <PATTERN>...
            A pattern to match against each file.

OPTIONS:
    -g, -G
            Parse pattern as a glob expression.
            [default behavior]

    -r, -R
            Parse pattern as a regular expression.
            Note that expressions are unanchored by default.
            Use '^' or '\A' to denote start, and '$' or '\z' for the end.

    -i, -I
            Matching files will be included in the output.
            [default behavior]

    -e, -E
            Matching files will be excluded from the output.

    -j <JOBS>
            Number of threads to use.
            Setting this to zero will choose the number of threads automatically.
            [default: num_cpus]

        --dir <PATH>
            Files will be searched in the directory specified by the PATH.
            Multiple occurences are allowed.
            [default: '.']

        --max-depth <DEPTH>
            Maximum depth to recurse.
            [default: None]

        --follow-links
            Allow to follow symbolic links.

        --show-hidden
            Allow to show hidden files.

        --no-gitignore
            Ignore .gitignore files.

        --no-ignore
            Ignore .ignore files.

        --no-strip-prefix
            Do not strip './' prefix, same as what GNU find does.

    -h, --help
            Print help information.

    -V, --version
            Print version information.

NOTES:
    -   Capitalized options (.e.g., '-G') apply to all subsequent patterns.
        E.g.: 'ff -g "*.rs" -g "*.md"' is equivalent to 'ff -G "*.rs" "*.md"'.
        You can always unset a flag by overriding it.

    -   Options can be grouped under the same '-'.
        E.g.: 'ff -e -g "*.rs"' is equivalent to 'ff -eg "*.rs"'.

    -   File exclusion is performed after file inclusion.

    -   For performance reasons, prefer to use more general patterns first,
        and more specific ones at the end.
        E.g.: 'ff "*.md" "Cargo.toml"' is (usually) faster but equivalent to 'ff "Cargo.toml" "*.md"'.
```

## Examples

```bash
> ff "*.rs"
# List all files with '.rs' extension

> ff "*.rs" -e "src/**.rs"
# List all files with 'rs' extension except those in the 'src' folder

> ff -r ".*\.md"
# List all files with 'md' extension, using regular expression

> ff -Re ".*\.md" ".*"
# List all files except those with 'md' extension, using regular expression
```

## GitHub Action

A major application to `FF` is to be used within repositories. Therefore, you can also use the FilesFinder GitHub Action withing your projects.

```yml
# Your action in .github/workflows
- name: Checkout repository
  uses: actions/checkout@v3
    # Repository name with owner. For example, actions/checkout
    # Default: ${{ github.repository }}
    repository: ''
- name: Find files matching "*.rs" or "*.md"
  uses: jeertmans/filesfinder@v0.4.4
  id: ff # Any id, to be used later to reference to files output
  with:
    # Only argument, a single string, to be passed as arguments to ff.
    # See `ff --help` for more help.
    # Default: "*"
    args: "*.rs *.md"
- name: Print files
  run: echo "${{ steps.ff.outputs.files }}"
```

## Contributing

Contributions are more than welcome!
