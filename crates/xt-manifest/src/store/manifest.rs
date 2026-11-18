use crate::error::invalid;
use crate::model::{DataEntry, DataManifest, FileRecord};
use crate::store::lock::DayLock;
use crate::store::locking::lock_day;
use crate::store::paths::parent_dir;
use crate::value::XmlValue;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use xt_error::Result;

pub struct ManifestStore<T> {
    pub(super) path: PathBuf,
    pub(super) model: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> ManifestStore<T> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            model: PhantomData,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Option<DataManifest<XmlValue>>> {
        match fs::read_to_string(&self.path) {
            Ok(text) => quick_xml::de::from_str(&text)
                .map(Some)
                .map_err(|e| invalid(format!("Invalid XML manifest: {e}"))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn upsert(
        &self,
        mut data: DataEntry<T>,
        mut file: FileRecord<T>,
    ) -> Result<DataManifest<XmlValue>> {
        crate::store::validation::validate_file(&file)?;
        file.touch();
        for status in &mut data.status {
            if status.updated_at.is_none() {
                status.updated_at = file.updated_at.clone();
            }
        }
        data.files.push(file);
        self.update(data)
    }

    pub fn update(&self, data: DataEntry<T>) -> Result<DataManifest<XmlValue>> {
        let _lock = lock_day(self.day()?)?;
        let now = chrono::Utc::now();
        self.apply(
            DataManifest {
                run_id: format!(
                    "manifest-{}-{}",
                    now.format("%Y%m%dT%H%M%S%.fZ"),
                    std::process::id()
                ),
                generated_at: now.to_rfc3339(),
                data,
            },
            false,
        )
    }

    pub fn merge(&self, incoming: DataManifest<T>) -> Result<DataManifest<XmlValue>> {
        let lock = lock_day(self.day()?)?;
        self.merge_locked(&lock, incoming)
    }

    pub fn merge_locked(
        &self,
        lock: &DayLock,
        incoming: DataManifest<T>,
    ) -> Result<DataManifest<XmlValue>> {
        if self.day()? != lock.day {
            return Err(invalid("Manifest path is outside the locked directory"));
        }
        self.apply(incoming, true)
    }

    pub(super) fn load_or_create(
        &self,
        initial: DataManifest<XmlValue>,
    ) -> Result<DataManifest<XmlValue>> {
        match self.load()? {
            Some(record) if !record.data.same_identity(&initial.data) => {
                Err(invalid("Existing manifest data ID, date, or type mismatch"))
            }
            Some(record) => Ok(record),
            None => Ok(initial),
        }
    }

    pub(super) fn day(&self) -> Result<&Path> {
        let name = self
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| invalid("Invalid XML manifest filename"))?;
        if name.eq_ignore_ascii_case("index.xml") {
            return Err(invalid("Detail manifest must not use the daily index name index.xml"));
        }
        Ok(parent_dir(&self.path))
    }
}
