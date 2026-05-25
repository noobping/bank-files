use std::env;
use std::path::Path;

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

    #[cfg(target_os = "windows")]
    embed_windows_icon(data_dir);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/resources.xml");
    println!("cargo:rerun-if-changed=data/scalable/apps");
    println!("cargo:rerun-if-changed=data/symbolic/actions");
    println!("cargo:rerun-if-changed=data/symbolic/apps");
    println!("cargo:rerun-if-changed=data/ui");
    println!("cargo:rerun-if-changed=data/css");
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

#[cfg(target_os = "windows")]
fn embed_windows_icon(data_dir: &Path) {
    let ico_path = data_dir.join(format!("{}.ico", env!("CARGO_PKG_NAME")));
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(ico_path.to_string_lossy().as_ref());
    resource
        .compile()
        .expect("Failed to compile Windows icon resource");
}
