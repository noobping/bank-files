use super::*;

const DESKTOP_NAME_NL: &str = "Bankbestanden";
const DESKTOP_NAME_DE: &str = "Bankdateien";
const DESKTOP_GENERIC_NAME: &str = "Personal Finance Preview";
const DESKTOP_GENERIC_NAME_NL: &str = "Persoonlijke geldplanner";
const DESKTOP_GENERIC_NAME_DE: &str = "Persönliche Finanzvorschau";
const DESKTOP_COMMENT_NL: &str = "Verken geldkeuzes";
const DESKTOP_COMMENT_DE: &str = "Geldentscheidungen testen";
const DESKTOP_KEYWORDS: &str =
    "finance;budget;csv;excel;calc;spreadsheet;bank;money;transactions;preview;what-if;transfers;refunds;";
const DESKTOP_KEYWORDS_NL: &str =
    "financien;budget;csv;excel;calc;spreadsheet;bank;geld;transacties;vooruitblik;wat-als;overschrijvingen;terugbetalingen;";
const DESKTOP_KEYWORDS_DE: &str =
    "finanzen;budget;csv;excel;calc;tabellenkalkulation;bank;geld;transaktionen;vorschau;was-wenn;transfers;erstattungen;";

pub(super) fn installed_local_binary_path() -> Option<PathBuf> {
    dirs_next::executable_dir().map(|bin| bin.join(env!("CARGO_PKG_NAME")))
}

pub(super) fn can_install_into(bin: &Path, data: &Path) -> bool {
    [
        bin.to_path_buf(),
        data.join("applications"),
        data.join("dbus-1").join("services"),
        data.join("gnome-shell").join("search-providers"),
        data.join("icons")
            .join("hicolor")
            .join("scalable")
            .join("apps"),
    ]
    .iter()
    .all(|target| install_target_dir_is_eligible(target))
}

pub(super) fn is_writable(dir: &Path) -> bool {
    for attempt in 0..8u32 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let test_path = dir.join(format!(
            ".perm_test.{}.{}.{}",
            process::id(),
            nanos,
            attempt
        ));
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&test_path)
        {
            Ok(_) => {
                let _ = fs::remove_file(test_path);
                return true;
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
            Err(_) => return false,
        }
    }

    false
}

pub(super) fn install_target_dir_is_eligible(path: &Path) -> bool {
    let mut candidate = Some(path);
    while let Some(dir) = candidate {
        if dir.exists() {
            return dir.is_dir() && is_writable(dir);
        }
        candidate = dir.parent();
    }

    false
}

pub(super) fn same_file_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

pub(super) fn write_desktop_file(apps_path: &Path, bin_path: &Path) -> std::io::Result<()> {
    let comment = option_env!("CARGO_PKG_DESCRIPTION").unwrap_or(DESKTOP_GENERIC_NAME);
    let exec = bin_path.display();
    let contents = format!(
        "[Desktop Entry]
Type=Application
Version=1.0
Name={APP_NAME}
Name[nl]={DESKTOP_NAME_NL}
Name[de]={DESKTOP_NAME_DE}
GenericName={DESKTOP_GENERIC_NAME}
GenericName[nl]={DESKTOP_GENERIC_NAME_NL}
GenericName[de]={DESKTOP_GENERIC_NAME_DE}
Comment={comment}
Comment[nl]={DESKTOP_COMMENT_NL}
Comment[de]={DESKTOP_COMMENT_DE}
Exec={exec} %F
Icon={APP_ID}
Terminal=false
Categories=Office;Finance;
MimeType=text/csv;text/comma-separated-values;application/csv;application/vnd.ms-excel;application/vnd.openxmlformats-officedocument.spreadsheetml.sheet;application/vnd.ms-excel.sheet.macroEnabled.12;application/vnd.ms-excel.sheet.binary.macroEnabled.12;application/vnd.oasis.opendocument.spreadsheet;
Keywords={DESKTOP_KEYWORDS}
Keywords[nl]={DESKTOP_KEYWORDS_NL}
Keywords[de]={DESKTOP_KEYWORDS_DE}
StartupNotify=true
",
    );

    let file = apps_path.join(format!("{APP_ID}.desktop"));
    fs::write(&file, contents)?;
    let mut perms = fs::metadata(&file)?.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&file, perms)
}

pub(super) fn write_icon(icons_path: &Path) -> std::io::Result<()> {
    let resource_path = format!("{RESOURCE_ID}/scalable/apps/{APP_ID}.svg");
    let bytes =
        adw::gio::resources_lookup_data(&resource_path, adw::gio::ResourceLookupFlags::NONE)
            .map_err(|err| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Icon resource not found: {err}"),
                )
            })?;
    let icon = icons_path.join(format!("{APP_ID}.svg"));
    fs::write(&icon, bytes.as_ref())?;
    let mut perms = fs::metadata(&icon)?.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&icon, perms)
}

pub(super) fn write_search_provider_file(search_providers_path: &Path) -> std::io::Result<()> {
    let contents = format!(
        "[Shell Search Provider]
DesktopId={APP_ID}.desktop
BusName={SEARCH_PROVIDER_BUS_NAME}
ObjectPath={SEARCH_PROVIDER_OBJECT_PATH}
Version=2
",
    );

    let file = search_providers_path.join(format!("{APP_ID}.search-provider.ini"));
    fs::write(&file, contents)?;
    let mut perms = fs::metadata(&file)?.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&file, perms)
}

pub(super) fn write_search_provider_service_file(
    services_path: &Path,
    bin_path: &Path,
) -> std::io::Result<()> {
    let exec = bin_path.display();
    let contents = format!(
        "[D-BUS Service]
Name={SEARCH_PROVIDER_BUS_NAME}
Exec={exec} --search-provider
",
    );

    let file = services_path.join(format!("{SEARCH_PROVIDER_BUS_NAME}.service"));
    fs::write(&file, contents)?;
    let mut perms = fs::metadata(&file)?.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&file, perms)
}
