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
macro_rules! assert_correct_suffix {
    (@include $output:expr, $suffixes:expr) => {
        let files = stdout_to_lines_vec($output.stdout);
        let suffixes = $suffixes;

        for file in files.iter() {
            assert!(
                suffixes.iter().any(|suff| file.ends_with(suff)),
                "file {} does end with any of {:?}",
                file,
                suffixes
            );
        }
    };
    (@exclude $output:expr, $suffixes:expr) => {
        let files = stdout_to_lines_vec($output.stdout);
        let suffixes = $suffixes;

        for file in files.iter() {
            assert!(
                !suffixes.iter().any(|suff| file.ends_with(suff)),
                "file {} shoud not end with any of {:?}",
                file,
                suffixes
            );
        }
    };
}

#[test]
fn test_one_glob_pattern() {
    assert_correct_suffix!(@include command!["ff", "*.rs"], &[".rs"]);
    assert_correct_suffix!(@exclude command!["ff", "*", "-e", "*.rs"], &[".rs"]);
}

#[test]
fn test_one_regex_pattern() {
    assert_correct_suffix!(@include command!["ff", "-r", r".*\.c$"], &[".c"]);
    assert_correct_suffix!(@exclude command!["ff", "*", "-er", r".*\.c$"], &[".c"]);
}

#[test]
fn test_two_glob_patterns() {
    assert_correct_suffix!(@include command!["ff", "*.rs", "*.toml"], &[".rs", ".toml"]);
    assert_correct_suffix!(@exclude command!["ff", "*", "-e", "*.rs", "-e", "*.toml"], &[".rs", ".toml"]);
}

#[test]
fn test_two_regex_patterns() {
    assert_correct_suffix!(@include command!["ff", "-r", r".*\.c$", "-r", r".*\.h$"], &[".c", ".h"]);
    assert_correct_suffix!(@exclude command!["ff", "*", "-er", r".*\.c$", "-er", r".*\.h$"], &[".c", ".h"]);
}
