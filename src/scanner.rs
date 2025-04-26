use crate::cli::Cli;
use std::{fs::{self, File}, io::{BufRead, BufReader}, path::Path};

const TODO_SEARCH_TERMS: [&str; 10] = [
    "TODO", "FIXME", "HACK", "NOTE", "BUG", "todo", "fixme", "hack", "note", "bug",
];

const TODO_END_TERMS: [&str; 2] = [
    ":", "->",
];

const CODE_COMMENTS: [&str; 6] = [
    "//", // JavaScript, C, C++, Java, Go, Rust
    "#", // Python, Ruby, Perl, Shell
    "/*", // C, C++, Java, Go, Rust, JavaScript
    "'''", // Python
    "<!--", // HTML, XML
    "--", // Lua, SQL
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
        let mut path = cli.path.clone();
        loop {
            if is_git_repo(&path) {
                println!("File is in a git repository: {}", path);
                break;
            }
            // Get the parent directory
            path = match fs::canonicalize(&path) {
                Ok(p) => match p.parent() {
                    Some(p) => match p.to_str() {
                        Some(p) => {
                            // Check if the parent directory is a git repository
                            path = p.to_string();
                            path
                        }
                        None => break,
                    },
                    None => break,
                },
                Err(_) => break,
            };
        }
    }

    // Check if the path is a directory
    if fs::metadata(&cli.path).unwrap().is_dir() {
        println!("Path is a directory");
        // Check if the directory is a git repository
        if is_git_repo(&cli.path) {
            println!("Path is a git repository");
        } else {
            println!("Path is not a git repository");
        }
        return;
    }
}

fn is_git_repo(path: &str) -> bool {
    // Check if the path is a git repository
    let git_path = format!("{}/.git", path);
    return fs::metadata(&git_path).is_ok();
}

// TODO: Implement a function to find TODOs in the code
fn find_todos(path: &Path) {
    // Check if the path is a file

    // Get all the search terms + end terms combinations
    let search_terms: Vec<String> = TODO_SEARCH_TERMS.iter()
        .flat_map(|&term| {
            TODO_END_TERMS.iter().map(move |&end_term| format!("{}{}", term, end_term))
        })
        .collect();

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

        // Check if the line contains any of the TODO search terms and starts with a comment
        // indicator
        let mut found = false;
        for term in &search_terms {
            if line.contains(term) {
                found = true;
                break;
            }
            continue;
        }

        // Check if the line starts with a comment indicator
        let mut is_comment = false;
        for comment in &CODE_COMMENTS {
            if line.trim_start().starts_with(comment) {
                is_comment = true;
                break;
            }
        }

        // If the line contains a TODO search term and starts with a comment indicator, print it
        if found && is_comment {
            println!("Found TODO in file: {}", path.display());
            println!("Line {}: {}", line_number + 1, line);
        }
    }
}
