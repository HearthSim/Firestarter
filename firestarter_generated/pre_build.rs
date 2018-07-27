extern crate glob;
extern crate prost_build;

use glob::{glob, Paths};
use std::env;
use std::fs::{DirBuilder, File};
use std::io::{copy, BufReader, BufWriter, Write};
use std::path::PathBuf;

const ENV_CRATE_DIR: &'static str = "CARGO_MANIFEST_DIR";
const ENV_OUT_DIR: &'static str = "OUT_DIR";

fn main() {
    let compiled_proto_paths = build_protos();
    concat_protos(compiled_proto_paths);
}

fn build_protos() -> Paths {
    let build_dir = env::var_os(ENV_OUT_DIR).expect("Build directory wasn't set");

    let mut proto_source_dir = PathBuf::new();
    proto_source_dir.push(env::var(ENV_CRATE_DIR).expect("Crate directory wasn't set"));
    proto_source_dir.push("proto_schemas");
    let proto_source_dir = proto_source_dir.as_path(); // Immutable rebind

    println!(
        "Discovering proto schema files from {:}",
        proto_source_dir.display()
    );

    let proto_file_glob = format!("{:}/**/*.proto", proto_source_dir.display());
    let proto_paths: Vec<_> = glob(&proto_file_glob)
        .expect("Glob pattern error")
        // Each item is a Result<_>, because building a PathBuf type could fail.
        // We naively ignore errors here.
        .filter_map(Result::ok)
        .collect();
    let proto_path_strings: Vec<_> = proto_paths
        .iter()
        .map(|path| path.to_str())
        // Each item is an Option<_>, because building a PathBuf could fail if the path
        // contains non-utf8 characters.
        // We naively ignore errors here.
        .filter_map(|x| x)
        .collect();

    // Prost builder uses $OUT_DIR to place compiled proto schemas. We temporarily replace
    // this variable with a custom location.
    let mut proto_out_dir = PathBuf::new();
    proto_out_dir.push(build_dir.clone());
    proto_out_dir.push("proto_generated");
    let proto_out_dir = proto_out_dir.as_path(); // Immutable rebind
    DirBuilder::new()
        .recursive(true)
        .create(&proto_out_dir)
        .expect("Failed creating build directories");

    env::set_var(ENV_OUT_DIR, &proto_out_dir);
    let proto_source_dir_str = proto_source_dir
        .to_str()
        .expect("Directory name contains invalid utf8");
    prost_build::compile_protos(&proto_path_strings[..], &[proto_source_dir_str])
        .expect("Failed compiling proto schemas");
    env::set_var(ENV_OUT_DIR, &build_dir);
    // Only run this script again if the source directory with the proto schemas was updated
    println!("cargo:rerun-if-changed=\"{}\"", proto_out_dir.display());

    // TODO: Glob compiled proto schemas.
    let compiled_proto_file_glob = format!("{:}/**/*.rs", proto_out_dir.display());
    glob(&compiled_proto_file_glob).expect("Glob pattern error")
}

fn concat_protos(files: Paths) {
    let mut proto_paths: Vec<_> = files.filter_map(Result::ok).collect();
    proto_paths.sort_unstable_by(|a, b| b.cmp(a));

    let mut concat_file_path = PathBuf::new();
    concat_file_path.push(env::var_os(ENV_OUT_DIR).unwrap());
    concat_file_path.push("concat_compiled_proto.rs");
    let concat_file = File::create(concat_file_path).expect("Concat file creation error");
    let mut file_writer = BufWriter::new(concat_file);

    // Module-aware merge of multiple Rust source code files.
    //
    // The module name/structure is derived from the filename.
    //
    // Precondition 1: The input array MUST be sorted alphabetically (order doesn't matter).
    // Precondition 2: The source files CAN NOT define the same module accross multiple files,
    // this agrees with the constraints defined by Rust on the module declaration.

    let mut current_module: Vec<String> = vec![];
    let module_block_end = "}\n".as_bytes();
    let module_block_open = |mod_name| format!("pub mod {:} {{\n", mod_name).into_bytes();

    for file_path in &proto_paths {
        println!("Processing file {}", file_path.display());
        let file_handle = File::open(&file_path).expect("Failed opening compiled proto-file");
        let mut file_reader = BufReader::new(file_handle);
        let module_parts: Vec<_> = file_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split(".")
            .collect();

        let mut current_module_depth = 0;
        loop {
            let mut should_close_modules = false;
            let mut push_string: Option<String> = None;
            match (
                module_parts.get(current_module_depth),
                current_module.get(current_module_depth),
            ) {
                (Some(mod_part), None) => {
                    // Open new module
                    file_writer
                        .write(&module_block_open(*mod_part))
                        .expect("IO Write");
                    // We're holding an immutable borrow of 'current_module'.
                    // The immutable borrow blocks us from getting a mutable borrow.
                    // NLL fixes this, but isn't ready for stable yet.
                    //
                    // Workaround for:
                    // current_module.push((**mod_part).into());
                    push_string = Some(String::from(*mod_part));

                    // DO NOT increase current module depth so we automatically flow
                    // into the next match arm during the next loop!
                    // current_module_depth += 1;
                }
                (Some(mod_part), Some(cur_part)) if mod_part == cur_part => {
                    let equal_length = module_parts.len() == current_module.len();
                    let match_open_module = mod_part == module_parts.last().unwrap();

                    match (equal_length, match_open_module) {
                        (true, true) => {
                            // Write out module contents
                            copy(&mut file_reader, &mut file_writer).expect("IO Read+Write");
                            break;
                        },
                        (false, true) => {
                            // The current module situation is deeper than necessary for this file.
                            // We need to pop all next module parts from 'current_module'.
                            should_close_modules = true;
                            // Cut off at the next idx; [0, idx[ and [idx, len[
                            current_module_depth += 1;
                        },
                        (_, false) => current_module_depth += 1,
                    };
                }
                // Exclusive version where 'mod_part != cur_part' automatically.
                (Some(_), Some(_)) => {
                    // Close modules
                    // See above about NLL.
                    should_close_modules = true;
                }
                (None, None) => {
                    // We could have cut too far, so reduce the depth by one
                    current_module_depth -= 1;
                },
                (None, Some(_)) => unreachable!(),
            }

            // NLL workaround nr 1
            if let Some(string) = push_string {
                current_module.push(string);
            }

            // NLL workaround nr 2
            if should_close_modules == true {
                let closed_modules = current_module.split_off(current_module_depth);
                for _ in closed_modules {
                    file_writer.write(module_block_end).expect("IO Write");
                }
            }
        }
    }

    // The algorithm ends with some modules open, which we have to close now.
    let closed_modules = current_module;
    for _ in closed_modules {
        file_writer.write(module_block_end).expect("IO Write");
    }
}
