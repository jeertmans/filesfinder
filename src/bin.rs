const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!(
        "{NAME} {VERSION}
{AUTHORS}

{DESCRIPTION}

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
    -   Capitalized options (.e.g. '-G') apply to all subsequent patterns.
        E.g.: 'ff -g \"*.rs\" -g \"*.md\"' is equivalent to 'ff -G \"*.rs\" \"*.md\"'.
        You can always unset a flag by overriding it.

    -   Options can be grouped under the same '-'.
        E.g.: 'ff -e -g \"*.rs\"' is equivalent to 'ff -eg \"*.rs\"'.

    -   File exclusion is performed after file inclusion.

    -   For performance reasons, prefer to use more general patterns first,
        and more specific ones at the end.
        E.g.: 'ff \"*.md\" \"README.md\"' is faster but equivalent to 'ff \"README.md\" \"*.md\"'."
    );
}

fn print_version() {
    println!("{NAME} {VERSION}");
}

fn print_invalid_option(option: &str) {
    eprintln!("Invalid option {}. Print help with '--help'.", option);
}

fn print_invalid_long_option(option: &str) {
    print_invalid_option(format!("--{}", option).as_str())
}
fn print_invalid_short_option(option: char) {
    print_invalid_option(format!("-{}", option).as_str())
}

#[derive(Clone, Copy)]
enum MatcherKind {
    Glob,
    Regex,
}

impl Default for MatcherKind {
    fn default() -> Self {
        MatcherKind::Glob
    }
}

#[derive(Clone)]
enum Matcher {
    Glob(globset::GlobMatcher),
    Regex(regex::Regex),
}

impl Matcher {
    #[inline]
    fn is_match(&self, string: &str) -> bool {
        match self {
            Matcher::Glob(glob) => glob.is_match(string),
            Matcher::Regex(regex) => regex.is_match(string),
        }
    }
}

#[derive(Default)]
struct MatcherBuilder<'source> {
    pattern: Option<&'source str>,
    kind: MatcherKind,
}

impl<'source> MatcherBuilder<'source> {
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

    fn build(self) -> Result<Matcher, Box<dyn std::error::Error>> {
        let pattern = self
            .pattern
            .expect("cannot build matcher if pattern is not set.");

        match self.kind {
            MatcherKind::Glob => {
                let glob = globset::GlobBuilder::new(pattern)
                    .build()?
                    .compile_matcher();
                Ok(Matcher::Glob(glob))
            }
            MatcherKind::Regex => {
                let regex = regex::Regex::new(pattern)?;
                Ok(Matcher::Regex(regex))
            }
        }
    }
}

#[derive(Clone)]
struct MatcherSet {
    matchers: Vec<Matcher>,
}

impl MatcherSet {
    fn new(matchers: Vec<Matcher>) -> MatcherSet {
        Self { matchers }
    }

    #[inline]
    fn is_match(&self, string: &str) -> bool {
        self.matchers.iter().any(|m| m.is_match(string))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut default_kind = MatcherKind::Glob;
    let mut default_include = true;
    let mut include: Vec<Matcher> = vec![];
    let mut exclude: Vec<Matcher> = vec![];
    let mut last_arg_seen = false;
    let mut directory = ".".to_string();
    let mut ignore_hidden = true;
    let mut use_gitignore = true;

    while !last_arg_seen {
        let mut matcher = MatcherBuilder::default().set_kind(default_kind);
        let mut include_next = default_include;

        loop {
            if let Some(arg) = args.next() {
                if let Some(option) = arg.strip_prefix("--") {
                    match option {
                        "dir" => {
                            if let Some(path) = args.next() {
                                directory = path;
                            } else {
                                eprintln!(
                                    "--dir option is missing a <PATH>. Print help with '--help'."
                                );
                                std::process::exit(1);
                            }
                        }
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
                            print_invalid_long_option(option);
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
                                print_invalid_short_option(option);
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

    let include = MatcherSet::new(include);
    let exclude = MatcherSet::new(exclude);

    let (tx, rx) = crossbeam_channel::unbounded::<std::path::PathBuf>();

    let walker = ignore::WalkBuilder::new(directory.as_str())
        .hidden(ignore_hidden)
        .git_ignore(use_gitignore)
        .build_parallel();

    let stdout_thread = std::thread::spawn(move || {
        for path in rx {
            println!("{:?}", path.as_os_str());
        }
    });

    walker.run(|| {
        let tx = tx.clone();
        let include = include.clone();
        let exclude = exclude.clone();

        Box::new(move |result| {
            if let Ok(de) = result {
                let path = de.into_path();

                if path.is_file() {
                    if let Some(filename) = path.to_str() {
                        if include.is_match(filename) && !exclude.is_match(filename) {
                            tx.send(path).unwrap();
                        }
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });

    drop(tx);
    stdout_thread.join().unwrap();
    Ok(())
}
