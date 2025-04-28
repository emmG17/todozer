use std::path::PathBuf;
use std::fs;

use chrono::DateTime;
use git2::{Repository, Status};

use crate::scanner::{NaiveTodo, Todo};

pub struct BlameData {
    author: String,
    email: String,
    time: String,
}

pub fn find_blame(repo_dir: &PathBuf, file: &PathBuf, todos: &Vec<NaiveTodo>) -> Vec<Todo> {
    let mut full_todos: Vec<Todo>= Vec::new();
    // Open the repository
    let repo = match get_repo(repo_dir) {
        Some(repo) => repo,
        None => {
            eprintln!("Error opening repository");
            return vec![];
        }
    };

    // File relative to the repository
    let file_relative = relative_path(&repo_dir, file);

    // Verify that the file is not ignored by .gitignore
    let ignore = repo.status_file(&file_relative).unwrap_or(Status::IGNORED);

    if ignore == Status::IGNORED {
        eprintln!("File {} is ignored by .gitignore", file_relative.display());
        return vec![];
    }

    // Get the blames for each TODO
    todos.iter().for_each(|t| {
        match line_blame(&repo, &file_relative, t.line_number) {
            Some(blame) => full_todos.push(Todo {
                title: t.value.clone(),
                author: blame.author,
                email: blame.email,
                datetime: blame.time,
                file: t.file_path.clone(),
                line: t.line_number
            }),
            None => {
                eprintln!("Error getting blame for line {}", t.line_number);
                return;
            }
        };
    });

    return full_todos;
}

fn get_repo(path: &PathBuf) -> Option<Repository> {
    let repo = match Repository::discover(path) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Error opening repository: {}", e);
            return None;
        }
    };
    Some(repo)
}

fn line_blame(
    repo: &Repository,
    file: &PathBuf,
    line_number: usize,
) -> Option<BlameData> {
    let blame = match repo.blame_file(&file, None) {
        Ok(blame) => blame,
        Err(e) => {
            eprintln!("Error getting blame: {}", e);
            return Some(BlameData {
                author: "".to_string(),
                email: "".to_string(),
                time: "".to_string()
            })
        }
    };
    let hunk = blame.get_line(line_number).unwrap();
    let signature = hunk.final_signature();

    let author = signature.name().unwrap_or("Unknown").to_string();
    let email = signature.email().unwrap_or("Unknown").to_string();

    let timestamp = signature.when().seconds();
    let datetime = DateTime::from_timestamp(timestamp, 0).unwrap().to_rfc3339();

    let blame_data = BlameData {
        author,
        email,
        time: datetime,
    };
    Some(blame_data)
}

pub fn relative_path(repo: &PathBuf, file: &PathBuf) -> PathBuf {
    // Make both paths absolute first
    let file_abs = fs::canonicalize(file).unwrap_or(file.clone());
    let repo_path = fs::canonicalize(repo).unwrap_or(repo.to_path_buf());

    // Strip the repo path from the file path
    let file_relative = file_abs.strip_prefix(repo_path).unwrap_or(&file_abs);
    return file_relative.to_path_buf();
}
