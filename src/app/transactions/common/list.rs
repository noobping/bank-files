use super::detail_actions::transaction_detail_actions;
use super::text::{markup_escape, transaction_subtitle, transaction_title};
use super::*;

pub(in crate::app) fn transaction_list(
    transactions: &[&Transaction],
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::ListBox {
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    for tx in transactions {
        let title = markup_escape(&truncate(&transaction_title(tx), 80));
        let subtitle = markup_escape(&truncate(&transaction_subtitle(tx), 140));
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();
        row.set_tooltip_text(Some(&tr("Click to show transaction details")));

        let direction_icon = gtk::Image::from_icon_name(if tx.amount >= Decimal::ZERO {
            "go-up-symbolic"
        } else {
            "go-down-symbolic"
        });
        direction_icon.add_css_class(if tx.amount >= Decimal::ZERO {
            "success"
        } else {
            "error"
        });
        row.add_prefix(&direction_icon);

        if crate::rules::transaction_classification_is_auto_detected(tx) {
            let badge = gtk::Label::new(Some(&tr("Auto detected")));
            badge.add_css_class("caption");
            badge.add_css_class("accent");
            badge.set_tooltip_text(Some(&tr(
                "This category and budget code were assigned by automatic detection.",
            )));
            row.add_suffix(&badge);
        }

        let amount = gtk::Label::new(Some(&signed_money(tx.amount)));
        amount.add_css_class(if tx.amount >= Decimal::ZERO {
            "success"
        } else {
            "error"
        });
        amount.set_xalign(1.0);
        row.add_suffix(&amount);

        let expand_icon = gtk::Image::from_icon_name("pan-down-symbolic");
        expand_icon.add_css_class("dim-label");
        row.add_suffix(&expand_icon);

        let revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideDown)
            .build();

        let click = gtk::GestureClick::new();
        click.set_button(0);
        let tx_for_details = (**tx).clone();
        let state_for_details = Rc::clone(state);
        let ui_for_details = Rc::clone(ui_handles);
        let revealer_for_click = revealer.clone();
        let expand_icon_for_click = expand_icon.clone();
        let details_built = std::rc::Rc::new(std::cell::Cell::new(false));
        let details_built_for_click = std::rc::Rc::clone(&details_built);
        click.connect_released(move |_, _, _, _| {
            let reveal = !revealer_for_click.reveals_child();
            if reveal && !details_built_for_click.get() {
                let details =
                    transaction_details_table(&tx_for_details, &state_for_details, &ui_for_details);
                revealer_for_click.set_child(Some(&details));
                details_built_for_click.set(true);
            }
            revealer_for_click.set_reveal_child(reveal);
            expand_icon_for_click.set_icon_name(Some(if reveal {
                "pan-up-symbolic"
            } else {
                "pan-down-symbolic"
            }));
        });
        row.add_controller(click);

        let item = gtk::Box::new(gtk::Orientation::Vertical, 0);
        item.append(&row);
        item.append(&revealer);
        list.append(&item);
    }
    list
}

fn transaction_details_table(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
) -> gtk::Box {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
    content.set_margin_top(0);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let details = gtk::Box::new(gtk::Orientation::Vertical, 8);
    details.set_hexpand(true);

    let mut rows = vec![
        ("Date", tx.date.to_string()),
        ("Amount", signed_money(tx.amount)),
        ("Counterparty", tx.counterparty.clone()),
        ("Description", tx.description.clone()),
        ("Tags", tx.tags.clone()),
        ("Category", tx.category.clone()),
    ];
    if ui_handles.advanced_features.get() {
        rows.push(("Budget code", tx.budget_code.clone()));
    }
    if crate::rules::transaction_classification_is_auto_detected(tx) {
        rows.push(("Classification", tr("Auto detected")));
    }
    rows.extend([
        ("Account", tx.account.clone()),
        ("Transaction ID", tx.transaction_id.clone()),
        ("Currency", tx.currency.clone()),
        ("Source file", tx.source_file.clone()),
        ("Notes", tx.notes.clone()),
    ]);

    for (label, value) in rows {
        details.append(&transaction_detail_row(label, &value));
    }

    content.append(&details);
    content.append(&transaction_detail_actions(tx, state, ui_handles));
    content
}

fn transaction_detail_row(label: &str, value: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 2);
    row.set_hexpand(true);

    let label = gtk::Label::new(Some(&tr(label)));
    label.add_css_class("caption");
    label.add_css_class("dim-label");
    label.set_xalign(0.0);
    row.append(&label);

    let value = if value.trim().is_empty() {
        tr("Not set")
    } else {
        value.trim().to_string()
    };
    let value = gtk::Label::new(Some(&value));
    value.set_xalign(0.0);
    value.set_hexpand(true);
    value.set_selectable(true);
    value.set_wrap(true);
    value.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    row.append(&value);

    row
}
