Got it! Here's the updated README with your changes:

---

# Todozer

**todozer** is a simple CLI tool that searches for TODO comments in source files.  
It can work on individual files or entire directories, and optionally enrich TODOs with Git blame information (author, timestamp, email).

> Find it. Fix it. Finish it.

---

## Features

- Scan a **file** or a **directory** recursively.
- Supports **filtering** by valid file extensions (e.g., `.rs`, `.ts`, `.js`, etc.).
- Detects TODOs inside code comments (e.g., `//`, `#`, `<!-- -->`).
- **Optionally** enriches results with Git metadata (author, email, timestamp).
- Outputs the results as a **JSON** file.
- Gracefully handles repositories and non-repositories alike.

---

## Installation

```bash
git clone https://github.com/your-username/todozer.git
cd todozer
cargo build --release
```

The final binary will be located at `target/release/todozer`.

---

## Usage

```bash
todozer --path <path-to-scan> --out <output-file.json>
```

### Example

```bash
todozer --path src/ --out todos.json
```

This will:

- Scan all supported files inside the `src/` directory.
- Find all TODOs inside comments.
- If inside a Git repo, attach blame information.
- Export everything to `todos.json`.

---

## ⚙️ Options

| Option    | Description                        |
|-----------|------------------------------------|
| `--path`  | Path to a file or directory to scan |
| `--out`   | Output JSON file path               |

---

## Example Output

```json
[
  {
    "title": "Refactor this function",
    "author": "Jane Doe",
    "email": "jane@example.com",
    "datetime": "2025-04-26T14:52:00",
    "file": "src/main.rs",
    "line": 42
  },
  {
    "title": "Handle error case here",
    "author": "",
    "email": "",
    "datetime": "",
    "file": "src/utils.rs",
    "line": 13
  }
]
```

> When Git metadata is not available (e.g., the file is outside a Git repo), fields like `author`, `email`, and `datetime` default to empty strings (`""`).

---

## Notes

- Only files with **valid extensions** will be scanned.
- Currently supported comment formats include `//`, `#`, and HTML `<!-- -->`.

---

## Roadmap

- [ ] Support custom TODO keywords (e.g., `FIXME`, `HACK`).
- [ ] Add option to ignore certain directories (e.g., `node_modules`).
- [ ] Add CLI progress bar.
- [ ] Export other formats (e.g., Markdown, CSV).

---

## Contributing

Contributions, issues, and feature requests are welcome!  
Feel free to open a Pull Request or an Issue. 🚀

---

## License

This project is licensed under the [MIT License](LICENSE).
