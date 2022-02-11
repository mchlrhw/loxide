use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

fn extract_expected_data(line_num: usize, line: &str) -> Option<String> {
    if let Some(cap) = regex!(r"// expect: ?(.*)").captures_iter(line).next() {
        let capture = &cap[1];
        return Some(capture.to_string());
    }

    if let Some(cap) = regex!(r"// (Error.*)").captures_iter(line).next() {
        let capture = &cap[1];
        return Some(format!("[line {line_num}] {capture}"));
    }

    if let Some(cap) = regex!(r"// \[((java|c) )?line (\d+)\] (Error.*)")
        .captures_iter(line)
        .next()
    {
        if let Some("c") = cap.get(2).map(|m| m.as_str()) {
            return None;
        }
        let line_num = &cap[3];
        let capture = &cap[4];
        return Some(format!("[line {line_num}] {capture}"));
    }

    if let Some(cap) = regex!(r"// expect runtime error: (.+)")
        .captures_iter(line)
        .next()
    {
        let capture = &cap[1];
        return Some(format!("{capture}\n[line {line_num}]"));
    }

    if let Some(cap) = regex!(r"\[.*line (\d+)\] (Error.+)")
        .captures_iter(line)
        .next()
    {
        let line_num = &cap[1];
        let capture = &cap[2];
        return Some(format!("[line {line_num}] {capture}"));
    }

    if let Some(cap) = regex!(r"(\[line \d+\])").captures_iter(line).next() {
        let capture = &cap[1];
        return Some(capture.to_string());
    }

    None
}

fn run_test(bin_path: &Path, source_file: &Path) -> anyhow::Result<()> {
    println!("{source_file:?}");
    let source = fs::read_to_string(source_file)?;

    let mut expected = String::new();
    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        if let Some(line) = extract_expected_data(line_num, line) {
            expected.push_str(&format!("{line}\n"));
        }
    }

    let output = Command::new(bin_path).arg(source_file).output()?;

    let output = String::from_utf8(output.stdout)?;
    assert_eq!(output, expected);

    Ok(())
}

#[test]
fn crafting_interpreters_test_suite() -> anyhow::Result<()> {
    let root_dir = env::var("CARGO_MANIFEST_DIR")?;
    let pkg_name = env::var("CARGO_PKG_NAME")?;

    let mut bin_path = PathBuf::from(root_dir);
    bin_path.push("target/debug");
    bin_path.push(pkg_name);

    for source_file in WalkDir::new("resources/test")
        .into_iter()
        .filter_map(|path| path.ok())
    {
        if source_file.metadata()?.is_file() {
            let path = source_file.path();
            let path_str = path.to_str().expect("must be string");
            if !path_str.starts_with("resources/test/benchmark")
                && !path_str.starts_with("resources/test/expressions")
                && !path_str.starts_with("resources/test/limit")
                && !path_str.starts_with("resources/test/scanning")
                // FIXME: This is only ignored because the impl is incomplete.
                && !path_str.starts_with("resources/test/operator/equals_class.lox")
                // FIXME: Ditto.
                && !path_str.starts_with("resources/test/operator/equals_method.lox")
                // FIXME: Not sure about this one.
                && !path_str.starts_with("resources/test/field/set_evaluation_order.lox")
                // FIXME: Or this one.
                && !path_str.starts_with("resources/test/number/decimal_point_at_eof.lox")
            {
                run_test(&bin_path, path)?
            }
        }
    }

    Ok(())
}
