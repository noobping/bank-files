use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct ResourceFileEntry {
    source: String,
}

impl ResourceFileEntry {
    fn source(source: String) -> Self {
        Self { source }
    }
}

fn main() {
    let data_dir = Path::new("data");
    let mut resource_files = Vec::new();
    collect_icon_assets(data_dir, data_dir, &mut resource_files);
    resource_files.sort();
    let resources_xml = write_resources_xml(&resource_files);

    glib_build_tools::compile_resources(
        &[data_dir],
        resources_xml
            .to_str()
            .expect("Generated resource XML path should be valid UTF-8"),
        "compiled.gresource",
    );

    #[cfg(target_os = "windows")]
    embed_windows_icon(data_dir);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/scalable/apps");
    println!("cargo:rerun-if-changed=data/symbolic/actions");
    println!("cargo:rerun-if-changed=data/symbolic/apps");
    println!("cargo:rerun-if-changed=data/bank-files.ico");
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

fn write_resources_xml(resource_files: &[ResourceFileEntry]) -> PathBuf {
    let mut xml = String::from("<gresources>\n");
    writeln!(xml, "\t<gresource prefix=\"{}\">", resource_id())
        .expect("Failed to format resource prefix");
    for file in resource_files {
        writeln!(xml, "\t\t<file>{}</file>", file.source).expect("Failed to format resource entry");
    }
    xml.push_str("\t</gresource>\n</gresources>\n");
    let path = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR should be set by Cargo"))
        .join("resources.xml");
    write_if_changed(&path, &xml);
    path
}

fn write_if_changed(path: &Path, contents: &str) {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return;
    }

    fs::write(path, contents)
        .unwrap_or_else(|err| panic!("Failed to write {}: {err}", path.display()));
}

fn collect_icon_assets(dir: &Path, data_dir: &Path, resource_files: &mut Vec<ResourceFileEntry>) {
    for entry in fs::read_dir(dir).expect("Failed to read resource directory") {
        let entry = entry.expect("Failed to read resource directory entry");
        let path = entry.path();

        if path.is_dir() {
            collect_icon_assets(&path, data_dir, resource_files);
            continue;
        }

        if !resource_icon_file(&path) {
            continue;
        }

        let rel = path
            .strip_prefix(data_dir)
            .expect("Resource path should stay within data/");
        resource_files.push(ResourceFileEntry::source(path_to_resource(rel)));
    }
}

fn resource_icon_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("png" | "svg")
    ) && path.components().any(|component| {
        let value = component.as_os_str();
        value == "actions" || value == "apps"
    })
}

fn path_to_resource(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

const fn resource_id() -> &'static str {
    "/io/github/noobping/BankFiles"
}
