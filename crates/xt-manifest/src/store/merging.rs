use crate::XmlValue;
use crate::model::{DataManifest, OperationStatus};
use crate::store::codec::xml_model;
use crate::store::manifest::ManifestStore;
use crate::store::persistence::persist_record;
use crate::store::records::put_file;
use crate::store::status::put_status;
use crate::store::validation::{validate_file, validate_identity};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use xt_error::Result;

impl<T: Serialize + DeserializeOwned> ManifestStore<T> {
    pub(super) fn apply(
        &self,
        mut incoming: DataManifest<T>,
        replace_run: bool,
    ) -> Result<DataManifest<XmlValue>> {
        validate_identity(&incoming.data)?;
        let now = chrono::Utc::now().to_rfc3339();
        for file in &mut incoming.data.files {
            validate_file(file)?;
            if file.updated_at.is_none() {
                file.updated_at = Some(now.clone());
            }
        }
        for status in &mut incoming.data.status {
            if status.updated_at.is_none() {
                status.updated_at = Some(now.clone());
            }
        }
        let incoming: DataManifest<XmlValue> = xml_model(&incoming)?;
        let mut initial = incoming.clone();
        initial.data.files.clear();
        initial.data.status.clear();
        let mut record = self.load_or_create(initial)?;
        if replace_run {
            record.run_id = incoming.run_id;
            record.generated_at = incoming.generated_at;
        }
        let mut mapping = BTreeMap::<String, Vec<String>>::new();
        for file in incoming.data.files {
            let original = file.product.clone();
            let assigned = put_file(&mut record.data.files, file);
            mapping.entry(original).or_default().push(assigned);
        }
        for mut status in incoming.data.status {
            let mut targets = Vec::new();
            for target in status.target.split_whitespace() {
                if let Some(ids) = mapping.get(target) {
                    targets.extend(ids.iter().cloned());
                } else {
                    targets.push(target.to_owned());
                }
            }
            let normalized =
                OperationStatus::new(status.id.clone(), status.state, targets.join(" "));
            status.target = normalized.target;
            put_status(&mut record.data.status, status);
        }
        persist_record(&self.path, &record)?;
        Ok(record)
    }
}
