use std::fs::File;
use std::path::Path;

const API_VERSION: &str = "2.0.0";

fn main() {
    let logging_api_url = format!(
        "https://repo1.maven.org/maven2/com/palantir/witchcraft/api/witchcraft-logging-api/{0}/witchcraft-logging-api-{0}.conjure.json",
        API_VERSION,
   );
    render(
        &logging_api_url,
        "com.palantir.witchcraft.api.logging",
        "witchcraft-server/src/logging/api",
    );
}

fn render(url: &str, prefix: &str, out_dir: &str) {
    let dir = tempfile::tempdir().unwrap();
    let ir_file = dir.path().join("ir.json");

    attohttpc::get(url)
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .write_to(File::create(&ir_file).unwrap())
        .unwrap();

    // ensure the path is relative to the workspace root
    let out_dir = Path::new(file!()).join("../../../..").join(out_dir);

    conjure_codegen::Config::new()
        .staged_builders(true)
        .exhaustive(true)
        .strip_prefix(prefix.to_string())
        .generate_files(ir_file, out_dir)
        .unwrap()
}
