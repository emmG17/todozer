use serde::Serialize;
use crate::{cli::Cli, git::find_blame, serialize::to_json};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
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

const TODO_SEARCH_TERMS: [&str; 10] = [
    "TODO", "FIXME", "HACK", "NOTE", "BUG", "todo", "fixme", "hack", "note", "bug",
];

const TODO_END_TERMS: [&str; 2] = [":", "->"];

const CODE_COMMENTS: [&str; 6] = [
    "//",   // JavaScript, C, C++, Java, Go, Rust
    "#",    // Python, Ruby, Perl, Shell
    "/*",   // C, C++, Java, Go, Rust, JavaScript
    "'''",  // Python
    "<!--", // HTML, XML
    "--",   // Lua, SQL
];

pub fn run(cli: &Cli) {
    // Check if the path exists
    if !fs::metadata(&cli.path).is_ok() {
        println!("Path does not exist");
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
        let repo_root = match find_git_repo(&cli.path) {
            Some(repo) => repo,
            None => {
                println!("No git repository found");
                return;
            }
        };

        let mut all_todos = Vec::new();

        // Iterate over all files in the directory
        let walker = walkdir::WalkDir::new(&cli.path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file());

        for entry in walker {
            let file_path = entry.path();

            let todos = find_todos(file_path);
            let full_todos = find_blame(&repo_root, &file_path.to_path_buf(), &todos);
            all_todos.extend(full_todos);
        }

        to_json(&all_todos, &cli.out); 
    }
}

/// Finds the first parent directory that is a git repository
fn find_git_repo(origin_path: &str) -> Option<PathBuf> {
    // Check if the file is a git repository, recurse until we find the first parent directory
    // that is a git repository
    let mut path = PathBuf::from(origin_path);

    loop {
        if is_git_repo(&path) {
            return Some(path);
        }

        match fs::canonicalize(&path)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        {
            Some(parent) => {
                path = parent;
            }
            None => return None,
        }
    }
}

fn is_git_repo(path: &PathBuf) -> bool {
    // Check if the path is a git repository
    let git_path = format!("{}/.git", path.display());
    return fs::metadata(&git_path).is_ok();
}

// TODO: Implement a function to find TODOs in the code
fn find_todos(path: &Path) -> Vec<NaiveTodo> {
    let mut todos: Vec<NaiveTodo> = Vec::new();
    // Get all the search terms + end terms combinations
    let search_terms = search_items_combinations(&TODO_SEARCH_TERMS, &TODO_END_TERMS);

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
            None => continue
        }
    }

    return todos;
}

fn todo_matcher(
    file: &Path,
    line: &str,
    line_number: usize,
    search_terms: &[String],
) -> Option<NaiveTodo> {
    let found_term = search_terms.iter().find(|term| line.contains(*term));
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

fn search_items_combinations(search_terms: &[&str], end_terms: &[&str]) -> Vec<String> {
    let mut combinations = Vec::new();
    for &term in search_terms {
        for &end_term in end_terms {
            combinations.push(format!("{}{}", term, end_term));
        }
    }
    return combinations;
}

fn handle_file(file_path: &Path, cli: &Cli) {
    let todos = find_todos(file_path);

    let path = cli.path.clone();

    match find_git_repo(path.as_str()) {
        Some(repo) => {
            let full_todos = find_blame(&repo, &file_path.to_path_buf(), &todos);
            to_json(&full_todos, &cli.out)
        }
        None => {
            return;
        }
    };
}
