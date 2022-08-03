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
        command!("ff", "--no-gitignore", $($args),*)
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
        println!("{:?}", right);
        assert_eq!(left.status, right.status);
        assert_eq!(left.stderr, right.stderr);
        let mut left = stdout_to_lines_vec($left.stdout);
        left.sort();
        let mut right = stdout_to_lines_vec($left.stdout);
        right.sort();
        assert_eq!(left, right);
    };
}

#[test]
fn test_one_glob_pattern() {
    assert_same_output!(ff!["*.rs"], find![".", "-type", "f", "-name", "*.rs"]);
    assert_same_output!(ff!["*.toml"], find![".", "-type", "f", "-name", "*.toml"]);
}

#[test]
fn test_one_regex_pattern() {
    assert_same_output!(
        ff!["-r", r".*\.c"],
        find![".", "-type", "f", "-regex", r".*\.c"]
    );
    assert_same_output!(
        ff!["-r", r".*\.h"],
        find![".", "-type", "f", "-regex", r".*\.h"]
    );
}
