//! Workspace-side linker fix: `mdbx-sys` 12.13.2 calls the Windows registry
//! API (`mdbx_RegGetValue` -> `RegCloseKey`/`RegOpenKeyA`/`RegQueryValueExA`)
//! but its build script only links `ntdll` and `user32`, leaving `advapi32`
//! unresolved on MSVC. Linking it here (any crate in the dependency graph
//! that links the test/binaries) resolves the symbols.

fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
