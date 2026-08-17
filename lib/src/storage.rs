use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

/// Name of the directory where tataki keeps its internal state.
const TATAKI_DIR: &str = ".tataki";

/// Find the project root: the closest ancestor of the current directory that
/// contains a `.tataki` folder.
fn find_root() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| find_root_from(&cwd))
}

fn find_root_from(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(TATAKI_DIR).is_dir())
        .map(|dir| dir.to_path_buf())
}

/// File-based storage rooted at a project directory.
///
/// State files live under `<root>/.tataki/`. Snapshots are stored as pretty
/// JSON documents (`.json`); logs (like the commit log) are stored as JSONL
/// (one JSON document per line) and appended to.
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Open the store for the project containing the current directory.
    /// Returns `None` if no project root is found.
    pub fn open_default() -> Option<Self> {
        find_root().map(Self::open)
    }

    /// Open the store rooted at the given directory.
    pub fn open(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// The project root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn state_dir(&self) -> PathBuf {
        self.root.join(TATAKI_DIR)
    }

    fn path(&self, name: impl AsRef<Path>) -> PathBuf {
        self.state_dir().join(name)
    }

    /// Load a JSON document from a file named `name`.
    pub(crate) fn load<T: DeserializeOwned>(
        &self,
        name: impl AsRef<Path>,
    ) -> Result<T, StorageError> {
        let path = self.path(name);
        let contents = fs::read_to_string(&path).map_err(|e| StorageError::Read {
            path: path.clone(),
            source: e,
        })?;
        serde_json::from_str(&contents).map_err(|e| StorageError::Parse { path, source: e })
    }

    /// Save a JSON document to a file named `name`, atomically.
    pub(crate) fn save<T: Serialize>(
        &self,
        name: impl AsRef<Path>,
        value: &T,
    ) -> Result<(), StorageError> {
        let contents = serde_json::to_string_pretty(value).map_err(StorageError::Serialize)?;
        atomic_write(&self.path(name), &contents)
    }

    /// Append a record to the JSONL file named `name`.
    pub(crate) fn append<T: Serialize>(
        &self,
        name: impl AsRef<Path>,
        value: &T,
    ) -> Result<(), StorageError> {
        let line = serde_json::to_string(value).map_err(StorageError::Serialize)?;
        ensure_dir(&self.state_dir())?;
        let path = self.path(name);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| StorageError::Write {
                path: path.clone(),
                source: e,
            })?;
        writeln!(file, "{line}").map_err(|e| StorageError::Write { path, source: e })
    }

    /// Load all records from the JSONL file named `name`.
    /// Returns an empty vec if the file does not exist.
    pub(crate) fn load_all<T: DeserializeOwned>(
        &self,
        name: impl AsRef<Path>,
    ) -> Result<Vec<T>, StorageError> {
        let path = self.path(name);
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(StorageError::Read { path, source: e }),
        };
        let read_err = |e: io::Error| StorageError::Read {
            path: path.clone(),
            source: e,
        };
        let parse_err = |line: usize, e: serde_json::Error| StorageError::ParseLine {
            path: path.clone(),
            line,
            source: e,
        };
        let mut records = Vec::new();
        for (line_no, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(&read_err)?;
            if line.trim().is_empty() {
                continue;
            }
            let record = serde_json::from_str(&line).map_err(|e| parse_err(line_no + 1, e))?;
            records.push(record);
        }
        Ok(records)
    }
}

/// Errors that can occur during storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("tataki not initialised - run `tt init`")]
    NotInitialised,

    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to parse {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to parse {path} at line {line}: {source}")]
    ParseLine {
        path: PathBuf,
        line: usize,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to serialize state: {0}")]
    Serialize(serde_json::Error),
}

/// Write `contents` to `path` atomically: write a sibling temp file, then
/// rename it over `path`. The rename is atomic, so readers never see a
/// partially written file. Not fsync'd, so a power loss may lose the new
/// content (the old file stays intact).
fn atomic_write(path: &Path, contents: &str) -> Result<(), StorageError> {
    ensure_dir(path.parent().unwrap())?;
    let tmp = path.with_file_name(format!(
        "{}.tmp",
        path.file_name().unwrap().to_string_lossy()
    ));
    fs::write(&tmp, contents).map_err(|e| StorageError::Write {
        path: tmp.clone(),
        source: e,
    })?;
    fs::rename(&tmp, path).map_err(|e| StorageError::Write {
        path: tmp,
        source: e,
    })
}

fn ensure_dir(dir: &Path) -> Result<(), StorageError> {
    fs::create_dir_all(dir).map_err(|e| StorageError::Write {
        path: dir.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::d;
    use crate::test::dir;
    use crate::types::experimental::book::Book;
    use crate::types::experimental::journal::Commit;
    use crate::types::experimental::journal::Journal;

    /// A temp directory for a single test, removed on drop.
    struct TmpDir(PathBuf);

    impl TmpDir {
        fn new(name: &str) -> Self {
            Self(dir(name))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TmpDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_save_load_roundtrip() {
        let dir = TmpDir::new("roundtrip");
        let store = Store::open(dir.path());
        let book = Book::new(vec![]);
        store.save("book.json", &book).unwrap();
        let loaded: Book = store.load("book.json").unwrap();
        assert_eq!(book, loaded);
    }

    #[test]
    fn test_load_missing_file() {
        let dir = TmpDir::new("missing");
        let store = Store::open(dir.path());
        let result: Result<Book, _> = store.load("book.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_append_and_load_all() {
        let dir = TmpDir::new("append");
        let store = Store::open(dir.path());
        let ts = d(2025, 10, 23, 14, 0, 0);
        store
            .append("journal.jsonl", &Commit::completed("1".into(), ts))
            .unwrap();
        store
            .append("journal.jsonl", &Commit::postponed("2".into(), ts))
            .unwrap();

        let commits: Vec<Commit> = store.load_all("journal.jsonl").unwrap();
        let journal = Journal::new(commits);
        assert_eq!(
            Some(&Commit::completed("1".into(), ts)),
            journal.last_commit_for("1")
        );
        assert_eq!(
            Some(&Commit::postponed("2".into(), ts)),
            journal.last_commit()
        );
    }

    #[test]
    fn test_load_all_missing_file() {
        let dir = TmpDir::new("load_all_missing");
        let store = Store::open(dir.path());
        let commits: Vec<Commit> = store.load_all("journal.jsonl").unwrap();
        assert!(commits.is_empty());
    }

    #[test]
    fn test_find_root_from() {
        let base = TmpDir::new("find_root");
        let root = base.path().join("root");
        fs::create_dir_all(root.join(".tataki")).unwrap();
        let nested = root.join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        assert_eq!(find_root_from(&nested), Some(root.clone()));
        assert_eq!(find_root_from(&root), Some(root));

        let elsewhere = base.path().join("elsewhere");
        fs::create_dir_all(&elsewhere).unwrap();
        assert_eq!(find_root_from(&elsewhere), None);
    }
}
