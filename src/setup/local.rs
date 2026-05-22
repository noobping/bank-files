use super::*;

pub fn local_menu_action_label(installed: bool) -> &'static str {
    if installed {
        "Remove from app menu"
    } else {
        "Add to app menu"
    }
}

pub fn can_install_locally() -> bool {
    let Some(bin) = dirs_next::executable_dir() else {
        return false;
    };
    let Some(data) = dirs_next::data_dir() else {
        return false;
    };

    can_install_into(&bin, &data)
}

pub fn is_installed_locally() -> bool {
    let Some(bin) = installed_local_binary_path() else {
        return false;
    };
    let Some(data) = dirs_next::data_dir() else {
        return false;
    };
    let desktop = data.join("applications").join(format!("{APP_ID}.desktop"));
    let search_provider = data
        .join("gnome-shell")
        .join("search-providers")
        .join(format!("{APP_ID}.search-provider.ini"));
    let service = data
        .join("dbus-1")
        .join("services")
        .join(format!("{SEARCH_PROVIDER_BUS_NAME}.service"));

    bin.exists()
        && bin.is_file()
        && desktop.exists()
        && desktop.is_file()
        && search_provider.exists()
        && search_provider.is_file()
        && service.exists()
        && service.is_file()
}

pub fn is_current_executable_installed_locally() -> bool {
    let Ok(current_exe) = std::env::current_exe() else {
        return false;
    };
    let Some(installed_exe) = installed_local_binary_path() else {
        return false;
    };

    same_file_path(&current_exe, &installed_exe)
}

pub fn install_locally() -> std::io::Result<()> {
    let exe_path = std::env::current_exe()?;
    let Some(bin) = dirs_next::executable_dir() else {
        return Err(Error::new(
            ErrorKind::NotFound,
            "No executable directory found",
        ));
    };
    let Some(data) = dirs_next::data_dir() else {
        return Err(Error::new(ErrorKind::NotFound, "No data directory found"));
    };

    let apps = data.join("applications");
    let services = data.join("dbus-1").join("services");
    let search_providers = data.join("gnome-shell").join("search-providers");
    let icons = data
        .join("icons")
        .join("hicolor")
        .join("scalable")
        .join("apps");
    let dest = bin.join(env!("CARGO_PKG_NAME"));

    if !can_install_into(&bin, &data) {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "One or more local install directories are not writable.",
        ));
    }

    fs::create_dir_all(&bin)?;
    fs::create_dir_all(&apps)?;
    fs::create_dir_all(&services)?;
    fs::create_dir_all(&search_providers)?;
    fs::create_dir_all(&icons)?;
    fs::copy(&exe_path, &dest)?;

    let mut perms = fs::metadata(&dest)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&dest, perms)?;

    write_desktop_file(&apps, &dest)?;
    write_search_provider_file(&search_providers)?;
    write_search_provider_service_file(&services, &dest)?;
    write_icon(&icons)?;
    Ok(())
}

pub fn uninstall_locally() -> std::io::Result<()> {
    let Some(bin) = dirs_next::executable_dir() else {
        return Err(Error::new(
            ErrorKind::NotFound,
            "No executable directory found",
        ));
    };
    let Some(data) = dirs_next::data_dir() else {
        return Err(Error::new(ErrorKind::NotFound, "No data directory found"));
    };

    let bin = bin.join(env!("CARGO_PKG_NAME"));
    let desktop = data.join("applications").join(format!("{APP_ID}.desktop"));
    let search_provider = data
        .join("gnome-shell")
        .join("search-providers")
        .join(format!("{APP_ID}.search-provider.ini"));
    let service = data
        .join("dbus-1")
        .join("services")
        .join(format!("{SEARCH_PROVIDER_BUS_NAME}.service"));
    let icon = data
        .join("icons")
        .join("hicolor")
        .join("scalable")
        .join("apps")
        .join(format!("{APP_ID}.svg"));

    if bin.exists() {
        fs::remove_file(bin)?;
    }
    if desktop.exists() {
        fs::remove_file(desktop)?;
    }
    if search_provider.exists() {
        fs::remove_file(search_provider)?;
    }
    if service.exists() {
        fs::remove_file(service)?;
    }
    if icon.exists() {
        fs::remove_file(icon)?;
    }

    Ok(())
}
