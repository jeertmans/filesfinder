use globset::GlobBuilder;
use regex::bytes::RegexSetBuilder;
use std::io::{self, Write};
use std::path::Path;

#[macro_export]
macro_rules! path_as_bytes {
    ($path: ident) => {
        $path.to_string_lossy().as_bytes()
    };
}

fn print_help() {
    println!(
        "{} {}
{}

{}

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

    -i, -I
            Matching files will be included in the output.
            [default behavior]

    -e, -E
            Matching files will be excluded from the output.

        --dir <PATH>
            Files will be searched in the directory specified by the PATH.
            Multiple values are allowed.
            [default: '.']

        --show-hidden
            Allow to show hidden files.

        --no-gitignore
            Ignore gitignore files.

    -h, --help
            Print help information.

    -V, --version
            Print version information.

NOTES:
    -   Capitalized options (.e.g., '-G') apply to all subsequent patterns.
        E.g.: 'ff -g \"*.rs\" -g \"*.md\"' is equivalent to 'ff -G \"*.rs\" \"*.md\"'.
        You can always unset a flag by overriding it.

    -   Options can be grouped under the same '-'.
        E.g.: 'ff -e -g \"*.rs\"' is equivalent to 'ff -eg \"*.rs\"'.

    -   File exclusion is performed after file inclusion.

    -   For performance reasons, prefer to use more general patterns first,
        and more specific ones at the end.
        E.g.: 'ff \"*.md\" \"Cargo.toml\"' is (usually) faster but equivalent to 'ff \"Cargo.toml\" \"*.md\"'.", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_DESCRIPTION"));
}

#[macro_export]
macro_rules! print_invalid_option {
    (@long $option:ident) => {
        eprintln!("Invalid option --{}. Print help with '--help'.", $option);
    };
    (@short $option:ident) => {
        eprintln!("Invalid option -{}. Print help with '--help'.", $option);
    };
}

fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

#[cfg(unix)]
fn write_path<W: Write>(mut wtr: W, path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    wtr.write_all(path.as_os_str().as_bytes()).unwrap();
    wtr.write_all(b"\n").unwrap();
}

#[cfg(not(unix))]
fn write_path<W: Write>(mut wtr: W, path: &Path) {
    wtr.write_all(path.to_string_lossy().as_bytes()).unwrap();
    wtr.write_all(b"\n").unwrap();
}

#[derive(Clone)]
enum MatcherKind {
    Glob,
    Regex,
}

struct MatcherBuilder<'source> {
    pattern: Option<&'source str>,
    kind: MatcherKind,
}

impl<'source> MatcherBuilder<'source> {
    #[inline]
    fn new(kind: MatcherKind) -> Self {
        Self {
            pattern: None,
            kind,
        }
    }
    #[inline]
    fn set_pattern(mut self, pattern: &'source str) -> Self {
        self.pattern = Some(pattern);
        self
    }

    #[inline]
    fn set_kind(mut self, kind: MatcherKind) -> Self {
        self.kind = kind;
        self
    }

    #[inline]
    fn set_glob(self) -> Self {
        self.set_kind(MatcherKind::Glob)
    }

    #[inline]
    fn set_regex(self) -> Self {
        self.set_kind(MatcherKind::Regex)
    }

    #[inline]
    fn build(self) -> Result<String, Box<dyn std::error::Error>> {
        let pattern = self
            .pattern
            .expect("cannot build matcher if pattern is not set.");

        match self.kind {
            MatcherKind::Glob => Ok(GlobBuilder::new(pattern).build()?.regex().to_string()),
            MatcherKind::Regex => Ok(pattern.to_string()),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut default_kind = MatcherKind::Glob;
    let mut default_include = true;
    let mut include: Vec<String> = vec![];
    let mut exclude: Vec<String> = vec![];
    let mut last_arg_seen = false;
    let mut directories: Vec<String> = vec![];
    let mut follow_links = false;
    let mut ignore_hidden = true;
    let mut use_gitignore = true;
    let mut max_depth: Option<usize> = None;

    while !last_arg_seen {
        let mut matcher = MatcherBuilder::new(default_kind.clone());
        let mut include_next = default_include;

        loop {
            if let Some(arg) = args.next() {
                if let Some(option) = arg.strip_prefix("--") {
                    match option {
                        "max-depth" => {
                            if let Some(depth) = args.next() {
                                max_depth = depth.parse().ok();
                            } else {
                                eprintln!(
                                    "--max-depth option is missing a <DEPTH>. Print help with '--help'."
                                );
                                std::process::exit(1);
                            }
                        }
                        "dir" => {
                            if let Some(path) = args.next() {
                                directories.push(path);
                            } else {
                                eprintln!(
                                    "--dir option is missing a <PATH>. Print help with '--help'."
                                );
                                std::process::exit(1);
                            }
                        }
                        "follow-links" => follow_links = true,
                        "show-hidden" => ignore_hidden = false,
                        "no-gitignore" => use_gitignore = false,
                        "help" => {
                            print_help();
                            std::process::exit(0);
                        }
                        "version" => {
                            print_version();
                            std::process::exit(0);
                        }
                        _ => {
                            print_invalid_option!(@long option);
                            std::process::exit(1);
                        }
                    }
                } else if let Some(options) = arg.strip_prefix('-') {
                    for option in options.chars() {
                        match option {
                            'g' | 'G' => {
                                matcher = matcher.set_glob();
                                if option == 'G' {
                                    default_kind = MatcherKind::Glob;
                                }
                            }
                            'r' | 'R' => {
                                matcher = matcher.set_regex();
                                if option == 'R' {
                                    default_kind = MatcherKind::Regex;
                                }
                            }
                            'i' | 'I' => {
                                include_next = true;
                                if option == 'I' {
                                    default_include = true;
                                }
                            }
                            'e' | 'E' => {
                                include_next = false;
                                if option == 'E' {
                                    default_include = false;
                                }
                            }
                            'h' => {
                                print_help();
                                std::process::exit(0);
                            }
                            'V' => {
                                print_version();
                                std::process::exit(0);
                            }
                            _ => {
                                print_invalid_option!(@short option);
                                std::process::exit(1);
                            }
                        }
                    }
                } else {
                    let matcher = matcher.set_pattern(arg.as_str()).build()?;
                    if include_next {
                        include.push(matcher);
                    } else {
                        exclude.push(matcher);
                    }
                    break;
                }
            } else {
                last_arg_seen = true;
                break;
            }
        }
    }

    if (include.is_empty()) && (exclude.is_empty()) {
        eprintln!(
            "No patterns were speficied, please provide at leat one. Print help with '--help'."
        );
        std::process::exit(1);
    }

    let include = RegexSetBuilder::new(include).build()?;
    let exclude = RegexSetBuilder::new(exclude).build()?;

    let (tx, rx) = crossbeam_channel::unbounded::<ignore::DirEntry>();

    let mut directories = directories.iter().map(|s| s.as_str());

    let mut walk_builder = ignore::WalkBuilder::new(directories.next().unwrap_or("."));

    walk_builder
        .follow_links(follow_links)
        .git_ignore(use_gitignore)
        .hidden(ignore_hidden)
        .max_depth(max_depth);

    for directory in directories {
        walk_builder.add(directory);
    }

    let walker = walk_builder.build_parallel();

    let stdout_thread = std::thread::spawn(move || {
        let mut stdout = io::BufWriter::new(io::stdout());
        for de in rx.iter().filter(|de| {
            let path = de.path();
            let strl = path.to_string_lossy();
            let utf8 = strl.as_bytes();
            path.is_file() && include.is_match(utf8) && !exclude.is_match(utf8)
        }) {
            write_path(&mut stdout, de.path());
        }
    });

    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            tx.send(result.unwrap()).unwrap();
            ignore::WalkState::Continue
        })
    });

    drop(tx);
    stdout_thread.join().unwrap();
    Ok(())
}
