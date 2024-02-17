use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{ArgAction, ArgMatches, Args, FromArgMatches, Parser};
use globset::Glob;
use regex::bytes::{RegexSet, RegexSetBuilder};

#[macro_export]
macro_rules! path_as_bytes {
    ($path: ident) => {
        $path.to_string_lossy().as_bytes()
    };
}

#[derive(Debug, Parser)]
#[command(
    author,
    name = "filesfinder",
    version,
    about,
    after_help = "For detailed help usage, see: `ff --help`.",
    after_long_help = color_print::cstr!("\
    <bold><underline>Notes:</underline></bold>
    -   Capitalized options (.e.g., '-G') apply to all subsequent patterns.
        E.g.: 'ff -g \"*.rs\" -g \"*.md\"' is equivalent to 'ff -G \"*.rs\" \"*.md\"'.
        You can always unset a flag by overriding it.

    -   Options can be grouped under the same '-'.
        E.g.: 'ff -e -g \"*.rs\"' is equivalent to 'ff -eg \"*.rs\"'.

    -   File exclusion is performed after file inclusion.

    -   For performance reasons, prefer to use more general patterns first,
        and more specific ones at the end.
        E.g.: 'ff \"*.md\" \"Cargo.toml\"' is (usually) faster but equivalent to 'ff \"Cargo.toml\" \"*.md\"'."),
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
    #[arg(short = 'd', long = "dir", value_name = "DIRECTORY", default_value = ".", action = ArgAction::Append, value_hint = clap::ValueHint::DirPath)]
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
    show_hidden: bool,
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
            .hidden(!self.show_hidden)
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

#[derive(Debug)]
struct Patterns {
    include: RegexSet,
    exclude: RegexSet,
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
            .get_many::<String>("patterns")
            .map(|v| v.collect::<Vec<_>>())
            .unwrap_or_default();

        let pattern_indices = matches.indices_of("patterns").unwrap();
        let mut flags = BTreeMap::new();

        if let Some(indices) = matches.indices_of("glob") {
            indices.for_each(|index| {
                flags.insert(index, Flag::Glob(false));
            })
        }

        if let Some(indices) = matches.indices_of("glob_global") {
            indices.for_each(|index| {
                flags.insert(index, Flag::Glob(true));
            })
        }

        if let Some(indices) = matches.indices_of("regex") {
            indices.for_each(|index| {
                flags.insert(index, Flag::Regex(false));
            })
        }

        if let Some(indices) = matches.indices_of("regex_global") {
            indices.for_each(|index| {
                flags.insert(index, Flag::Regex(true));
            })
        }

        if let Some(indices) = matches.indices_of("include") {
            indices.for_each(|index| {
                flags.insert(index, Flag::Include(false));
            })
        }

        if let Some(indices) = matches.indices_of("include_global") {
            indices.for_each(|index| {
                flags.insert(index, Flag::Include(true));
            })
        }

        if let Some(indices) = matches.indices_of("exclude") {
            indices.for_each(|index| {
                flags.insert(index, Flag::Exclude(false));
            })
        }

        if let Some(indices) = matches.indices_of("exclude_global") {
            indices.for_each(|index| {
                flags.insert(index, Flag::Exclude(true));
            })
        }

        let mut glob_is_default = true;
        let mut include_is_default = true;

        let mut flags: Vec<(usize, Flag)> = flags.into_iter().collect();
        let mut include_patterns: Vec<Cow<'_, str>> = Vec::new();
        let mut exclude_patterns: Vec<Cow<'_, str>> = Vec::new();

        for (index, pattern_str) in pattern_indices.into_iter().zip(patterns) {
            let mut glob = glob_is_default;
            let mut include = include_is_default;

            let i = flags.partition_point(|(i, _)| *i < index);
            for (_, flag) in flags.drain(0..i) {
                match flag {
                    Flag::Glob(global) => {
                        if global {
                            glob_is_default = true;
                        }
                        glob = true;
                    }
                    Flag::Regex(global) => {
                        if global {
                            glob_is_default = false;
                        }
                        glob = false;
                    }
                    Flag::Include(global) => {
                        if global {
                            include_is_default = true;
                        }
                        include = true;
                    }
                    Flag::Exclude(global) => {
                        if global {
                            include_is_default = false;
                        }
                        include = false;
                    }
                }
            }

            let pattern: Cow<'_, str> = if glob {
                let regex = Glob::new(pattern_str)
                    .map_err(|e| clap::Error::raw(clap::error::ErrorKind::ValueValidation, e))?
                    .regex()
                    .to_string();
                Cow::Owned(regex)
            } else {
                Cow::Borrowed(pattern_str)
            };

            if include {
                include_patterns.push(pattern);
            } else {
                exclude_patterns.push(pattern);
            }
        }
        let include = RegexSetBuilder::new(include_patterns)
            .build()
            .map_err(|e| clap::Error::raw(clap::error::ErrorKind::ValueValidation, e))?;
        let exclude = RegexSetBuilder::new(exclude_patterns)
            .build()
            .map_err(|e| clap::Error::raw(clap::error::ErrorKind::ValueValidation, e))?;

        Ok(Self { include, exclude })
    }
    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        <Self as FromArgMatches>::from_arg_matches(matches).map(|other| {
            self.include = other.include;
            self.exclude = other.exclude;
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
                let members: [clap::Id; 9usize] = [
                    clap::Id::from("patterns"),
                    clap::Id::from("glob"),
                    clap::Id::from("glob_global"),
                    clap::Id::from("regex"),
                    clap::Id::from("regex_global"),
                    clap::Id::from("include"),
                    clap::Id::from("include_global"),
                    clap::Id::from("exclude"),
                    clap::Id::from("exclude_global"),
                ];
                members
            }))
            .arg(
                clap::Arg::new("patterns")
                    .value_name("PATTERN")
                    .num_args(1..)
                    .value_parser(clap::builder::NonEmptyStringValueParser::new())
                    .action(clap::ArgAction::Append)
                    .help("A pattern to match against each file")
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
                    .help("Parse pattern as a glob expression (default) [global alias: G]"),
            )
            .arg(
                clap::Arg::new("glob_global")
                    .short('G')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .hide(true),
            )
            .arg(
                clap::Arg::new("regex")
                    .short('r')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .help("Parse pattern as a regular expression [global alias: R]")
                    .long_help(
                        "Parse pattern as a regular expression.\n\n\
                               Note that expressions are unanchored by default. \
                               Use '^' or '\\\\A' to denote start, and '$' or \
                               '\\\\z' for the end.",
                    ),
            )
            .arg(
                clap::Arg::new("regex_global")
                    .short('R')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .hide(true),
            )
            .arg(
                clap::Arg::new("include")
                    .short('i')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .help(
                        "Matching files will be included in the output (default) [global alias: I]",
                    ),
            )
            .arg(
                clap::Arg::new("include_global")
                    .short('I')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .hide(true),
            )
            .arg(
                clap::Arg::new("exclude")
                    .short('e')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .help("Matching files will be excluded from the output [global alias: E]"),
            )
            .arg(
                clap::Arg::new("exclude_global")
                    .short('E')
                    .num_args(0)
                    .default_missing_value("true")
                    .default_value("false")
                    .action(clap::ArgAction::Append)
                    .hide(true),
            )
        }
    }
    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        {
            <Self as clap::Args>::augment_args(cmd)
        }
    }
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

fn main() {
    let Cli {
        walk_options,
        match_options:
            MatchOptions {
                patterns: Patterns { include, exclude },
                no_strip_prefix,
            },
    } = Cli::parse();

    let (tx, rx) = crossbeam_channel::unbounded::<PathBuf>();

    let walker = walk_options.into_builder().build_parallel();

    let mut stdout = io::BufWriter::new(io::stdout());

    let stdout_thread = std::thread::spawn(move || {
        for path_buf in rx {
            write_path(&mut stdout, path_buf.as_path());
        }
    });

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

            if !no_strip_prefix {
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
}
