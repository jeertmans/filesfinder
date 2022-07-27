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
    ff [OPTIONS] <PATTERNS>
    ff [OPTIONS] <PATTERN> [OPTIONS] <PATTERN> ...

OPTIONS:
    -g                  Sets next pattern to be parsed as a glob
    -G                  Sets next patterns to be parsed as a glob [default]
    -r                  Sets next pattern to be parsed as a regex
    -R                  Sets next patterns to be parsed as a regex
    -h, --help          Print help information
    -V, --version       Print version information"
    );
}

fn print_version() {
    println!("{NAME} {VERSION}");
}

fn print_invalid_option(option: &str) {
    eprintln!("Invalid option {}. See valid options below:\n", option);
    print_help();
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
        assert!(
            self.pattern.is_some(),
            "cannot build matcher if pattern is not set."
        );

        let pattern = self.pattern.unwrap();

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

fn main() -> Result<(), Box< dyn std::error::Error>>{
    let mut args = std::env::args().skip(1);
    let mut default_kind = MatcherKind::Glob;
    let mut matchers: Vec<Matcher> = vec![];
    let mut last_arg_seen = false;

    while !last_arg_seen {
        let mut matcher = MatcherBuilder::default().set_kind(default_kind);

        loop {
            if let Some(arg) = args.next() {
                if let Some(option) = arg.strip_prefix("--") {
                    match option {
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
                            'r' => matcher = matcher.set_regex(),
                            'g' => matcher = matcher.set_glob(),
                            'R' => {
                                matcher = matcher.set_regex();
                                default_kind = MatcherKind::Regex;
                            }
                            'G' => {
                                matcher = matcher.set_glob();
                                default_kind = MatcherKind::Glob;
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
                    matchers.push(matcher.set_pattern(arg.as_str()).build()?);
                    break;
                }
            } else {
                last_arg_seen = true;
                break;
            }
        }
    }

    if matchers.is_empty() {
        eprintln!("No patterns were speficied, please provide at leat one. See usage below:\n");
        print_help();
        std::process::exit(1);
    }

    let m = MatcherSet::new(matchers);

    let paths = ignore::WalkBuilder::new(".")
        .build()
        .filter_map(|de| de.ok().map(|de| de.into_path()));
    paths
        .filter(|p| p.is_file() && m.is_match(p.to_str().unwrap()))
        .for_each(|s| println!("{:?}", s));

    Ok(())
}
