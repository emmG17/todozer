use serde_json;
use crate::scanner::Todo;

pub fn to_json(todos: &[Todo]) -> String {
    let json = serde_json::to_string_pretty(todos).unwrap();
    println!("{}", json);
    json
}
