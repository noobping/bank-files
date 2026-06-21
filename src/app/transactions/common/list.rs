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
        let subtitle = markup_escape(&truncate(
            &transaction_subtitle(tx, ui_handles.advanced_features.get()),
            140,
        ));
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

    let details = transaction_details_grid(&transaction_detail_rows(
        tx,
        ui_handles.advanced_features.get(),
    ));
    content.append(&details);
    content.append(&transaction_detail_actions(tx, state, ui_handles));
    content
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app::transactions::common) struct TransactionDetailRow {
    pub(in crate::app::transactions::common) label: &'static str,
    pub(in crate::app::transactions::common) value: String,
}

pub(in crate::app::transactions::common) fn transaction_detail_rows(
    tx: &Transaction,
    advanced_features: bool,
) -> Vec<TransactionDetailRow> {
    let mut rows = Vec::new();
    push_detail_row(&mut rows, "Date", tx.date.to_string());
    push_detail_row(&mut rows, "Amount", signed_money(tx.amount));
    push_detail_row(&mut rows, "Counterparty", tx.counterparty.clone());
    if !same_detail_value(&tx.description, &tx.counterparty) {
        push_detail_row(&mut rows, "Description", tx.description.clone());
    }
    push_detail_row(&mut rows, "Tags", tx.tags.clone());
    push_detail_row(&mut rows, "Category", tx.category.clone());
    if advanced_features {
        push_detail_row(&mut rows, "Budget code", tx.budget_code.clone());
    }
    if crate::rules::transaction_classification_is_auto_detected(tx) {
        push_detail_row(&mut rows, "Classification", tr("Auto detected"));
    }
    if let Some(rule_match) = &tx.rule_match {
        push_detail_row(
            &mut rows,
            "Rule match",
            rule_match_summary(rule_match, advanced_features),
        );
    }
    if advanced_features {
        push_detail_row(&mut rows, "Account", tx.account.clone());
        push_detail_row(&mut rows, "Transaction ID", tx.transaction_id.clone());
        push_detail_row(&mut rows, "Currency", tx.currency.clone());
        push_detail_row(&mut rows, "Source file", tx.source_file.clone());
        push_detail_row(&mut rows, "Notes", tx.notes.clone());
    }
    rows
}

fn transaction_details_grid(rows: &[TransactionDetailRow]) -> gtk::Grid {
    let grid = ui::form_grid();
    for (index, row) in rows.iter().enumerate() {
        let value = ui::selectable_wrapped_label(&row.value);
        ui::add_labeled(&grid, index as i32, row.label, &value);
    }
    grid
}

fn push_detail_row(rows: &mut Vec<TransactionDetailRow>, label: &'static str, value: String) {
    let value = value.trim().to_string();
    if !value.is_empty() {
        rows.push(TransactionDetailRow { label, value });
    }
}

fn same_detail_value(left: &str, right: &str) -> bool {
    let left = crate::util::normalize_key(left);
    let right = crate::util::normalize_key(right);
    !left.is_empty() && left == right
}
