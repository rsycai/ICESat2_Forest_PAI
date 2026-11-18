use crate::processing::config::Icesat2Config;
use crate::model::ProcessRequest;
use crate::output::metadata::json;
use crate::output::model::OutputBuilder;
use hdf5_pure::{AttrValue, FileBuilder};
use xt_error::Result;

pub(crate) fn builder(
    request: &ProcessRequest,
    config: &Icesat2Config,
    key: &str,
) -> Result<OutputBuilder> {
    let mut file = FileBuilder::new();
    let atl03 = file.create_group("ATL03");
    let atl08 = file.create_group("ATL08");
    let mut b = file.create_group("ATL03_08");
    b.set_attr("schema_version", AttrValue::U64(5));
    b.set_attr("tool", AttrValue::AsciiString("xt-icesat2".into()));
    b.set_attr("pair_key", AttrValue::AsciiString(key.into()));
    b.set_attr(
        "source_atl03",
        AttrValue::String(
            request
                .atl03
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
    );
    b.set_attr(
        "source_atl08",
        AttrValue::String(
            request
                .atl08
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
    );
    b.set_attr("filter_config", AttrValue::String(json(config)?));
    b.set_attr("index_base", AttrValue::U64(0));
    b.set_attr(
        "match_scope",
        AttrValue::AsciiString("ATL03/ATL08 photon intersection within ATL08 land_segments".into()),
    );
    b.set_attr(
        "selection_mode",
        AttrValue::AsciiString("matched_only".into()),
    );
    b.set_attr(
        "length_definition",
        AttrValue::AsciiString(
            "max-minus-min of valid along-track coordinates of all matched photons within each ATL08 land segment; NaN if none valid".into(),
        ),
    );
    Ok(OutputBuilder {
        file,
        atl03,
        atl08,
        matched: b,
        source03: request.atl03.clone(),
        source08: request.atl08.clone(),
    })
}
