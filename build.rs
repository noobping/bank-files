use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    configure_linux_install_paths();

    let data_dir = Path::new("data");
    if !installed_resources_enabled() {
        glib_build_tools::compile_resources(
            &[data_dir],
            "data/resources.xml",
            "compiled.gresource",
        );
    }

    if env::var_os("CARGO_FEATURE_LOCAL_AI").is_some()
        && env::var_os("CARGO_FEATURE_EMBEDDED_AI_MODEL").is_none()
    {
        copy_local_ai_assets_to_profile(data_dir);
    }

    #[cfg(target_os = "windows")]
    embed_windows_icon(data_dir);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/resources.xml");
    println!("cargo:rerun-if-changed=data/scalable/apps");
    println!("cargo:rerun-if-changed=data/symbolic/actions");
    println!("cargo:rerun-if-changed=data/symbolic/apps");
    println!("cargo:rerun-if-changed=data/ui");
    println!("cargo:rerun-if-changed=data/ai");
    println!("cargo:rerun-if-changed=data/bank-files.ico");
}

fn configure_linux_install_paths() {
    println!("cargo:rustc-check-cfg=cfg(bank_files_installed_resources)");
    println!("cargo:rerun-if-env-changed=BANK_FILES_DATADIR");
    println!("cargo:rerun-if-env-changed=BANK_FILES_GRESOURCE");
    println!("cargo:rerun-if-env-changed=BANK_FILES_INSTALLED_RESOURCES");
    if let Some(datadir) = env::var_os("BANK_FILES_DATADIR") {
        println!(
            "cargo:rustc-env=BANK_FILES_DATADIR={}",
            datadir.to_string_lossy()
        );
    }
    if let Some(resource) = env::var_os("BANK_FILES_GRESOURCE") {
        println!(
            "cargo:rustc-env=BANK_FILES_GRESOURCE={}",
            resource.to_string_lossy()
        );
    }
    if installed_resources_enabled() {
        println!("cargo:rustc-cfg=bank_files_installed_resources");
    }
}

fn installed_resources_enabled() -> bool {
    env::var_os("BANK_FILES_INSTALLED_RESOURCES").is_some()
}

fn copy_local_ai_assets_to_profile(data_dir: &Path) {
    let source = data_dir.join("ai");
    if !source.is_dir() {
        panic!("Local AI feature enabled, but data/ai is missing");
    }
    let profile = cargo_profile_dir();
    let target = profile.join("models").join("ai");
    fs::create_dir_all(&target)
        .unwrap_or_else(|err| panic!("Failed to create {}: {err}", target.display()));
    copy_tree(&source, &target);
}

fn cargo_profile_dir() -> PathBuf {
    let mut path = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR should be set by Cargo"));
    for _ in 0..3 {
        path.pop();
    }
    path
}

fn copy_tree(source: &Path, target: &Path) {
    for entry in fs::read_dir(source)
        .unwrap_or_else(|err| panic!("Failed to read {}: {err}", source.display()))
    {
        let entry = entry.expect("Failed to read local AI asset entry");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            fs::create_dir_all(&target_path)
                .unwrap_or_else(|err| panic!("Failed to create {}: {err}", target_path.display()));
            copy_tree(&source_path, &target_path);
        } else {
            copy_if_changed(&source_path, &target_path);
        }
    }
}

fn copy_if_changed(source: &Path, target: &Path) {
    let source_bytes =
        fs::read(source).unwrap_or_else(|err| panic!("Failed to read {}: {err}", source.display()));
    if fs::read(target).ok().as_deref() == Some(source_bytes.as_slice()) {
        return;
    }
    fs::write(target, source_bytes)
        .unwrap_or_else(|err| panic!("Failed to write {}: {err}", target.display()));
}

#[cfg(target_os = "windows")]
fn embed_windows_icon(data_dir: &Path) {
    let ico_path = data_dir.join(format!("{}.ico", env!("CARGO_PKG_NAME")));
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(ico_path.to_string_lossy().as_ref());
    resource
        .compile()
        .expect("Failed to compile Windows icon resource");
}
