use super::*;
use adw::glib::variant::{StaticVariantType, ToVariant};

pub(in crate::app) fn add_view_action(
    app: &adw::Application,
    ui: &Rc<UiHandles>,
    action_name: &str,
    page_name: &str,
) {
    let ui_for_action = Rc::clone(ui);
    let page_name = page_name.to_string();
    let action = gtk::gio::SimpleAction::new(action_name, None);
    action.connect_activate(move |_, _| {
        ui_for_action.stack.set_visible_child_name(&page_name);
    });
    app.add_action(&action);
}

fn set_simple_action_enabled(app: &adw::Application, name: &str, enabled: bool) {
    if let Some(action) = app
        .lookup_action(name)
        .and_then(|action| action.downcast::<gtk::gio::SimpleAction>().ok())
    {
        action.set_enabled(enabled);
    }
}

pub(in crate::app) fn connect_actions(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    import_button: gtk::Button,
    menu_button: gtk::MenuButton,
) {
    #[cfg(not(all(target_os = "linux", feature = "setup", not(feature = "flatpak"))))]
    let _ = menu_button;

    install_action_accelerators(app);

    let state_for_import = Rc::clone(state);
    let ui_for_import = Rc::clone(ui);
    let window_for_import = window.clone();
    let import_action = gtk::gio::SimpleAction::new("import-csv", None);
    import_action.connect_activate(move |action, _| {
        if !action.is_enabled() || ui_for_import.loading_count.get() > 0 {
            return;
        }
        action.set_enabled(false);
        show_status(&ui_for_import, "Opening the file portal for CSV files...");

        let action_for_import = action.clone();
        let state_for_import = Rc::clone(&state_for_import);
        let ui_for_import = Rc::clone(&ui_for_import);
        let window_for_import = window_for_import.clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            let handles = rfd::AsyncFileDialog::new()
                .set_title(tr("Choose one or more CSV files"))
                .add_filter("CSV", &["csv"])
                .pick_files()
                .await;

            let Some(handles) = handles.filter(|handles| !handles.is_empty()) else {
                action_for_import.set_enabled(true);
                show_status(&ui_for_import, "CSV import canceled.");
                return;
            };
            let files = handles
                .into_iter()
                .map(|handle| handle.path().to_path_buf())
                .collect::<Vec<_>>();

            show_status(&ui_for_import, "Opening CSV files...");
            open_paths_in_background(
                files,
                Rc::clone(&state_for_import),
                Rc::clone(&ui_for_import),
            )
            .await;
            action_for_import.set_enabled(true);
            window_for_import.present();
        });
    });
    app.add_action(&import_action);
    import_button.set_action_name(Some("app.import-csv"));

    let state_for_back = Rc::clone(state);
    let ui_for_back = Rc::clone(ui);
    let back_action = gtk::gio::SimpleAction::new("go-back", None);
    back_action.connect_activate(move |_, _| {
        navigate_back(&ui_for_back, &state_for_back);
    });
    app.add_action(&back_action);

    let state_for_reload = Rc::clone(state);
    let ui_for_reload = Rc::clone(ui);
    let reload_action = gtk::gio::SimpleAction::new("reload", None);
    reload_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        reload_state(&state_for_reload, &ui_for_reload);
    });
    app.add_action(&reload_action);

    let state_for_reload_all = Rc::clone(state);
    let ui_for_reload_all = Rc::clone(ui);
    let reload_all_action = gtk::gio::SimpleAction::new("reload-all", None);
    reload_all_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        reload_state_with_scope(
            &state_for_reload_all,
            &ui_for_reload_all,
            TransactionLoadScope::All,
            "Reloading all CSV data...",
            tr("All CSV data reloaded."),
            "Reload error: {error}",
            Vec::new(),
        );
    });
    app.add_action(&reload_all_action);

    let ui_for_find = Rc::clone(ui);
    let find_action = gtk::gio::SimpleAction::new("find", None);
    find_action.connect_activate(move |_, _| {
        ui_for_find.search_bar.set_search_mode(true);
        ui_for_find.search_entry.grab_focus();
        ui_for_find.search_entry.select_region(0, -1);
    });
    app.add_action(&find_action);

    let window_for_configuration = window.clone();
    let state_for_configuration = Rc::clone(state);
    let ui_for_configuration = Rc::clone(ui);
    let configuration_action = gtk::gio::SimpleAction::new("configuration", None);
    configuration_action.connect_activate(move |_, _| {
        show_configuration_dialog(
            &window_for_configuration,
            &state_for_configuration,
            &ui_for_configuration,
        );
    });
    app.add_action(&configuration_action);

    let window_for_preferences = window.clone();
    let ui_for_preferences = Rc::clone(ui);
    let preferences_action = gtk::gio::SimpleAction::new("preferences", None);
    let state_for_preferences = Rc::clone(state);
    preferences_action.connect_activate(move |_, _| {
        show_preferences_dialog(
            &window_for_preferences,
            &state_for_preferences,
            &ui_for_preferences,
        );
    });
    app.add_action(&preferences_action);

    add_view_action(app, ui, "view-overview", "overview");
    add_view_action(app, ui, "view-budget", "categories");
    add_view_action(app, ui, "view-transactions", "transactions");
    add_view_action(app, ui, "view-diagnostics", "debug");

    let state_for_copy_page = Rc::clone(state);
    let ui_for_copy_page = Rc::clone(ui);
    let copy_page_action = gtk::gio::SimpleAction::new("copy-page", None);
    copy_page_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        let snapshot = current_page_snapshot(&state_for_copy_page.borrow(), &ui_for_copy_page);
        ui_for_copy_page.window.clipboard().set_text(&snapshot.text);
        show_page_copy_feedback(&ui_for_copy_page);
    });
    app.add_action(&copy_page_action);

    let state_for_print_page = Rc::clone(state);
    let ui_for_print_page = Rc::clone(ui);
    let print_page_action = gtk::gio::SimpleAction::new("print-page", None);
    print_page_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        let report = current_print_report(&state_for_print_page.borrow(), &ui_for_print_page);
        print_report(&ui_for_print_page, report);
    });
    app.add_action(&print_page_action);

    let state_for_export = Rc::clone(state);
    let ui_for_export = Rc::clone(ui);
    let export_action = gtk::gio::SimpleAction::new("export-csv", None);
    export_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        let snapshot = current_real_page_snapshot(&state_for_export.borrow(), &ui_for_export);
        export_transactions_from_action(&ui_for_export, action, &snapshot.transactions);
    });
    app.add_action(&export_action);

    let manage_rules_action = gtk::gio::SimpleAction::new("manage-rules", None);
    let manage_budgets_action = gtk::gio::SimpleAction::new("manage-budgets", None);
    let manage_aliases_action = gtk::gio::SimpleAction::new("manage-aliases", None);
    manage_rules_action.set_enabled(ui.advanced_features.get());
    *ui.management_actions.borrow_mut() = vec![
        manage_rules_action.clone(),
        manage_budgets_action.clone(),
        manage_aliases_action.clone(),
    ];

    let state_for_rules = Rc::clone(state);
    let ui_for_rules = Rc::clone(ui);
    let window_for_rules = window.clone();
    manage_rules_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        show_management_dialog(&window_for_rules, &state_for_rules, &ui_for_rules, "rules");
    });
    app.add_action(&manage_rules_action);

    let state_for_budgets = Rc::clone(state);
    let ui_for_budgets = Rc::clone(ui);
    let window_for_budgets = window.clone();
    manage_budgets_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        show_management_dialog(
            &window_for_budgets,
            &state_for_budgets,
            &ui_for_budgets,
            "budgets",
        );
    });
    app.add_action(&manage_budgets_action);

    let state_for_aliases = Rc::clone(state);
    let ui_for_aliases = Rc::clone(ui);
    let window_for_aliases = window.clone();
    manage_aliases_action.connect_activate(move |action, _| {
        if !action.is_enabled() {
            return;
        }
        show_management_dialog(
            &window_for_aliases,
            &state_for_aliases,
            &ui_for_aliases,
            "aliases",
        );
    });
    app.add_action(&manage_aliases_action);

    let dedupe_action = gtk::gio::SimpleAction::new_stateful(
        "dedupe-enabled",
        None,
        &state.borrow().dedupe_mode.is_enabled().to_variant(),
    );
    dedupe_action.connect_activate(move |action, _| {
        let enabled = action
            .state()
            .and_then(|state| state.get::<bool>())
            .unwrap_or(true);
        action.change_state(&(!enabled).to_variant());
    });
    dedupe_action.set_enabled(ui.preferences.action_is_writable("dedupe-enabled"));

    let state_for_dedupe = Rc::clone(state);
    let ui_for_dedupe = Rc::clone(ui);
    dedupe_action.connect_change_state(move |action, value| {
        let Some(enabled) = value.and_then(|value| value.get::<bool>()) else {
            return;
        };
        set_dedupe_enabled(enabled, action.clone(), &state_for_dedupe, &ui_for_dedupe);
    });
    app.add_action(&dedupe_action);

    let state_for_remember_mode = Rc::clone(state);
    let ui_for_remember_mode = Rc::clone(ui);
    let remember_mode_action = gtk::gio::SimpleAction::new_stateful(
        "remember-mode",
        Some(&String::static_variant_type()),
        &ui.remember_mode.get().as_settings().to_variant(),
    );
    remember_mode_action.connect_activate(move |action, _| {
        let current = action
            .state()
            .and_then(|state| state.get::<String>())
            .map(|state| RememberMode::from_settings(&state))
            .unwrap_or_default();
        let next_index = RememberMode::SETTINGS_VALUES
            .iter()
            .position(|mode| *mode == current)
            .map(|index| (index + 1) % RememberMode::SETTINGS_VALUES.len())
            .unwrap_or(0);
        action.change_state(
            &RememberMode::SETTINGS_VALUES[next_index]
                .as_settings()
                .to_variant(),
        );
    });
    remember_mode_action.set_enabled(ui.preferences.action_is_writable("remember-mode"));
    remember_mode_action.connect_change_state(move |action, value| {
        let Some(value) = value.and_then(|value| value.get::<String>()) else {
            return;
        };
        let remember_mode = RememberMode::from_settings(&value);
        action.set_state(&remember_mode.as_settings().to_variant());
        set_remember_mode(
            remember_mode,
            &state_for_remember_mode,
            &ui_for_remember_mode,
        );
    });
    app.add_action(&remember_mode_action);

    let state_for_advanced_features = Rc::clone(state);
    let ui_for_advanced_features = Rc::clone(ui);
    let advanced_features_action = add_bool_toggle_action(
        app,
        "advanced-features",
        ui.advanced_features.get(),
        false,
        move |enabled| {
            ui_for_advanced_features.advanced_features.set(enabled);
            ui_for_advanced_features
                .preferences
                .set_advanced_features(enabled);
            refresh_write_actions(ui_for_advanced_features.as_ref());
            refresh_menu(
                &ui_for_advanced_features,
                &state_for_advanced_features.borrow(),
            );
            render_views(
                &state_for_advanced_features.borrow(),
                &ui_for_advanced_features,
                &state_for_advanced_features,
            );
            show_status(
                &ui_for_advanced_features,
                if enabled {
                    "Advanced Features enabled. Rule editing and direction controls are available."
                } else {
                    "Simple mode enabled. Rule editing and direction controls are hidden."
                },
            );
        },
    );
    advanced_features_action.set_enabled(ui.preferences.action_is_writable("advanced-features"));

    let state_for_show_all = Rc::clone(state);
    let ui_for_show_all = Rc::clone(ui);
    let show_all_action =
        add_bool_toggle_action(app, "show-all", ui.show_all.get(), false, move |enabled| {
            ui_for_show_all.show_all.set(enabled);
            ui_for_show_all.preferences.set_show_all(enabled);
            render_views(
                &state_for_show_all.borrow(),
                &ui_for_show_all,
                &state_for_show_all,
            );
            show_status(
                &ui_for_show_all,
                if enabled {
                    "Full lists enabled. More buttons are hidden."
                } else {
                    "Preview mode enabled. Sections show More buttons again."
                },
            );
        });
    show_all_action.set_enabled(ui.preferences.action_is_writable("show-all"));

    let state_for_predictions = Rc::clone(state);
    let ui_for_predictions = Rc::clone(ui);
    let app_for_predictions = app.clone();
    let show_predictions_action = add_bool_toggle_action(
        app,
        "show-predictions",
        ui.show_predictions.get(),
        false,
        move |enabled| {
            ui_for_predictions.show_predictions.set(enabled);
            ui_for_predictions.preferences.set_show_predictions(enabled);
            set_simple_action_enabled(
                &app_for_predictions,
                "hide-canceled-transactions",
                smart_pattern_detection_enabled(enabled)
                    && ui_for_predictions
                        .preferences
                        .action_is_writable("hide-canceled-transactions"),
            );
            if !enabled
                && matches!(
                    ui_for_predictions
                        .active_transaction_filter
                        .borrow()
                        .as_ref(),
                    Some(TransactionFilter::Pattern(_))
                )
            {
                *ui_for_predictions.active_transaction_filter.borrow_mut() = None;
            }
            render_views(
                &state_for_predictions.borrow(),
                &ui_for_predictions,
                &state_for_predictions,
            );
            refresh_menu(&ui_for_predictions, &state_for_predictions.borrow());
            show_status(
                &ui_for_predictions,
                if enabled {
                    "Smart Insights enabled. Forecast cards and transaction pattern detection are available."
                } else {
                    "Smart Insights disabled. Forecast cards and transaction pattern detection are hidden."
                },
            );
        },
    );
    show_predictions_action.set_enabled(ui.preferences.action_is_writable("show-predictions"));

    #[cfg(not(feature = "flatpak"))]
    {
        let ui_for_online_smart_insights = Rc::clone(ui);
        let online_smart_insights_action = add_bool_toggle_action(
            app,
            "online-smart-insights",
            ui.online_smart_insights.get(),
            false,
            move |enabled| {
                ui_for_online_smart_insights
                    .online_smart_insights
                    .set(enabled);
                ui_for_online_smart_insights
                    .preferences
                    .set_online_smart_insights(enabled);
                show_status(
                    &ui_for_online_smart_insights,
                    if enabled {
                        "Online Smart Insights enabled. Only privacy-filtered company labels may be used for online lookups."
                    } else {
                        "Online Smart Insights disabled. Smart Insights use local transactions only."
                    },
                );
            },
        );
        online_smart_insights_action
            .set_enabled(ui.preferences.action_is_writable("online-smart-insights"));
    }

    let state_for_compare_categories = Rc::clone(state);
    let ui_for_compare_categories = Rc::clone(ui);
    let compare_categories_action = add_bool_toggle_action(
        app,
        "compare-categories-previous-period",
        ui.compare_categories_previous_period.get(),
        true,
        move |enabled| {
            ui_for_compare_categories
                .compare_categories_previous_period
                .set(enabled);
            ui_for_compare_categories
                .preferences
                .set_compare_categories_previous_period(enabled);
            render_views(
                &state_for_compare_categories.borrow(),
                &ui_for_compare_categories,
                &state_for_compare_categories,
            );
            show_status(
                &ui_for_compare_categories,
                if enabled {
                    "Spending comparison enabled. Spending is compared with the previous period."
                } else {
                    "Spending comparison disabled. Spending comparisons are hidden."
                },
            );
        },
    );
    compare_categories_action.set_enabled(
        ui.preferences
            .action_is_writable("compare-categories-previous-period"),
    );

    let ui_for_advanced_autofill = Rc::clone(ui);
    let advanced_autofill_action = add_bool_toggle_action(
        app,
        "advanced-autofill",
        ui.advanced_autofill.get(),
        true,
        move |enabled| {
            ui_for_advanced_autofill.advanced_autofill.set(enabled);
            ui_for_advanced_autofill
                .preferences
                .set_advanced_autofill(enabled);
            show_status(
                &ui_for_advanced_autofill,
                if enabled {
                    "Smart Autofill enabled. Forms can fill matching categories and budget codes."
                } else {
                    "Smart Autofill disabled. Forms only use values you select."
                },
            );
        },
    );
    advanced_autofill_action.set_enabled(ui.preferences.action_is_writable("advanced-autofill"));

    let ui_for_auto_clean_config = Rc::clone(ui);
    let auto_clean_config_action = add_bool_toggle_action(
        app,
        "auto-clean-config",
        ui.auto_clean_config.get(),
        true,
        move |enabled| {
            ui_for_auto_clean_config.auto_clean_config.set(enabled);
            ui_for_auto_clean_config
                .preferences
                .set_auto_clean_config(enabled);
            show_status(
                &ui_for_auto_clean_config,
                if enabled {
                    "Auto Clean Config enabled. Orphaned rules are removed during reload and import."
                } else {
                    "Auto Clean Config disabled. Orphaned rules stay visible in Diagnostics."
                },
            );
        },
    );
    auto_clean_config_action.set_enabled(ui.preferences.action_is_writable("auto-clean-config"));

    let state_for_hide_canceled = Rc::clone(state);
    let ui_for_hide_canceled = Rc::clone(ui);
    let hide_canceled_action = add_bool_toggle_action(
        app,
        "hide-canceled-transactions",
        ui.hide_canceled_transactions.get(),
        false,
        move |enabled| {
            if !smart_pattern_detection_enabled(ui_for_hide_canceled.show_predictions.get()) {
                show_status(
                    &ui_for_hide_canceled,
                    "Hide Refunded Transactions needs Smart Insights. Enable Smart Insights to use refunded transaction hiding.",
                );
                return;
            }
            ui_for_hide_canceled.hide_canceled_transactions.set(enabled);
            ui_for_hide_canceled
                .preferences
                .set_hide_canceled_transactions(enabled);
            render_views(
                &state_for_hide_canceled.borrow(),
                &ui_for_hide_canceled,
                &state_for_hide_canceled,
            );
            show_status(
                &ui_for_hide_canceled,
                if enabled {
                    "Hide Refunded Transactions enabled. Detected refunds and offsetting groups are excluded from normal views."
                } else {
                    "Hide Refunded Transactions disabled. Detected refunds and offsetting groups are included again."
                },
            );
        },
    );
    hide_canceled_action.set_enabled(
        smart_pattern_detection_enabled(ui.show_predictions.get())
            && ui
                .preferences
                .action_is_writable("hide-canceled-transactions"),
    );

    #[cfg(all(target_os = "linux", feature = "setup", not(feature = "flatpak")))]
    {
        let ui_for_install = Rc::clone(ui);
        let menu_button_for_install = menu_button.clone();
        let state_for_install_menu = Rc::clone(state);
        let install_action = gtk::gio::SimpleAction::new("install-locally", None);
        install_action.set_enabled(setup::can_install_locally());
        install_action.connect_activate(move |action, _| {
            if !action.is_enabled() {
                return;
            }

            action.set_enabled(false);
            let installed = setup::is_installed_locally();
            let result = if installed {
                setup::uninstall_locally()
            } else {
                setup::install_locally()
            };

            match result {
                Ok(()) => {
                    let message = if installed {
                        "Removed from app menu."
                    } else {
                        "Added to app menu."
                    };
                    show_status(&ui_for_install, message);
                    let storage_capabilities = ui_for_install.storage_capabilities.borrow();
                    let menu = build_menu_model(
                        &state_for_install_menu.borrow(),
                        ui_for_install.advanced_features.get(),
                        &storage_capabilities,
                        &ui_for_install.preferences,
                    );
                    menu_button_for_install.set_menu_model(Some(&menu));
                }
                Err(_) => {
                    show_status(&ui_for_install, "Couldn't update the app menu.");
                }
            }

            action.set_enabled(setup::can_install_locally());
        });
        app.add_action(&install_action);
    }

    let window_for_shortcuts = window.clone();
    let ui_for_shortcuts = Rc::clone(ui);
    let shortcuts_action = gtk::gio::SimpleAction::new("shortcuts", None);
    shortcuts_action.connect_activate(move |_, _| {
        let shortcuts = build_shortcuts_dialog(
            ui_for_shortcuts.advanced_features.get(),
            smart_pattern_detection_enabled(ui_for_shortcuts.show_predictions.get()),
        );
        shortcuts.present(Some(&window_for_shortcuts));
    });
    app.add_action(&shortcuts_action);

    let window_for_about = window.clone();
    let about_action = gtk::gio::SimpleAction::new("about", None);
    about_action.connect_activate(move |_, _| {
        let app_name = app_info::display_name();
        let dialog = adw::AboutDialog::builder()
            .application_name(&app_name)
            .application_icon(APP_ID)
            .developer_name("Nick")
            .version(env!("CARGO_PKG_VERSION"))
            .comments(app_info::summary())
            .copyright("Copyright 2026 Nick")
            .website(env!("CARGO_PKG_HOMEPAGE"))
            .issue_url("https://github.com/noobping/bank-files/issues")
            .license_type(gtk::License::MitX11)
            .build();
        dialog.present(Some(&window_for_about));
    });
    app.add_action(&about_action);

    let app_for_quit = app.clone();
    let quit_action = gtk::gio::SimpleAction::new("quit", None);
    quit_action.connect_activate(move |_, _| {
        app_for_quit.quit();
    });
    app.add_action(&quit_action);

    updater::register_app_actions(app);
    refresh_write_actions(ui.as_ref());
    refresh_menu(ui, &state.borrow());
    connect_search(state, ui);
}

fn add_bool_toggle_action<F>(
    app: &adw::Application,
    name: &str,
    initial: bool,
    fallback: bool,
    on_change: F,
) -> gtk::gio::SimpleAction
where
    F: Fn(bool) + 'static,
{
    let action = gtk::gio::SimpleAction::new_stateful(name, None, &initial.to_variant());
    action.connect_activate(move |action, _| {
        let enabled = action
            .state()
            .and_then(|state| state.get::<bool>())
            .unwrap_or(fallback);
        action.change_state(&(!enabled).to_variant());
    });
    action.connect_change_state(move |action, value| {
        let Some(enabled) = value.and_then(|value| value.get::<bool>()) else {
            return;
        };
        on_change(enabled);
        action.set_state(&enabled.to_variant());
    });
    app.add_action(&action);
    action
}
