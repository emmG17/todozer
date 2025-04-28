use crate::{
    cli::Cli,
    git::{self, find_blame},
    serialize::to_json,
};
use git2::Repository;
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(Debug, Serialize)]
pub struct Todo {
    pub title: String,
    pub author: String,
    pub email: String,
    pub datetime: String,
    pub file: String,
    pub line: usize,
}

pub struct NaiveTodo {
    pub line_number: usize,
    pub file_path: String,
    pub value: String,
}

const TODO_SEARCH_TERMS: [&str; 5] = ["TODO", "FIXME", "HACK", "NOTE", "BUG"];

const TODO_END_TERMS: [&str; 3] = [":", "->", " "];

const CODE_COMMENTS: [&str; 6] = [
    "//",   // JavaScript, C, C++, Java, Go, Rust
    "#",    // Python, Ruby, Perl, Shell
    "/*",   // C, C++, Java, Go, Rust, JavaScript
    "'''",  // Python
    "<!--", // HTML, XML
    "--",   // Lua, SQL
];

const ALLOWED_EXTENSIONS: [&str; 23] = [
    "rs", "js", "ts", "go", "java", "py", "rb", "sh", "c", "cpp", "html", "xml", "lua", "sql",
    "tsx", "jsx", "css", "scss", "less", "json", "yaml", "yml", "toml",
];

pub fn run(cli: &Cli) {
    // Check if the path exists
    if !fs::metadata(&cli.path).is_ok() {
        eprintln!("Path does not exist");
        return;
    }

    // Check if the path is a file
    if fs::metadata(&cli.path).unwrap().is_file() {
        let dir = Path::new(&cli.path);
        handle_file(dir, cli);
        return;
    }

    // Check if the path is a directory
    if fs::metadata(&cli.path).unwrap().is_dir() {
        let repo_root = git::find_git_repo(&cli.path);

        let is_some_repo = repo_root.is_some();

        let mut all_todos = Vec::new();

        let git_repo = if is_some_repo {
            Some(Repository::open(repo_root.clone().unwrap()).unwrap())
        } else {
            None
        };

        // Iterate over all files in the directory
        let walker = walkdir::WalkDir::new(&cli.path).follow_links(false);
        let files = filter_files(walker, git_repo);

        for entry in files {
            let file_path = entry.path();
            let todos = find_todos(file_path);
            if is_some_repo {
                // If the path is a git repository, get the full todos
                let repo = repo_root.clone().unwrap();
                let full_todos = find_blame(&repo, &file_path.to_path_buf(), &todos);
                all_todos.extend(full_todos);
            } else {
                let full_todos = naive_to_full(&todos);
                all_todos.extend(full_todos);
            }
        }

        to_json(&all_todos, &cli.out);
    }
}

fn filter_files(walker: walkdir::WalkDir, git_repo: Option<Repository>) -> Vec<walkdir::DirEntry> {
    walker
        .into_iter()
        // Use filter_entry to skip traversing into ignored directories entirely
        .filter_entry(|entry| {
            if let Some(repo) = &git_repo {
                if entry.path().is_dir() {
                    let dir_relative = git::relative_path(
                        &repo.workdir().unwrap().to_path_buf(),
                        &entry.path().to_path_buf(),
                    );

                    return !repo.status_should_ignore(&dir_relative).unwrap_or(false);
                }
            }
            true // Continue with traversal
        })
        .filter_map(Result::ok)
        // Filter files with valid extensions
        .filter(|e| e.path().is_file() && is_valid_extension(e.path()))
        // Filter out ignored files
        .filter(|e| {
            if let Some(repo) = &git_repo {
                let file_relative = git::relative_path(
                    &repo.workdir().unwrap().to_path_buf(),
                    &e.path().to_path_buf(),
                );

                if let Ok(status) = repo.status_file(&file_relative) {
                    return !status.is_ignored();
                }
            }
            true
        })
        .collect()
}

// TODO: Implement a function to find TODOs in the code
fn find_todos(path: &Path) -> Vec<NaiveTodo> {
    let mut todos: Vec<NaiveTodo> = Vec::new();
    // Get all the search terms + end terms combinations
    let search_terms = search_items_combinations(&TODO_SEARCH_TERMS, &TODO_END_TERMS);
    // Seach temrs

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Error opening file: {}", path.display());
            return vec![];
        }
    };

    let reader = BufReader::new(file);

    for (line_number, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => {
                eprintln!("Error reading line {}: {}", line_number, path.display());
                continue;
            }
        };

        match todo_matcher(path, &line, line_number, &search_terms) {
            Some(todo) => {
                todos.push(todo);
            }
            None => continue,
        }
    }

    return todos;
}

/// Check if the line contains a TODO and is a valid comment
fn todo_matcher(
    file: &Path,
    line: &str,
    line_number: usize,
    search_terms: &[String],
) -> Option<NaiveTodo> {
    let found_term = search_terms
        .iter()
        .find(|term| line.to_uppercase().contains(*term));

    let comment_string = CODE_COMMENTS
        .iter()
        .find(|term| line.trim_start().starts_with(*term));

    if let (Some(term), Some(comment)) = (found_term, comment_string) {
        let value = get_todo_value(line, term, comment);
        let todo = NaiveTodo {
            line_number,
            value,
            file_path: file.display().to_string(),
        };
        return Some(todo);
    }

    None
}

/// Get the value of the TODO from the line
fn get_todo_value(line: &str, term: &str, comment: &str) -> String {
    line.trim_start()
        .strip_prefix(comment)
        .unwrap_or(line)
        .trim_start()
        .strip_prefix(term)
        .unwrap_or(line)
        .trim_start()
        .to_string()
}

/// Generate all combinations of search terms and end terms
fn search_items_combinations(search_terms: &[&str], end_terms: &[&str]) -> Vec<String> {
    let mut combinations = Vec::new();
    for &term in search_terms {
        for &end_term in end_terms {
            combinations.push(format!("{}{}", term, end_term));
        }
    }
    return combinations;
}

/// Handle a single file search for TODOs
fn handle_file(file_path: &Path, cli: &Cli) {
    let todos = find_todos(file_path);

    let path = cli.path.clone();

    match git::find_git_repo(path.as_str()) {
        Some(repo) => {
            let full_todos = find_blame(&repo, &file_path.to_path_buf(), &todos);
            to_json(&full_todos, &cli.out)
        }
        None => {
            let full_todos = naive_to_full(&todos);
            to_json(&full_todos, &cli.out);
        }
    };
}

/// Check if the file extension is valid
fn is_valid_extension(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => ALLOWED_EXTENSIONS.contains(&ext),
        None => false,
    }
}

/// Converts a vector of NaiveTodo to a vector of Todo
fn naive_to_full(naives: &[NaiveTodo]) -> Vec<Todo> {
    naives
        .iter()
        .map(|nt| Todo {
            title: nt.value.clone(),
            author: "".to_string(),
            email: "".to_string(),
            datetime: "".to_string(),
            file: nt.file_path.clone(),
            line: nt.line_number,
        })
        .collect()
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn dummy_path() -> &'static Path {
        Path::new("dummy.rs")
    }

    #[test]
    fn matches_todo_in_comment() {
        let search_terms = search_items_combinations(&TODO_SEARCH_TERMS, &TODO_END_TERMS);
        let line = "// TODO: fix this function";
        let result = todo_matcher(dummy_path(), line, 5, &search_terms);
        assert!(result.is_some());
        let todo = result.unwrap();
        assert_eq!(todo.line_number, 5);
        assert_eq!(todo.value, "fix this function"); // assuming get_todo_value trims correctly
    }

    #[test]
    fn matches_todo_without_two_colons() {
        let search_terms = search_items_combinations(&TODO_SEARCH_TERMS, &TODO_END_TERMS);
        let line  = "// TODO Agregar toast de rror";
        let result = todo_matcher(dummy_path(), line, 8, &search_terms);
        assert!(result.is_some());
        let todo = result.unwrap();
        assert_eq!(todo.line_number, 8);
        assert_eq!(todo.value, "fix this function");
    }

    #[test]
    fn matches_fixme_in_comment() {
        let search_terms = search_items_combinations(&TODO_SEARCH_TERMS, &TODO_END_TERMS);
        let line = "// FIXME: broken logic here";
        let result = todo_matcher(dummy_path(), line, 10, &search_terms);
        assert!(result.is_some());
        let todo = result.unwrap();
        assert_eq!(todo.line_number, 10);
        assert_eq!(todo.value, "broken logic here");
    }

    #[test]
    fn ignores_lines_without_comment_prefix() {
        let search_terms = search_items_combinations(&TODO_SEARCH_TERMS, &TODO_END_TERMS);
        let line = "TODO: no comment marker";
        let result = todo_matcher(dummy_path(), line, 3, &search_terms);
        assert!(result.is_none());
    }

    #[test]
    fn ignores_lines_without_search_term() {
        let search_terms = search_items_combinations(&TODO_SEARCH_TERMS, &TODO_END_TERMS);
        let line = "// WHAT: just a note, not a todo";
        let result = todo_matcher(dummy_path(), line, 7, &search_terms);
        assert!(result.is_none());
    }
}
