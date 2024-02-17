use std::collections::HashSet;
use std::process::Command;

const REPOS_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/repos");
const FF_BIN: &'static str = env!("CARGO_BIN_EXE_ff");

fn stdout_to_paths_set(stdout: Vec<u8>) -> HashSet<String> {
    String::from_utf8(stdout)
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect()
}

#[macro_export]
macro_rules! command {
    ($name: expr, $( $args: expr ),* ) => {
        Command::new($name).args(vec![$($args),*]).output().unwrap()
    };
}

#[macro_export]
macro_rules! ff {
    ($( $args: expr ),* ) => {
        command!(FF_BIN, "-d", REPOS_DIR, "--hidden", "--no-gitignore", "--no-ignore", "--no-strip-prefix", $($args),*)
    };
}

#[macro_export]
macro_rules! find {
    ($( $args: expr ),* ) => {
        command!("find", REPOS_DIR, $($args),*)
    };
}

#[macro_export]
macro_rules! assert_same_output {
    ($left:expr, $right:expr) => {
        let left = $left;
        let right = $right;
        assert_eq!(left.status, right.status);
        assert_eq!(left.stderr, right.stderr);
        let left = stdout_to_paths_set($left.stdout);
        let right = stdout_to_paths_set($right.stdout);

        let diff_lr: Vec<_> = left.difference(&right).collect();
        let diff_rl: Vec<_> = right.difference(&left).collect();

        assert!(
            diff_lr.is_empty() && diff_rl.is_empty(),
            "Outputs are differing: left contains {diff_lr:#?} and right contains {diff_rl:#?}"
        );
    };
}

#[test]
fn test_one_glob_pattern() {
    assert_same_output!(ff!["*.rs"], find!["-wholename", "*.rs"]);
    assert_same_output!(ff!["*.toml"], find!["-wholename", "*.toml"]);
}

#[test]
fn test_one_regex_pattern() {
    assert_same_output!(ff!["-r", r".*\.c$"], find!["-regex", r".*\.c"]);
    assert_same_output!(ff!["-r", r".*\.h$"], find!["-regex", r".*\.h"]);
}
