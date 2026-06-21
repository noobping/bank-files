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
        Ok(Some(release)) => start_download_if_possible(app, parent, mode, release),
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

fn start_download_if_possible(
    app: &Application,
    parent: Option<&ApplicationWindow>,
    mode: CheckMode,
    release: SelectedRelease,
) {
    if let Some(parent) = parent.cloned().or_else(|| active_window(app)) {
        start_download_and_install(app, parent, release);
        return;
    }

    let message = "No application window is available for the update download.";
    if mode == CheckMode::Manual {
        show_alert(parent, "Couldn't download the update.", message);
    } else {
        eprintln!("Automatic update download skipped: {message}");
    }
}

fn start_download_and_install(
    app: &Application,
    parent: ApplicationWindow,
    release: SelectedRelease,
) {
    let (dialog, progress, status, cancel_button) = build_download_dialog(&release);
    dialog.present(Some(&parent));

    let canceled = Arc::new(AtomicBool::new(false));
    let canceled_for_button = Arc::clone(&canceled);
    let dialog_for_cancel = dialog.clone();
    let status_for_cancel = status.clone();
    cancel_button.connect_clicked(move |button| {
        canceled_for_button.store(true, Ordering::Relaxed);
        button.set_sensitive(false);
        status_for_cancel.set_label(&gettext("Canceling download..."));
        dialog_for_cancel.close();
    });

    let canceled_for_closed = Arc::clone(&canceled);
    dialog.connect_closed(move |_| {
        canceled_for_closed.store(true, Ordering::Relaxed);
    });

    let (tx, rx) = mpsc::channel();
    let release_for_worker = release.clone();
    let canceled_for_worker = Arc::clone(&canceled);
    let spawn = std::thread::Builder::new()
        .name("bank-files-updater-download".to_string())
        .spawn(move || {
            let result = download_release(&release_for_worker, &tx, canceled_for_worker.as_ref());
            let _ = tx.send(DownloadMessage::Finished(result));
        });

    if let Err(error) = spawn {
        canceled.store(true, Ordering::Relaxed);
        dialog.force_close();
        show_alert(
            Some(&parent),
            "Couldn't download the update.",
            &format!("Failed to start the update download: {error}"),
        );
        return;
    }

    let app = app.clone();
    let canceled_for_ui = Arc::clone(&canceled);
    glib::timeout_add_local(Duration::from_millis(WORKER_POLL_INTERVAL_MS), move || {
        loop {
            match rx.try_recv() {
                Ok(DownloadMessage::Progress { downloaded, total }) => {
                    if canceled_for_ui.load(Ordering::Relaxed) {
                        return glib::ControlFlow::Break;
                    }
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
                Ok(DownloadMessage::Finished(Ok(DownloadOutcome::Completed(download)))) => {
                    if canceled_for_ui.load(Ordering::Relaxed) {
                        return glib::ControlFlow::Break;
                    }
                    dialog.force_close();
                    confirm_install(&app, &parent, download);
                    return glib::ControlFlow::Break;
                }
                Ok(DownloadMessage::Finished(Ok(DownloadOutcome::Canceled))) => {
                    dialog.force_close();
                    return glib::ControlFlow::Break;
                }
                Ok(DownloadMessage::Finished(Err(error))) => {
                    dialog.force_close();
                    if !canceled_for_ui.load(Ordering::Relaxed) {
                        show_alert(Some(&parent), "Couldn't download the update.", &error);
                    }
                    return glib::ControlFlow::Break;
                }
                Err(TryRecvError::Empty) if canceled_for_ui.load(Ordering::Relaxed) => {
                    return glib::ControlFlow::Break;
                }
                Err(TryRecvError::Empty) => return glib::ControlFlow::Continue,
                Err(TryRecvError::Disconnected) => return glib::ControlFlow::Break,
            }
        }
    });
}
