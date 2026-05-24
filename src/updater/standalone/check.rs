use super::*;

pub(in crate::updater::standalone) fn start_update_check(
    app: &Application,
    parent: Option<ApplicationWindow>,
    mode: CheckMode,
) {
    let (tx, rx) = mpsc::channel();
    let spawn = std::thread::Builder::new()
        .name("bank-files-updater-check".to_string())
        .spawn(move || {
            let _ = tx.send(fetch_update_release());
        });

    if let Err(error) = spawn {
        if mode == CheckMode::Manual {
            show_alert(
                parent.as_ref(),
                "Couldn't check for updates.",
                &format!("Failed to start the update checker: {error}"),
            );
        }
        return;
    }

    let app = app.clone();
    glib::timeout_add_local(
        Duration::from_millis(WORKER_POLL_INTERVAL_MS),
        move || match rx.try_recv() {
            Ok(result) => {
                handle_update_check_result(&app, parent.as_ref(), mode, result);
                glib::ControlFlow::Break
            }
            Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(TryRecvError::Disconnected) => glib::ControlFlow::Break,
        },
    );
}

fn handle_update_check_result(
    app: &Application,
    parent: Option<&ApplicationWindow>,
    mode: CheckMode,
    result: Result<Option<SelectedRelease>, String>,
) {
    match result {
        Ok(Some(release)) => show_update_available_dialog(app, parent, release),
        Ok(None) if mode == CheckMode::Manual => {
            show_alert(parent, "No update available", "Already up to date.");
        }
        Ok(None) => {}
        Err(error) if mode == CheckMode::Manual => {
            show_alert(parent, "Couldn't check for updates.", &error);
        }
        Err(error) => eprintln!("Automatic update check failed: {error}"),
    }
}

fn show_update_available_dialog(
    app: &Application,
    parent: Option<&ApplicationWindow>,
    release: SelectedRelease,
) {
    let heading = gettext("Update available");
    let body = crate::i18n::format(
        "Version {version} is available from GitHub Releases.",
        &[("version", release.version.to_string())],
    );
    let dialog = ui::alert_dialog(heading, body)
        .responses(&[
            ui::AlertResponse::neutral("later", "Later"),
            ui::AlertResponse::suggested("install", "Install Update"),
        ])
        .close_response("later")
        .default_response("install")
        .build();

    let app = app.clone();
    let parent = parent.cloned();
    let parent_for_callback = parent.clone();
    dialog.connect_response(None, move |_, response| {
        if response != "install" {
            return;
        }
        let Some(parent) = parent_for_callback.clone().or_else(|| active_window(&app)) else {
            return;
        };
        start_download_and_install(&app, parent, release.clone());
    });

    present_alert(&dialog, parent.as_ref());
}

fn start_download_and_install(
    app: &Application,
    parent: ApplicationWindow,
    release: SelectedRelease,
) {
    let (dialog, progress, status) = build_download_dialog(&release);
    dialog.present(Some(&parent));

    let (tx, rx) = mpsc::channel();
    let release_for_worker = release.clone();
    let spawn = std::thread::Builder::new()
        .name("bank-files-updater-download".to_string())
        .spawn(move || {
            let result = download_release(&release_for_worker, &tx);
            let _ = tx.send(DownloadMessage::Finished(result));
        });

    if let Err(error) = spawn {
        dialog.force_close();
        show_alert(
            Some(&parent),
            "Couldn't download the update.",
            &format!("Failed to start the update download: {error}"),
        );
        return;
    }

    let app = app.clone();
    glib::timeout_add_local(Duration::from_millis(WORKER_POLL_INTERVAL_MS), move || {
        loop {
            match rx.try_recv() {
                Ok(DownloadMessage::Progress { downloaded, total }) => {
                    let fraction = if total == 0 {
                        0.0
                    } else {
                        (downloaded as f64 / total as f64).clamp(0.0, 1.0)
                    };
                    progress.set_fraction(fraction);
                    status.set_label(&crate::i18n::format(
                        "Downloaded {downloaded} of {total}",
                        &[
                            ("downloaded", human_size(downloaded)),
                            ("total", human_size(total)),
                        ],
                    ));
                }
                Ok(DownloadMessage::Finished(Ok(download))) => {
                    dialog.force_close();
                    confirm_install(&app, &parent, download);
                    return glib::ControlFlow::Break;
                }
                Ok(DownloadMessage::Finished(Err(error))) => {
                    dialog.force_close();
                    show_alert(Some(&parent), "Couldn't download the update.", &error);
                    return glib::ControlFlow::Break;
                }
                Err(TryRecvError::Empty) => return glib::ControlFlow::Continue,
                Err(TryRecvError::Disconnected) => return glib::ControlFlow::Break,
            }
        }
    });
}
