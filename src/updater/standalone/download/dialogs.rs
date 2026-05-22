use super::*;

pub(in crate::updater::standalone) fn build_download_dialog(
    release: &SelectedRelease,
) -> (Dialog, ProgressBar, Label) {
    let heading = Label::new(Some(&gettext("Downloading update")));
    heading.set_halign(Align::Start);
    heading.set_xalign(0.0);
    heading.set_wrap(true);
    heading.add_css_class("title-3");

    let body = Label::new(Some(&crate::i18n::format(
        "Downloading version {version}.",
        &[("version", release.version.to_string())],
    )));
    body.set_halign(Align::Start);
    body.set_xalign(0.0);
    body.set_wrap(true);
    body.add_css_class("dim-label");

    let progress = ProgressBar::new();
    progress.set_hexpand(true);

    let status = Label::new(Some(&gettext("Preparing download...")));
    status.set_halign(Align::Start);
    status.set_xalign(0.0);
    status.set_wrap(true);
    status.add_css_class("caption");
    status.add_css_class("dim-label");

    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_margin_top(18);
    content.set_margin_bottom(18);
    content.set_margin_start(18);
    content.set_margin_end(18);
    content.append(&heading);
    content.append(&body);
    content.append(&progress);
    content.append(&status);

    let dialog = Dialog::builder()
        .title(gettext("Bank Files Update"))
        .content_width(520)
        .follows_content_size(true)
        .child(&content)
        .build();

    (dialog, progress, status)
}

pub(in crate::updater::standalone) fn confirm_install(
    app: &Application,
    parent: &ApplicationWindow,
    download: DownloadedUpdate,
) {
    let dialog = AlertDialog::builder()
        .heading(gettext("Close Bank Files to install the update?"))
        .body(gettext(
            "Installing the update will close Bank Files. Unsaved changes will be lost.",
        ))
        .build();
    let cancel = gettext("Cancel");
    let install = gettext("Install Update");
    dialog.add_responses(&[("cancel", cancel.as_str()), ("install", install.as_str())]);
    dialog.set_close_response("cancel");
    dialog.set_default_response(Some("install"));

    let app = app.clone();
    let parent_for_response = parent.clone();
    dialog.connect_response(None, move |_, response| {
        if response != "install" {
            return;
        }

        match launch_update(&download) {
            Ok(()) => app.quit(),
            Err(error) => show_alert(
                Some(&parent_for_response),
                "Couldn't start the installer.",
                &error,
            ),
        }
    });
    dialog.present(Some(parent));
}

#[cfg(target_os = "windows")]
fn launch_update(download: &DownloadedUpdate) -> Result<(), String> {
    std::process::Command::new("msiexec")
        .arg("/i")
        .arg(&download.path)
        .arg("/norestart")
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Failed to start msiexec for update install: {error}"))
}

#[cfg(target_os = "linux")]
fn launch_update(download: &DownloadedUpdate) -> Result<(), String> {
    let mut perms = fs::metadata(&download.path)
        .map_err(|error| format!("Failed to read update file metadata: {error}"))?
        .permissions();
    perms.set_mode(UPDATE_EXECUTABLE_MODE);
    fs::set_permissions(&download.path, perms)
        .map_err(|error| format!("Failed to make the downloaded update executable: {error}"))?;

    let cleanup_dir = download
        .path
        .parent()
        .ok_or_else(|| "Linux update is missing its cleanup directory.".to_string())?;

    process::Command::new(&download.path)
        .arg(AUTO_INSTALL_ARG)
        .arg(cleanup_dir)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Failed to start Linux update install helper: {error}"))
}

pub(in crate::updater::standalone) fn show_alert(
    parent: Option<&ApplicationWindow>,
    heading: &str,
    body: &str,
) {
    let body = gettext(body);
    let dialog = AlertDialog::builder()
        .heading(gettext(heading))
        .body(body)
        .build();
    dialog.add_response("close", &gettext("Close"));
    dialog.set_close_response("close");
    dialog.set_default_response(Some("close"));
    present_alert(&dialog, parent);
}

pub(in crate::updater::standalone) fn present_alert(
    dialog: &AlertDialog,
    parent: Option<&ApplicationWindow>,
) {
    if let Some(parent) = parent {
        dialog.present(Some(parent));
    } else {
        dialog.present(None::<&ApplicationWindow>);
    }
}
