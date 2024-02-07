/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::Path;

fn main() {
    // It would be better to generate this in /.generated or /target/godot-gen, however IDEs currently
    // struggle with static analysis when symbols are outside the crate directory (April 2023).
    let gen_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/gen"));

    if gen_path.exists() {
        // To handle some CI errors
        let mut retry_count = 0;

        loop {
           match std::fs::remove_dir_all(gen_path) {
            Ok(_) => break,
            Err(err) => {
                if retry_count >= 5 {
                    panic!("failed to delete dir: {err} in path: {}", gen_path.display());
                }
                retry_count += 1;
                std::thread::sleep(std::time::Duration::from_secs(retry_count));
            },
            }    
        }

        std::fs::remove_dir_all(gen_path).unwrap_or_else(|e| panic!("failed to delete dir: {e}"));
    }

    godot_codegen::generate_core_files(gen_path);
    println!("cargo:rerun-if-changed=build.rs");

    godot_bindings::emit_godot_version_cfg();
}
