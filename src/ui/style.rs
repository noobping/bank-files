use super::*;

pub fn install_css() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        flowboxchild:hover,
        flowboxchild:selected,
        flowboxchild:active,
        flowboxchild:focus,
        flowboxchild:focus-visible {
            background: transparent;
            box-shadow: none;
            outline: none;
        }

        .action-card {
            transition: outline-color 120ms ease-out, opacity 120ms ease-out;
        }

        .action-card:hover {
            outline: 1px solid alpha(currentColor, 0.22);
            outline-offset: -1px;
        }

        .action-card:active {
            opacity: 0.88;
        }

        .warning-card {
            background: alpha(@warning_color, 0.12);
            box-shadow: inset 0 0 0 1px alpha(@warning_color, 0.55);
        }

        .warning-card .warning-title {
            color: @warning_color;
        }

        .status-page-actions button,
        .status-page-actions button:hover,
        .status-page-actions button:active,
        .status-page-actions button:checked {
            background: transparent;
            border-color: transparent;
            box-shadow: none;
            outline: none;
        }
        ",
    );
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
