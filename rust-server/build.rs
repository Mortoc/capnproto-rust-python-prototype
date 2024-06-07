extern crate capnpc;
extern crate walkdir;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let proto_dir = Path::new("../protos");
    let out_dir = Path::new("./protos");

    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir).expect("create out_dir");
    }

    let mut command = capnpc::CompilerCommand::new();
    command.src_prefix(proto_dir);
    command.output_path(out_dir);

    let mod_file_path = Path::join(std::env::current_dir().unwrap().as_path(), "protos/mod.rs");
    println!("mod: {:?}", mod_file_path);
    let mut prelude_file = File::create(mod_file_path).expect("create prelude.rs");

    for entry in WalkDir::new(proto_dir) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension() == Some(OsStr::new("capnp")) {
            println!("cargo:rerun-if-changed={}", path.display());

            let relative_path = path.strip_prefix(proto_dir).expect("Getting relative path");
            let module_name = relative_path.file_stem().unwrap().to_str().unwrap();
            writeln!(prelude_file, "pub mod {}_capnp;", module_name).expect("write to prelude.rs");

            command.file(entry.path());
        }
    }

    command.run().expect("compiling capnp schema");
}
