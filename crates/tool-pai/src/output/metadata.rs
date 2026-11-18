use hdf5_pure::{AttrValue, DatasetBuilder};

pub(super) fn apply(ds: &mut DatasetBuilder, field: &str) {
    let (long_name, units, description) = match field {
        "latitude" => (
            "latitude",
            "degrees",
            "Latitude of the center-most signal photon within each segment.",
        ),
        "longitude" => (
            "longitude",
            "degrees",
            "Longitude of the center-most signal photon within each segment.",
        ),
        "segment_id_beg" => (
            "begin_geolocation_segment_bin",
            "1",
            "Geolocation segment number of the first photon in the land segment.",
        ),
        "segment_id_end" => (
            "end_geolocation_segment_bin",
            "1",
            "Geolocation segment number of the last photon in the land segment.",
        ),
        "PAIe" => (
            "effective_plant_area_index",
            "m^2/m^2",
            "Effective plant area index effect within segments.",
        ),
        "CI" => ("clumping_index", "", "Clumping index within segments."),
        "QA" => (
            "quality_assurance",
            "",
            "Quality gradually decreases from 0 to 9.",
        ),
        "gap_f" => ("gap_fraction", "", "Canopy gap fraction within segments."),
        "delta_time" => (
            "mean_pass_time",
            "seconds since 2018-01-01",
            "Mean time for the segment in number of GPS seconds since the ATLAS SDP epoch. The ATLAS Standard Data Products (SDP) epoch offset is defined within /ancillary_data/atlas_sdp_gps_epoch as the number of GPS seconds between the GPS epoch (1980-01-06T00:00:00.000000Z UTC) and the ATLAS SDP epoch. By adding the offset contained within atlas_sdp_gps_epoch to delta time parameters, the time in gps_seconds relative to the GPS epoch can be computed.",
        ),
        "PAI" => (
            "plant_area_index",
            "m^2/m^2",
            "The plant area index was estimated using the path length distribution method for indirect plant area index calculation.",
        ),
        "pai_status" => (
            "pai_calculation_status",
            "1",
            "0 valid; 1 no usable photons; 2 insufficient coverage or counts; 3 inversion failed",
        ),
        _ => unreachable!("Unknown PAI output column"),
    };
    let mut ascii = |key, value: &str| {
        ds.set_attr(key, AttrValue::VarLenAsciiString(value.into()));
    };
    ascii("long_name", long_name);
    ascii("units", units);
    ascii("description", description);
    match field {
        "latitude" => ascii("coordinates", "delta_time longitude"),
        "longitude" => ascii("coordinates", "delta_time latitude"),
        "segment_id_beg" | "segment_id_end" | "PAIe" | "CI" | "QA" | "gap_f" | "PAI"
        | "pai_status" => ascii("coordinates", "delta_time latitude longitude"),
        "delta_time" => ascii("coordinates", "latitude longitude"),
        _ => {}
    }
    match field {
        "latitude" | "longitude" => ascii("contentType", "modelResult"),
        "segment_id_beg" | "segment_id_end" | "PAIe" | "CI" | "QA" | "gap_f" | "delta_time"
        | "pai_status" => ascii("contentType", "referenceInformation"),
        _ => {}
    }
    match field {
        "latitude" => ascii("standard_name", "latitude"),
        "longitude" | "segment_id_beg" | "segment_id_end" => ascii("standard_name", "longitude"),
        "delta_time" => ascii("standard_name", "time"),
        _ => {}
    }
    match field {
        "segment_id_beg" | "segment_id_end" => ascii("source", "ATL08"),
        "delta_time" => ascii("source", "Land ATBD section 2.4"),
        _ => {}
    }
    ds.set_attr("_ChunkSizes", AttrValue::U32(10000));
    let limits = match field {
        "latitude" => Some((AttrValue::F32(-90.0), AttrValue::F32(90.0))),
        "longitude" => Some((AttrValue::F32(-180.0), AttrValue::F32(180.0))),
        "PAIe" | "PAI" => Some((AttrValue::F32(0.0), AttrValue::F32(200.0))),
        "QA" => Some((AttrValue::I8(0), AttrValue::I8(9))),
        "pai_status" => Some((AttrValue::U8(0), AttrValue::U8(3))),
        _ => None,
    };
    if let Some((min, max)) = limits {
        ds.set_attr("valid_min", min);
        ds.set_attr("valid_max", max);
    }
    match field {
        "PAIe" | "CI" | "gap_f" | "PAI" => {
            ds.set_attr("_FillValue", AttrValue::F32(crate::model::FILL));
        }
        "QA" => {
            ds.set_attr("_FillValue", AttrValue::I8(-128));
        }
        _ => {}
    }
}
