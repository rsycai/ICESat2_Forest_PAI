mod catalog;
fn main() {
    let path = "../../resource/download/sources.xml";
    println!("cargo:rerun-if-changed={path}");
    let xml = std::fs::read_to_string(path).expect("Cannot read the default download XML");
    let public = catalog::without_credentials(&xml)
        .unwrap_or_else(|_| panic!("Cannot generate the credential-free embedded download XML"));
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is missing"));
    std::fs::write(output.join("sources.xml"), public).expect("Cannot write the embedded download XML");
}
