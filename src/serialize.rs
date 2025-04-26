use crate::scanner::Todo;
use serde_json;
use std::{fs::File, path::PathBuf};

pub fn to_json(todos: &[Todo], out: &str) {
    let out_path = PathBuf::from(out);

    if let Err(e) = File::create(&out_path).and_then(|file| {
        serde_json::to_writer_pretty(file, todos)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }) {
        eprintln!("Unable to write results to {}: {}", out_path.display(), e);
    }
}
