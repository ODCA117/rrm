use crate::rrm_error::RRMError;
use std::path::PathBuf;

use log::{trace, error};
use rusqlite::{Connection, Result};

pub struct FileDB {
    path: PathBuf,
    conn: Connection,
}

pub struct FileEntryDB {
    pub name: String,
    pub origin: String,
}

impl FileDB {
    /// Create new FileDB, if path exists, connect to existing DB.
    /// Otherwise, create a new Database.
    pub fn new(path: &PathBuf) -> Result<FileDB, RRMError> {
        // Connect or create DB table
        let conn = Connection::open(path).map_err(RRMError::DBConnection).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                 name TEXT PRIMARY KEY,
                 origin TEXT
             )",
            (),
        )?;

        Ok(FileDB {path: path.clone(), conn})
    }

    // TODO: Add command line options to add or remove entries to the DataBase
    /// Add file to database
    pub fn add(&self, file: FileEntryDB) -> Result<(), RRMError> {
        trace!("Add row to DB: {}", &file.name);
        self.conn.execute(
            "INSERT INTO files (name, origin) VALUES (?1, ?2)",
            (&file.name, &file.origin))?;
        Ok(())
    }

    /// Remove file from data base.
    pub fn remove(&self, name: &String) -> Result<(), RRMError> {
        trace!("remove row from DB: {}", name);
        let mut stmt = self.conn.prepare("DELETE FROM files WHERE name = (?1)")?;
        stmt.execute([name])?;
        Ok(())
    }

    pub fn get(&mut self, name: &String) -> Result<FileEntryDB, RRMError> {
        let mut stmt = self.conn.prepare("SELECT * FROM files WHERE name = ?1")?;
        let mut rows = stmt.query(rusqlite::params![name])?;
        let row = rows.next()?.ok_or(RRMError::FileNotFound(name.to_owned()))?;
        let fe = FileEntryDB {name: row.get(0)?, origin: row.get(1)?};
        trace!("Got file {}, origin {}", &fe.name, &fe.origin);
        Ok(fe)
    }

    pub fn get_all(&self) -> Result<Vec<FileEntryDB>, RRMError> {
        let mut file_entries: Vec<FileEntryDB> = Vec::new();
        let mut stmt = self.conn.prepare("SELECT * FROM files")?;
        let mut rows = stmt.query(())?;
        while let Ok(row) = rows.next() {
            match row {
                Some(row) => {
                    let fe = FileEntryDB{name: row.get(0)?, origin: row.get(1)?};
                    file_entries.push(fe);
                }
                None => break,
            }
        }
        Ok(file_entries)
    }

    /// Adapt database to the existing files.
    /// Will remove items not included.
    /// Will add items found, theses will have unknown origin.
    pub fn update(&mut self, files: Vec<FileEntryDB>) -> Result<(), RRMError> {
        todo!();
    }

    pub fn clear_db(&self) {
        match self.conn.execute("DELETE FROM files", ()) {
            Ok(n) => trace!("Cleared DB of {} rows", n),
            Err(_)  => error!("Failed to remove from db"),
        }
    }
}
