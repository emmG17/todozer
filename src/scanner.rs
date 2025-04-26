use crate::cli::Cli;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

struct NaiveTodo {
    line_number: usize,
    line: String,
    file_path: String,
    value: String,
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
        println!("Path is a file");
        let dir = Path::new(&cli.path);
        find_todos(&dir);
        // Check if the file is a git repository, recurse until we find the first parent directory
        // that is a git repository
        let path = cli.path.clone();
        let repo = match find_git_repo(path.as_str()) {
            Some(repo) => repo,
            None => {
                println!("Path is not a git repository");
                return;
            }
        };
        println!("File is in a git repository: {}", repo.display());
    }

    // Check if the path is a directory
    if fs::metadata(&cli.path).unwrap().is_dir() {
        println!("Path is a directory");
        // Check if the directory is a git repository
        let dir = PathBuf::from(&cli.path);
        if is_git_repo(&dir) {
            println!("Path is a git repository");
        } else {
            println!("Path is not a git repository");
        }
        return;
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
fn find_todos(path: &Path) {
    // Get all the search terms + end terms combinations
    let search_terms = search_items_combinations(&TODO_SEARCH_TERMS, &TODO_END_TERMS);

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Error opening file: {}", path.display());
            return;
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

        todo_matcher(path, &line, line_number, &search_terms).map(|todo| {
            println!(
                "Found TODO in {} - {}: {} -> {}",
                todo.file_path,
                todo.line_number + 1,
                todo.line,
                todo.value
            );
        });
    }
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
            line: line.to_string(),
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
