//! Workspace-side linker fix: `mdbx-sys` 12.13.2 calls Windows registry and
//! privilege APIs (`RegCloseKey`, `OpenProcessToken`, ...) but its build
//! script only links `ntdll`/`user32`, leaving `advapi32` unresolved on
//! MSVC. Link it from every crate that consumes `libmdbx` directly.

fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
