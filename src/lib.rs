//! In-memory key-value store
#![feature(seek_convenience)]
#![feature(bind_by_move_pattern_guards)]
#![deny(missing_docs)]

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate lru;
extern crate rmp_serde;

use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Custom error type
#[derive(Fail, Debug)]
pub enum KvError {
    /// I/O Error
    #[fail(display = "IO error")]
    IoError(#[cause] io::Error),
    /// Encode error
    #[fail(display = "Encode error")]
    EncodeError(#[cause] rmp_serde::encode::Error),
    /// Decode error
    #[fail(display = "Decode error")]
    DecodeError(#[cause] rmp_serde::decode::Error),
    /// Key not found error
    #[fail(display = "Key not found")]
    KeyNotFound,
    /// Unknown error
    #[fail(display = "Unknown error")]
    Unknown,
}

/// Alias for io results.
pub type Result<T> = std::result::Result<T, KvError>;

impl From<io::Error> for KvError {
    fn from(err: io::Error) -> KvError {
        KvError::IoError(err)
    }
}

impl From<rmp_serde::encode::Error> for KvError {
    fn from(err: rmp_serde::encode::Error) -> KvError {
        KvError::EncodeError(err)
    }
}

impl From<rmp_serde::decode::Error> for KvError {
    fn from(err: rmp_serde::decode::Error) -> KvError {
        KvError::DecodeError(err)
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type")]
enum LogEntry {
    Set { key: String, value: String },
    Remove { key: String },
}

/// Implements a KV store
pub struct KvStore {
    path: PathBuf,
    log: File,
    index: HashMap<String, u64>,
    cache: LruCache<String, String>,
    compaction_counter: u32,
}

impl KvStore {
    /// Opens an existing database
    pub fn open(path: &Path) -> Result<KvStore> {
        let path = if path.is_dir() {
            path.join("data.log")
        } else {
            path.to_path_buf()
        };

        let mut log = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path.as_path())?;

        let mut reader = io::BufReader::new(&mut log);
        let mut pointer = reader.stream_position()?;
        let mut index: HashMap<String, u64> = HashMap::new();

        while let Ok(entry) = rmp_serde::decode::from_read(&mut reader) {
            match entry {
                LogEntry::Remove { key } => index.remove(&key),
                LogEntry::Set { key, .. } => index.insert(key, pointer),
            };
            pointer = reader.stream_position()?;
        }

        Ok(KvStore {
            path,
            log,
            index,
            cache: LruCache::new(100),
            compaction_counter: 0,
        })
    }

    /// Retrieve the value for a key
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(value) = self.cache.get(&key) {
            return Ok(Some(value.to_string()));
        }

        if let Some(pointer) = self.index.get(&key) {
            let res = self.read_log_entry(*pointer)?;
            return Ok(res.map(|v| {
                self.cache.put(key, v.clone());
                v
            }));
        }

        Ok(None)
    }

    fn read_log_entry(&self, pointer: u64) -> Result<Option<String>> {
        let mut reader = io::BufReader::new(&self.log);
        reader.seek(SeekFrom::Start(pointer))?;
        let entry: LogEntry = rmp_serde::decode::from_read(&mut reader)?;
        match entry {
            LogEntry::Remove { .. } => Ok(None),
            LogEntry::Set { value, .. } => Ok(Some(value)),
        }
    }

    /// Set the value for a key
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        match self.get(key.clone()) {
            Ok(Some(v)) if v == value => Ok(()),
            _ => {
                let entry = LogEntry::Set {
                    key: key.clone(),
                    value: value.clone(),
                };
                let pointer = self.append_to_log(&entry)?;
                for _ in self.index.insert(key.clone(), pointer).iter() {
                    self.compact()?;
                }
                self.cache.put(key.clone(), value);
                Ok(())
            }
        }
    }

    /// Delete a key
    pub fn remove(&mut self, key: String) -> Result<()> {
        match self.index.remove(&key) {
            None => Err(KvError::KeyNotFound),
            Some(_) => {
                self.cache.pop(&key);
                let entry = LogEntry::Remove { key };
                self.append_to_log(&entry).map(|_| ())?;
                self.compact()
            }
        }
    }

    fn append_to_log(&mut self, entry: &LogEntry) -> Result<u64> {
        self.log.seek(SeekFrom::End(0))?;
        let pointer = self.log.stream_position()?;
        rmp_serde::encode::write(&mut self.log, entry)?;
        Ok(pointer)
    }

    fn compact(&mut self) -> Result<()> {
        self.compaction_counter += 1;
        if self.compaction_counter > 1000 {
            let old_path = self.path.as_path();
            let new_path = self.path.with_extension("bak");
            {
                let mut new_log = File::create(&new_path)?;
                let mut compactor = io::BufWriter::new(&mut new_log);
                for (key, ptr) in &self.index {
                    if let Some(value) = self.read_log_entry(*ptr)? {
                        let entry = LogEntry::Set {
                            key: key.to_string(),
                            value,
                        };
                        rmp_serde::encode::write(&mut compactor, &entry)?;
                    }
                }
            }

            std::mem::drop(&self.log);
            std::fs::rename(&new_path, &old_path)?;
            self.log = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&old_path)?;
            self.path = old_path.to_path_buf();
            self.compaction_counter = 0;
        }
        Ok(())
    }
}

/*
impl Drop for KvStore {
    fn drop(&mut self) {
        self.compact().unwrap();
    }
}
*/
