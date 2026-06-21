use super::*;

pub(in crate::updater::standalone) fn build_download_dialog(
    release: &SelectedRelease,
) -> (Dialog, ProgressBar, Label, Button) {
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

    let cancel_button = Button::with_label(&gettext("Cancel"));
    cancel_button.set_halign(Align::End);

    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_margin_top(18);
    content.set_margin_bottom(18);
    content.set_margin_start(18);
    content.set_margin_end(18);
    content.append(&heading);
    content.append(&body);
    content.append(&progress);
    content.append(&status);
    content.append(&cancel_button);

    let dialog = ui::content_dialog(gettext("Update"), &content)
        .content_width(520)
        .follows_content_size()
        .build();

    (dialog, progress, status, cancel_button)
}

pub(in crate::updater::standalone) fn confirm_install(
    app: &Application,
    parent: &ApplicationWindow,
    download: DownloadedUpdate,
) {
    let dialog = ui::alert_dialog(
        gettext("Close before installing the update?"),
        gettext("Unsaved changes will be lost."),
    )
    .responses(&[
        ui::AlertResponse::neutral("cancel", "Cancel"),
        ui::AlertResponse::suggested("install", "Install Update"),
    ])
    .close_response("cancel")
    .default_response("install")
    .build();

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
    let dialog = ui::alert_dialog(gettext(heading), gettext(body))
        .responses(&[ui::AlertResponse::neutral("close", "Close")])
        .close_response("close")
        .default_response("close")
        .build();
    present_alert(&dialog, parent);
}

pub(in crate::updater::standalone) fn present_alert(
    dialog: &AlertDialog,
    parent: Option<&ApplicationWindow>,
) {
    ui::present_alert_dialog(dialog, parent);
}
