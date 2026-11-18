use crate::model::{Cli, ExportFields};
use xt_error::Result;
use xt_manifest::{ManifestStore, model::DataEntry, store::completed_file};

pub(crate) fn run(args: Cli) -> Result<()> {
    let store = ManifestStore::<ExportFields>::new(args.xml_path);
    let file = completed_file(
        "DEMO",
        "xt-export",
        "export",
        &args.output,
        ExportFields {
            format: "csv".into(),
            rows: 1,
        },
    )?;
    store.upsert(
        DataEntry {
            id: "demo-scene".into(),
            paired: false,
            data_type: "DEMO".into(),
            date: "2020-01-01".into(),
            status: vec![xt_manifest::OperationStatus::new(
                "export",
                xt_manifest::TaskState::Completed,
                "DEMO",
            )],
            files: vec![],
        },
        file,
    )?;
    tracing::info!(path = %xt_manifest::display_path(store.path()), "Manifest created or updated");
    Ok(())
}
