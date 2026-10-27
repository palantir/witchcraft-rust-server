use std::fs::{self, File};
use std::path::Path;

const PPROF_COMMIT: &str = "dbd0ed30aa888a30e21ba83b9831ac0e80d41022";

fn main() {
    let url = format!(
        "https://raw.githubusercontent.com/google/pprof/{PPROF_COMMIT}/proto/profile.proto"
    );

    let dir = tempfile::tempdir().unwrap();
    let proto_file = dir.path().join("profile.proto");

    attohttpc::get(url)
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .write_to(File::create(&proto_file).unwrap())
        .unwrap();

    // protox compiles the schema in pure Rust, so protoc doesn't need to be installed.
    let descriptors = protox::compile([&proto_file], [dir.path()]).unwrap();
    prost_build::Config::new()
        .out_dir(dir.path())
        .compile_fds(descriptors)
        .unwrap();

    // ensure the path is relative to the workspace root
    let out_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../witchcraft-server/src/debug/cpu_profile/profile.rs");
    fs::copy(dir.path().join("perftools.profiles.rs"), out_file).unwrap();
}
