use std::process::Command;

fn stdout_to_lines_vec(stdout: Vec<u8>) -> Vec<String> {
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
        command!("ff", "--hidden", "--no-gitignore", "--no-ignore", "--no-strip-prefix", $($args),*)
    };
}

#[macro_export]
macro_rules! find {
    ($( $args: expr ),* ) => {
        command!("find", $($args),*)
    };
}

#[macro_export]
macro_rules! assert_same_output {
    ($left:expr, $right:expr) => {
        let left = $left;
        let right = $right;
        assert_eq!(left.status, right.status);
        assert_eq!(left.stderr, right.stderr);
        let mut left = stdout_to_lines_vec($left.stdout);
        left.sort();
        let mut right = stdout_to_lines_vec($right.stdout);
        right.sort();
        assert_eq!(left.len(), right.len(), "vectors are not of equal length");

        for (l, r) in left.iter().zip(right.iter()) {
            assert_eq!(l, r);
        }
    };
}

#[test]
fn test_one_glob_pattern() {
    assert_same_output!(ff!["*.rs"], find![".", "-wholename", "*.rs"]);
    assert_same_output!(ff!["*.toml"], find![".", "-wholename", "*.toml"]);
}

#[test]
fn test_one_regex_pattern() {
    assert_same_output!(ff!["-r", r".*\.c$"], find![".", "-regex", r".*\.c"]);
    assert_same_output!(ff!["-r", r".*\.h$"], find![".", "-regex", r".*\.h"]);
}
