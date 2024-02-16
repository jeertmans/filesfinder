use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{ArgAction, ArgMatches, Args, FromArgMatches, Parser};
use globset::GlobBuilder;
use regex::bytes::RegexSetBuilder;

#[macro_export]
macro_rules! path_as_bytes {
    ($path: ident) => {
        $path.to_string_lossy().as_bytes()
    };
}

#[derive(Debug, Parser)]
#[command(
    author,
    name = "ff",
    version,
    about,
    after_help = "For detailed help usage, see: `ff --help`.",
    after_long_help = "NOTES:
    -   Capitalized options (.e.g., '-G') apply to all subsequent patterns.
        E.g.: 'ff -g \"*.rs\" -g \"*.md\"' is equivalent to 'ff -G \"*.rs\" \"*.md\"'.
        You can always unset a flag by overriding it.

    -   Options can be grouped under the same '-'.
        E.g.: 'ff -e -g \"*.rs\"' is equivalent to 'ff -eg \"*.rs\"'.

    -   File exclusion is performed after file inclusion.

    -   For performance reasons, prefer to use more general patterns first,
        and more specific ones at the end.
        E.g.: 'ff \"*.md\" \"Cargo.toml\"' is (usually) faster but equivalent to 'ff \"Cargo.toml\" \"*.md\"'.",
    override_usage = "ff [OPTIONS] <PATTERN>...\n       \
     ff [OPTIONS] <PATTERN> [OPTIONS] <PATTERN> ..."
)]
struct Cli {
    #[clap(flatten, next_help_heading = "Walk options")]
    pub walk_options: WalkOptions,
    #[clap(flatten, next_help_heading = "Match options")]
    pub match_options: MatchOptions,
}

#[derive(Args, Debug)]
struct WalkOptions {
    /// Number of threads to use.
    ///
    /// Setting this to zero will choose the number of threads automatically.
    #[arg(short = 'j', value_name = "JOBS")]
    threads: Option<usize>,
    /// Directory to search for files.
    #[arg(short = 'd', long = "dir", default_value = ".", action = ArgAction::Append)]
    directories: Vec<String>,
    /// Maximum depth to recurse into directories.
    #[arg(long, value_name = "DEPTH")]
    max_depth: Option<usize>,
    /// Allow to follow symbolic links.
    #[arg(long)]
    follow_links: bool,
    /// Search hidden files and directories.
    ///
    /// By default, hidden files and directories are skipped.
    #[arg(short = '.', long, alias = "show-hidden")]
    hidden: bool,
    /// Ignore .gitignore files.
    #[arg(long)]
    no_gitignore: bool,
    /// Ignore .ignore files.
    #[arg(long)]
    no_ignore: bool,
}

impl WalkOptions {
    pub fn into_builder(self) -> ignore::WalkBuilder {
        let mut walk_builder = ignore::WalkBuilder::new(&self.directories[0]);
        walk_builder
            .follow_links(self.follow_links)
            .git_ignore(!self.no_gitignore)
            .hidden(self.hidden)
            .ignore(!self.no_ignore)
            .max_depth(self.max_depth)
            .threads(self.threads.unwrap_or(num_cpus::get()));

        for directory in self.directories[1..].iter() {
            walk_builder.add(directory);
        }

        walk_builder
    }
}

#[derive(Args, Debug)]
struct MatchOptions {
    #[clap(flatten)]
    patterns: Patterns,
    /// Do not strip './' prefix, same as what GNU find does.
    #[arg(long)]
    no_strip_prefix: bool,
}

#[derive(Debug, Default)]
struct Patterns {
    include: Vec<String>,
    exclude: Vec<String>,
}

#[derive(Debug)]
enum Flag {
    Glob(bool),
    Regex(bool),
    Include(bool),
    Exclude(bool),
}

impl clap::FromArgMatches for Patterns {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let patterns = matches
            .get_many::<&str>("patterns")
            .map(|v| v.collect::<Vec<_>>())
            .unwrap_or_else(Vec::new);

        let pattern_indices = matches.indices_of("patterns").unwrap();
        let mut flags = BTreeMap::new();

        matches.indices_of("glob").map(|indices| {
            indices.for_each(|index| {
                flags.insert(index, Flag::Glob(true));
            })
        });

        matches.indices_of("regex").map(|indices| {
            indices.for_each(|index| {
                flags.insert(index, Flag::Regex(true));
            })
        });

        matches.indices_of("include").map(|indices| {
            indices.for_each(|index| {
                flags.insert(index, Flag::Include(true));
            })
        });

        matches.indices_of("exclude").map(|indices| {
            indices.for_each(|index| {
                flags.insert(index, Flag::Exclude(true));
            })
        });

        let mut glob_is_default = true;
        let mut include_is_default = true;

        let mut flags: Vec<(usize, Flag)> = flags.into_iter().collect();
        let mut include_patterns: Vec<Cow<'str>> = Vec::new();
        let mut exclude_patterns: Vec<Cow<'str>> = Vec::new();

        for (index, pattern_str) in pattern_indices.into_iter().zip(patterns) {
            let mut glob = glob_is_default;
            let mut include = include_is_default;

            let i = flags.partition_point(|(i, _)| *i < index);
            for (_, flag) in flags.drain(0..i) {
                match flag {
                    Flag::Glob(_) => glob = true,
                    Flag::Regex(_) => glob = false,
                    Flag::Include(_) => include = true,
                    Flag::Exclude(_) => include = false,
                }
            }

            if glob {
                pattern = GlobBuilder::new(pattern_str).build().unwrap().regex().to_string();
            }

            if include {
                include_patterns.push(pattern);
            } else {
                exclude_patterns.push(pattern);
            }

            let include = RegexSetBuilder::new(include_patterns).build().unwrap();
            let exclude = RegexSetBuilder::new(exclude_patterns).build().unwrap();
            
        }
        //println!("{:#?}", flags);

        Ok(Self::default())
    }
    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        <Self as FromArgMatches>::from_arg_matches(matches).map(|other| {
            self.include.extend_from_slice(other.include.as_ref());
            self.exclude.extend_from_slice(other.exclude.as_ref());
        })
    }
}

impl clap::Args for Patterns {
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("Patterns"))
    }
    fn augment_args(cmd: clap::Command) -> clap::Command {
        {
            cmd.group(clap::ArgGroup::new("Patterns").multiple(true).args({
                let members: [clap::Id; 5usize] = [
                    clap::Id::from("patterns"),
                    clap::Id::from("glob"),
                    clap::Id::from("regex"),
                    clap::Id::from("include"),
                    clap::Id::from("exclude"),
                ];
                members
            }))
            .arg(
                clap::Arg::new("patterns")
                    .value_name("PATTERN")
                    .num_args(1..)
                    .value_parser({ clap::builder::NonEmptyStringValueParser::new() })
                    .action(clap::ArgAction::Append)
                    .help("A pattern to match against each file")
                    .long_help(None)
                    .required(true)
                    .help_heading(None),
            )
            .arg(
                clap::Arg::new("glob")
                    .short('g')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .help("Parse pattern as a glob expression")
                    .long_help(None),
            )
            .arg(
                clap::Arg::new("regex")
                    .short('r')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .help("Parse pattern as a regular expression")
                    .long_help(
                        "Parse pattern as a regular expression.\n\n\
                               Note that expressions are unanchored by default. \
                               Use '^' or '\\\\A' to denote start, and '$' or \
                               '\\\\z' for the end.",
                    ),
            )
            .arg(
                clap::Arg::new("include")
                    .short('i')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .help("Matching files will be included in the output")
                    .long_help(None),
            )
            .arg(
                clap::Arg::new("exclude")
                    .short('e')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .help("Matching files will be excluded from the output")
                    .long_help(None),
            )
        }
    }
    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        {
            <Self as clap::Args>::augment_args(cmd)
        }
    }
}

#[derive(Clone, Debug)]
enum Pattern {
    Include(String),
    Exclude(String),
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
            Note that expressions are unanchored by default.
            Use '^' or '\\A' to denote start, and '$' or '\\z' for the end.

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
            Print version information.",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_DESCRIPTION")
    );
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::CommandFactory;
    let mut arg_matches = Cli::command().get_matches();

    let walk_option = <WalkOptions as FromArgMatches>::from_arg_matches_mut(&mut arg_matches);

    println!("There");

    for id in arg_matches.ids() {
        println!("{:#?}", id);
    }

    let args = Cli::parse();

    let include: Vec<String> = vec![];
    let exclude: Vec<String> = vec![];

    let include = RegexSetBuilder::new(include).build()?;
    let exclude = RegexSetBuilder::new(exclude).build()?;

    let (tx, rx) = crossbeam_channel::unbounded::<PathBuf>();

    let walker = args.walk_options.into_builder().build_parallel();

    let mut stdout = io::BufWriter::new(io::stdout());

    let stdout_thread = std::thread::spawn(move || {
        for path_buf in rx {
            write_path(&mut stdout, path_buf.as_path());
        }
    });

    let strip_prefix = true;

    walker.run(|| {
        let tx = tx.clone();
        let include = &include;
        let exclude = &exclude;

        Box::new(move |result| {
            let de = match result {
                Ok(de) => de,
                Err(_) => return ignore::WalkState::Continue,
            };

            let mut path = de.path();

            if strip_prefix {
                if let Ok(p) = path.strip_prefix("./") {
                    path = p;
                }
            }

            let strl = path.to_string_lossy();
            let utf8 = strl.as_bytes();
            if path.is_file() && include.is_match(utf8) && !exclude.is_match(utf8) {
                match tx.send(path.to_path_buf()) {
                    Ok(_) => ignore::WalkState::Continue,
                    Err(_) => ignore::WalkState::Quit,
                }
            } else {
                ignore::WalkState::Continue
            }
        })
    });

    drop(tx);
    stdout_thread.join().unwrap();
    Ok(())
}
