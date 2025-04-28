use crate::{cli::Cli, git::find_blame, serialize::to_json};
use serde::Serialize;
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

const TODO_SEARCH_TERMS: [&str; 5] = [
    "TODO", "FIXME", "HACK", "NOTE", "BUG"
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
        let repo_root = find_git_repo(&cli.path);

        let is_some_repo = repo_root.is_some();

        let mut all_todos = Vec::new();

        // Iterate over all files in the directory
        let walker = walkdir::WalkDir::new(&cli.path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file() && is_valid_extension(e.path()));

        for entry in walker {
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

/// Check if the path is a git repository
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
    let found_term = search_terms.iter().find(|term| line.to_uppercase().contains(*term));
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

    match find_git_repo(path.as_str()) {
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
