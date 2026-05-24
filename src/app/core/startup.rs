use super::build::build_ui_with_startup_request;
use super::session::ACTIVE_SESSION;
use super::*;

pub fn run() {
    crate::i18n::init();
    register_icon_resources();

    let pending_startup_request = Rc::new(RefCell::new(StartupRequest::default()));
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(
            gtk::gio::ApplicationFlags::HANDLES_OPEN
                | gtk::gio::ApplicationFlags::HANDLES_COMMAND_LINE,
        )
        .build();

    app.connect_startup(|_| {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default);
        ui::install_css();
        add_icon_resource_path();
    });
    let request_for_activate = Rc::clone(&pending_startup_request);
    app.connect_activate(move |app| {
        let request = {
            let mut pending = request_for_activate.borrow_mut();
            std::mem::take(&mut *pending)
        };
        build_ui_with_startup_request(app, request);
    });
    let request_for_command_line = Rc::clone(&pending_startup_request);
    app.connect_command_line(move |app, command_line| {
        let request = startup_request_from_args(&command_line.arguments());
        if request.is_empty() {
            app.activate();
        } else if !apply_startup_request_to_active_session(request.clone()) {
            *request_for_command_line.borrow_mut() = request;
            app.activate();
        }
        0.into()
    });
    app.connect_open(open_files);
    app.connect_shutdown(updater::shutdown);
    app.run();
}

fn register_icon_resources() {
    if let Err(err) = crate::resources::register() {
        eprintln!("Failed to register icon resources: {err}");
    }
}

fn add_icon_resource_path() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    crate::resources::add_icon_theme_path(&display);
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct StartupRequest {
    pub(super) opened_uris: Vec<String>,
    pub(super) transaction_search: Option<String>,
}

impl StartupRequest {
    fn is_empty(&self) -> bool {
        self.opened_uris.is_empty()
            && self
                .transaction_search
                .as_ref()
                .map(|query| query.trim().is_empty())
                .unwrap_or(true)
    }
}

fn startup_request_from_args(args: &[std::ffi::OsString]) -> StartupRequest {
    let mut request = StartupRequest::default();
    let mut index = 1;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--transaction-search" {
            if let Some(query) = args.get(index + 1) {
                request.transaction_search = Some(query.to_string_lossy().trim().to_string());
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if arg == "--" {
            for file_arg in &args[index + 1..] {
                request.opened_uris.push(command_line_file_uri(file_arg));
            }
            break;
        }

        if arg.to_string_lossy().starts_with('-') {
            index += 1;
            continue;
        }

        request.opened_uris.push(command_line_file_uri(arg));
        index += 1;
    }

    request
}

fn command_line_file_uri(arg: &std::ffi::OsStr) -> String {
    gtk::gio::File::for_commandline_arg(arg).uri().to_string()
}

fn apply_startup_request_to_active_session(request: StartupRequest) -> bool {
    ACTIVE_SESSION.with(|active| {
        let Some(session) = active.borrow().clone() else {
            return false;
        };

        if !request.opened_uris.is_empty() {
            import_uris_into_session(
                request.opened_uris,
                Rc::clone(&session.state),
                Rc::clone(&session.ui),
            );
        }
        if let Some(query) = request.transaction_search {
            apply_transaction_search(&session.state, &session.ui, &query);
        }
        true
    })
}

pub(super) fn apply_transaction_search(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    query: &str,
) {
    let query = query.trim();
    if query.is_empty() {
        ui.window.present();
        return;
    }

    show_transaction_search(state, ui, query, TransactionFilter::from_query(query));
    ui.window.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_request_parses_transaction_search() {
        let args = vec![
            std::ffi::OsString::from("bank-files"),
            std::ffi::OsString::from("--transaction-search"),
            std::ffi::OsString::from("rent may"),
        ];

        assert_eq!(
            startup_request_from_args(&args),
            StartupRequest {
                opened_uris: Vec::new(),
                transaction_search: Some("rent may".to_string()),
            }
        );
    }

    #[test]
    fn startup_request_keeps_file_open_arguments() {
        let args = vec![
            std::ffi::OsString::from("bank-files"),
            std::ffi::OsString::from("--"),
            std::ffi::OsString::from("/tmp/bank.csv"),
        ];
        let request = startup_request_from_args(&args);

        assert_eq!(request.transaction_search, None);
        assert_eq!(request.opened_uris.len(), 1);
        assert!(request.opened_uris[0].ends_with("/tmp/bank.csv"));
    }
}
