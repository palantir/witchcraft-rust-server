use std::fs::File;
use std::path::Path;

const API_VERSION: &str = "2.4.0";

fn main() {
    let logging_api_url = format!(
        "https://oss.sonatype.org/service/local/repositories/releases/content/com/palantir/witchcraft/api/witchcraft-logging-api/{0}/witchcraft-logging-api-{0}.conjure.json",
        API_VERSION,
    );
    render(
        &logging_api_url,
        "com.palantir.witchcraft.api.logging",
        "witchcraft-server/src/logging/api",
    );

    let health_api_url = format!(
        "https://oss.sonatype.org/service/local/repositories/releases/content/com/palantir/witchcraft/api/witchcraft-health-api/{0}/witchcraft-health-api-{0}.conjure.json",
        API_VERSION,
   );
    render(
        &health_api_url,
        "com.palantir.witchcraft.api.health",
        "witchcraft-server/src/health/api",
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
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(out_dir);

    conjure_codegen::Config::new()
        .exhaustive(true)
        .strip_prefix(prefix.to_string())
        .generate_files(ir_file, out_dir)
        .unwrap()
}
