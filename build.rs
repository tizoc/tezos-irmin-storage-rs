// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use colored::*;
use os_type::{current_platform, OSType};

const LIBTEZOS_BUILD_NAME: &str = "libtezos-storage-ffi.so";
const ARTIFACTS_DIR: &str = "artifacts";

fn libtezos_filename() -> &'static str {
    let platform = current_platform();
    match platform.os_type {
        OSType::OSX => "libtezos-storage.dylib",
        _ => "libtezos-storage.so",
    }
}

fn run_builder(tezos_base_dir: &str) {
    let artifacts_path = Path::new(ARTIFACTS_DIR);
    let tezos_path = Path::new(&tezos_base_dir);
    let libtezos_storage_ffi_src_path = tezos_path.join(LIBTEZOS_BUILD_NAME);
    let libtezos_storage_ffi_dst_path = artifacts_path.join(libtezos_filename());

    if !tezos_path.exists() {
        println!(
            "{} TEZOS_BASE_DIR={} was not found!",
            "error".bright_red(),
            tezos_base_dir
        );
        panic!()
    }

    if !libtezos_storage_ffi_src_path.exists() {
        println!(
            "{} {} was not found!",
            "error".bright_red(),
            libtezos_storage_ffi_src_path.to_str().unwrap()
        );

        println!();
        println!(
            "Please build libtezos-ffi before continuing (see: ./tezos/interop/README.md)."
        );
        panic!();
    }

    Command::new("cp")
        .args(&[
            "-f",
            libtezos_storage_ffi_src_path.to_str().unwrap(),
            libtezos_storage_ffi_dst_path.to_str().unwrap(),
        ])
        .status()
        .expect("Couldn't copy libtezos-storage-ffi.");
}

fn main() {
    let tezos_base_dir = env::var("TEZOS_BASE_DIR").expect("TEZOS_BASE_DIR env variable is not defined");
    let out_dir = env::var("OUT_DIR").unwrap();

    fs::create_dir_all(ARTIFACTS_DIR).expect("Failed to create artifacts directory!");

    run_builder(&tezos_base_dir);

    let artifacts_dir_items = fs::read_dir(ARTIFACTS_DIR)
        .unwrap()
        .filter_map(Result::ok)
        .map(|dir_entry| dir_entry.path())
        .filter(|path| path.is_file())
        .collect::<Vec<PathBuf>>();
    let artifacts_dir_items: Vec<&Path> = artifacts_dir_items.iter().map(|p| p.as_path()).collect();
    let mut copy_options = fs_extra::dir::CopyOptions::new();
    copy_options.overwrite = true;
    let bytes_copied = fs_extra::copy_items(&artifacts_dir_items, &out_dir, &copy_options)
        .expect("Failed to copy artifacts to build output directory.");
    if bytes_copied == 0 {
        println!("cargo:warning=No files were found in the artifacts directory.");
        panic!("Failed to build tezos_interop artifacts.");
    }

    println!("cargo:rustc-link-search={}", &out_dir);
    println!("cargo:rustc-link-lib=dylib=tezos-storage");
    println!("cargo:rerun-if-env-changed=TEZOS_BASE_DIR");
}
