use std::fs;
use std::path::{Path, PathBuf};

fn source_files() -> Vec<PathBuf> {
    fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(dir).expect("source directory must be readable");
        for entry in entries {
            let path = entry.expect("directory entry must be readable").path();
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect(&root, &mut files);
    files.sort();
    files
}

fn strip_visibility(text: &str) -> &str {
    let mut text = text.trim();
    loop {
        if let Some(rest) = text.strip_prefix("pub(")
            && let Some(end) = rest.find(')')
        {
            text = rest[end + 1..].trim_start();
            continue;
        }
        if let Some(rest) = text.strip_prefix("pub ") {
            text = rest.trim_start();
            continue;
        }
        return text;
    }
}

fn is_type_declaration(line: &str) -> bool {
    let text = strip_visibility(line);
    ["struct ", "enum ", "trait "]
        .iter()
        .any(|keyword| text.starts_with(keyword))
}

fn is_impl_declaration(line: &str) -> bool {
    strip_visibility(line).starts_with("impl ")
}

fn is_plugin_impl(line: &str) -> bool {
    let text = strip_visibility(line);
    text.starts_with("impl Plugin for") || text.starts_with("impl bevy::app::Plugin for")
}

fn is_free_function(line: &str) -> bool {
    strip_visibility(line).starts_with("fn ")
}

fn is_test_module(line: &str) -> bool {
    let text = strip_visibility(line);
    text == "mod tests" || text.starts_with("mod tests ")
}

fn scope_ids(lines: &[&str]) -> Vec<usize> {
    let mut next_id = 0usize;
    let mut stack = vec![0usize];
    let mut result = Vec::with_capacity(lines.len());
    for line in lines {
        result.push(*stack.last().expect("scope stack is never empty"));
        let code = line.split("//").next().unwrap_or("");
        for character in code.chars() {
            match character {
                '{' => {
                    next_id += 1;
                    stack.push(next_id);
                }
                '}' => {
                    if stack.len() > 1 {
                        stack.pop();
                    }
                }
                _ => {}
            }
        }
    }
    result
}

#[test]
fn plugins_are_declared_in_mod_rs() {
    let mut violations = Vec::new();
    for file in source_files() {
        let source = fs::read_to_string(&file).expect("source file must be readable");
        if source.lines().any(is_plugin_impl)
            && file.file_name().is_none_or(|name| name != "mod.rs")
        {
            violations.push(file.display().to_string());
        }
    }
    assert!(
        violations.is_empty(),
        "plugins must be declared in mod.rs:\n{}",
        violations.join("\n")
    );
}

#[test]
fn system_files_declare_no_types() {
    let mut violations = Vec::new();
    for file in source_files() {
        if file.file_name().is_none_or(|name| name != "system.rs") {
            continue;
        }
        let source = fs::read_to_string(&file).expect("source file must be readable");
        for (index, line) in source.lines().enumerate() {
            if is_type_declaration(line) || is_plugin_impl(line) {
                violations.push(format!("{}:{}", file.display(), index + 1));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "system.rs must contain functions only:\n{}",
        violations.join("\n")
    );
}

#[test]
fn crate_paths_are_imported() {
    let mut violations = Vec::new();
    for file in source_files() {
        let source = fs::read_to_string(&file).expect("source file must be readable");
        let mut in_use_block = false;
        for (index, line) in source.lines().enumerate() {
            let text = line.trim();
            if in_use_block {
                if text.ends_with(';') {
                    in_use_block = false;
                }
                continue;
            }
            let starts_use = strip_visibility(text).starts_with("use ");
            if starts_use && text.contains('{') && !text.contains('}') {
                in_use_block = true;
                continue;
            }
            if !text.contains("crate::") {
                continue;
            }
            if starts_use
                || text.starts_with('#')
                || text.starts_with("//")
                || text.contains("$crate::")
                || text.contains("ron_document!")
            {
                continue;
            }
            violations.push(format!("{}:{}", file.display(), index + 1));
        }
    }
    assert!(
        violations.is_empty(),
        "crate:: paths must be imported at file scope:\n{}",
        violations.join("\n")
    );
}

#[test]
fn declarations_precede_functions_and_tests_are_last() {
    let mut violations = Vec::new();
    for file in source_files() {
        let source = fs::read_to_string(&file).expect("source file must be readable");
        let lines: Vec<&str> = source.lines().collect();
        let scopes = scope_ids(&lines);
        let mut seen_function = false;
        let mut test_module_line: Option<usize> = None;
        for (index, line) in lines.iter().enumerate() {
            if scopes[index] != 0 {
                continue;
            }
            let text = line.trim();
            if text.is_empty() || text.starts_with("//") || text.starts_with('#') {
                continue;
            }
            let is_type = is_type_declaration(text) || is_impl_declaration(text);
            let is_function = is_free_function(text);
            if is_test_module(text) {
                test_module_line = Some(index);
            } else if test_module_line.is_some() {
                violations.push(format!(
                    "{}:{} item follows the test module",
                    file.display(),
                    index + 1
                ));
            } else if is_type && seen_function {
                violations.push(format!(
                    "{}:{} type declaration follows the first free function",
                    file.display(),
                    index + 1
                ));
            }
            if is_function {
                seen_function = true;
            }
        }
    }
    assert!(
        violations.is_empty(),
        "types must precede functions and tests must be last:\n{}",
        violations.join("\n")
    );
}
