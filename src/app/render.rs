use super::*;

const PAGE_RENDER_DELAY_MS: u64 = 250;
const RENDER_REQUEST_DEBOUNCE_MS: u64 = 220;

pub(in crate::app) fn request_render_views(ui: &Rc<UiHandles>, state: &Rc<RefCell<AppData>>) {
    let request = ui.render_request_generation.get().wrapping_add(1);
    ui.render_request_generation.set(request);
    cancel_current_page_render(ui.as_ref());

    let ui = Rc::clone(ui);
    let state = Rc::clone(state);
    gtk::glib::timeout_add_local_once(
        std::time::Duration::from_millis(RENDER_REQUEST_DEBOUNCE_MS),
        move || {
            if ui.render_request_generation.get() == request {
                render_views(&state.borrow(), &ui, &state);
            }
        },
    );
}

pub(in crate::app) fn render_views(
    data: &AppData,
    ui: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    ui.render_request_generation
        .set(ui.render_request_generation.get().wrapping_add(1));
    refresh_menu(ui.as_ref(), data);
    let fake_transactions = ui.fake_transactions.list();
    let scope_data = data_with_fake_transactions(data.clone(), fake_transactions);
    let desired_scope = current_transaction_load_scope(&scope_data, ui.as_ref());
    let generation = ui.render_generation.get().wrapping_add(1);
    ui.render_generation.set(generation);
    ui.render_loading_generation.set(None);

    match scope_render_action(data.loaded_scope, desired_scope, ui.loading_count.get()) {
        ScopeRenderAction::StartLoad => {
            render_loading_placeholder(ui.as_ref());
            reload_state_with_scope(
                state,
                ui,
                desired_scope,
                "Loading selected period...",
                tr("Selected period loaded."),
                "Load error: {error}",
                Vec::new(),
            );
            return;
        }
        ScopeRenderAction::WaitForLoad => {
            render_loading_placeholder(ui.as_ref());
            return;
        }
        ScopeRenderAction::Render => {}
    }

    let container_is_empty = current_page_container(ui.as_ref()).first_child().is_none();
    if container_is_empty {
        render_loading_placeholder(ui.as_ref());
    } else {
        ui.render_loading_generation.set(Some(generation));
        let ui_for_loading = Rc::clone(ui);
        gtk::glib::timeout_add_local_once(
            std::time::Duration::from_millis(PAGE_RENDER_DELAY_MS),
            move || {
                if ui_for_loading.render_loading_generation.get() == Some(generation) {
                    render_loading_placeholder(ui_for_loading.as_ref());
                }
            },
        );
    }

    prepare_visible_page_data(generation, Rc::clone(ui), Rc::clone(state));
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ScopeRenderAction {
    Render,
    StartLoad,
    WaitForLoad,
}

fn scope_render_action(
    loaded_scope: TransactionLoadScope,
    desired_scope: TransactionLoadScope,
    loading_count: u32,
) -> ScopeRenderAction {
    if loaded_scope.satisfies(desired_scope) {
        ScopeRenderAction::Render
    } else if loading_count == 0 {
        ScopeRenderAction::StartLoad
    } else {
        ScopeRenderAction::WaitForLoad
    }
}

fn cancel_current_page_render(ui: &UiHandles) {
    ui.render_generation
        .set(ui.render_generation.get().wrapping_add(1));
    ui.render_loading_generation.set(None);
}

fn prepare_visible_page_data(generation: u64, ui: Rc<UiHandles>, state: Rc<RefCell<AppData>>) {
    let page = current_page(ui.as_ref());
    let transaction_filter = ui.active_transaction_filter.borrow().clone();
    let fake_transactions = ui.fake_transactions.list();
    let data = state.borrow().clone();

    gtk::glib::MainContext::default().spawn_local(async move {
        let task = gtk::gio::spawn_blocking(move || {
            let data = data_with_fake_transactions(data, fake_transactions);
            prepare_page_data(data, page, transaction_filter)
        });

        match task.await {
            Ok(prepared) => {
                if ui.render_generation.get() != generation {
                    return;
                }
                ui.render_loading_generation.set(None);
                render_prepared_page(&prepared, &ui, &state);
            }
            Err(_) => {
                if ui.render_generation.get() == generation {
                    show_status(
                        &ui,
                        "Page loading canceled: the background task stopped unexpectedly.",
                    );
                }
            }
        }
    });
}

fn render_prepared_page(
    prepared: &PreparedPageData,
    ui: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    let page_data = prepared.visible.as_ref().unwrap_or(&prepared.data);
    match ui.stack.visible_child_name().as_deref() {
        Some("categories") => render_budget_page(page_data, ui, state),
        Some("transactions") => render_transactions_page(page_data, ui, state),
        Some("debug") => render_diagnostics_page(&prepared.data, ui, state),
        _ => render_overview(page_data, ui, state),
    }
}

pub(in crate::app) fn render_loading_placeholder(ui: &UiHandles) {
    let container = current_page_container(ui);
    ui::clear_box(container);

    let builder = ui::builder_from_resource("loading-placeholder.ui");
    let placeholder = ui::builder_object::<adw::StatusPage>(
        &builder,
        "loading_placeholder",
        "loading-placeholder.ui",
    );
    container.append(&placeholder);
}

fn current_page_container(ui: &UiHandles) -> &gtk::Box {
    match ui.stack.visible_child_name().as_deref() {
        Some("categories") => &ui.categories,
        Some("transactions") => &ui.transactions,
        Some("debug") => &ui.debug,
        _ => &ui.overview,
    }
}

#[derive(Debug, Clone)]
struct PreparedPageData {
    data: AppData,
    visible: Option<AppData>,
}

fn prepare_page_data(
    data: AppData,
    _page: AppPage,
    transaction_filter: Option<TransactionFilter>,
) -> PreparedPageData {
    let visible = page_data_for_render(&data, transaction_filter.as_ref());
    PreparedPageData { data, visible }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_render_action_waits_with_loading_placeholder_when_load_is_active() {
        assert_eq!(
            scope_render_action(
                TransactionLoadScope::Year(Some(2025)),
                TransactionLoadScope::All,
                1,
            ),
            ScopeRenderAction::WaitForLoad
        );
    }

    #[test]
    fn scope_render_action_starts_load_only_when_idle() {
        assert_eq!(
            scope_render_action(
                TransactionLoadScope::Year(Some(2025)),
                TransactionLoadScope::All,
                0,
            ),
            ScopeRenderAction::StartLoad
        );
    }

    #[test]
    fn scope_render_action_renders_when_scope_is_ready() {
        assert_eq!(
            scope_render_action(TransactionLoadScope::All, TransactionLoadScope::All, 1),
            ScopeRenderAction::Render
        );
    }

    #[test]
    fn scope_render_action_uses_scope_coverage() {
        assert_eq!(
            scope_render_action(
                TransactionLoadScope::All,
                TransactionLoadScope::Month(Some(MonthKey::new(2025, 5))),
                0,
            ),
            ScopeRenderAction::Render
        );
    }
}
