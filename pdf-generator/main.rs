use std::fmt::Write;
use std::fs;
use std::io::{Read, Write as ioWrite};
use std::process::Stdio;

fn md5sum(s: &str) -> String {
    // Because lines are 1-indexed
    let cmd = std::process::Command::new("sh")
        .arg("-c")
        .arg("cpp -dD -P -fpreprocessed | tr -d '[:space:]' | md5sum | cut -c-6")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start command");

    let mut stdin = cmd.stdin.as_ref().expect("Failed to open stdin");
    stdin
        .write_all(s.as_bytes())
        .expect("Failed to write to stdin");
    String::from_utf8(
        cmd.wait_with_output()
            .unwrap()
            .stdout
            .into_iter()
            .take(3)
            .collect(),
    )
    .unwrap()
}

fn ignore_line(line: &str) -> bool {
    line.is_empty() || line.starts_with("//") || line.starts_with("warning")
}

fn trim_comments(line: &str) -> &str {
    if let Some((pre, _comments)) = line.split_once("//") {
        pre
    } else {
        line
    }
}

/// Returns the hashes list of a file
fn hashes(lines: &[String], path: &str) -> Vec<String> {
    let mut scopes: Vec<usize> = Vec::new();

    let mut hashes = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let line = trim_comments(line).trim_end();
        if ignore_line(line) {
            hashes.push("---".to_string());
            continue;
        }
        let mut scope = index;
        for c in line.chars() {
            if c == '{' {
                scopes.push(index);
            } else if c == '}' {
                scope = *scopes.last().expect(&format!(
                    "Parse error with scopes in file {path} in line {}",
                    index + 1
                ));
                scopes.pop();
            }
        }
        let scope_code = lines[scope..=index]
            .iter()
            .map(String::as_str)
            .filter(|&s| !ignore_line(s))
            .map(|s| {
                if let Some((pre, _comments)) = s.split_once("//") {
                    pre
                } else {
                    s
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        hashes.push(md5sum(&scope_code));
    }

    hashes
}

/// For each section, do something like this
/// == Section
/// === Code
/// #cpp(`int main() {
/// }`, hashes: ("foo", "bar"))
fn codes() -> Result<String, std::fmt::Error> {
    let mut sections = fs::read_dir("code/")
        .expect("Path `code/ doesn't exists!")
        .map(|entry| entry.expect("Failed to get entry").path())
        .collect::<Vec<_>>();
    sections.sort_by_key(|p| p.file_name().unwrap().to_string_lossy().to_lowercase());

    let mut result = String::new();

    for section in sections {
        let section_file_name = section.file_name();
        let section_file_name_str = section_file_name.unwrap().to_string_lossy();

        writeln!(&mut result, "== {section_file_name_str}").unwrap();
        let mut codes = fs::read_dir(section.as_path())
            .expect(&format!("{section_file_name_str} is not a directory!"))
            .map(|entry| entry.expect("Failed to get entry").path())
            .collect::<Vec<_>>();
        codes.sort_by_key(|p| p.file_name().unwrap().to_string_lossy().to_lowercase());
        for code in codes {
            let file_name: String = code.file_name().unwrap().to_string_lossy().to_string();
            let mut file = fs::File::open(&code).expect(&format!("{code:?} is not a file!"));

            assert!(
                file_name.ends_with(".cpp"),
                "{} is not a cpp file!",
                file_name
            );
            let file_name = file_name.trim_end_matches(".cpp");

            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            if content.find("/*").is_some() {
                panic!("`/*` comments not supported");
            }

            if content.find("`").is_some() {
                panic!("backticks (\"`\") are not supported");
            }

            let lines: Vec<String> = content.lines().map(|s| s.to_owned()).collect();

            writeln!(&mut result, "=== {file_name}")?;
            writeln!(
                &mut result,
                "#cpp(`{}`, hashes: ({}))",
                &lines.join("\n"),
                &hashes(&lines, &code.as_path().to_string_lossy())
                    .into_iter()
                    .map(|s| format!("\"{s}\""))
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }
    }

    Ok(result)
}

fn main() {
    let codes = codes().unwrap();
    println!("{codes}");
}
