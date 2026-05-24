use super::*;

pub(in crate::app) fn transaction_pattern_card<F>(
    pattern: &analytics::TransactionPattern,
    badges: &[String],
    on_activate: F,
) -> gtk::Box
where
    F: Fn() + 'static,
{
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    card.set_margin_start(4);
    card.set_margin_end(4);
    card.set_hexpand(true);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.set_hexpand(true);

    let icon = gtk::Image::from_icon_name(transaction_pattern_icon(pattern.kind));
    icon.add_css_class("dim-label");
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);

    let title_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    title_box.set_hexpand(true);

    let title_label = pattern_label(&transaction_pattern_title(pattern));
    title_label.add_css_class("heading");
    title_box.append(&title_label);

    let label = pattern_label(&pattern.label);
    label.add_css_class("dim-label");
    title_box.append(&label);
    header.append(&title_box);

    let count = gtk::Label::new(Some(&pattern.count.to_string()));
    count.add_css_class("title-3");
    count.set_valign(gtk::Align::Start);
    count.set_tooltip_text(Some(&tr("Transactions")));
    header.append(&count);
    content.append(&header);

    if !badges.is_empty() {
        let badge_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        for badge in badges {
            let badge_label = gtk::Label::new(Some(badge));
            badge_label.add_css_class("caption");
            badge_label.add_css_class("dim-label");
            badge_label.set_xalign(0.0);
            badge_box.append(&badge_label);
        }
        content.append(&badge_box);
    }

    let period = trf(
        "{count} transactions total from {first} to {last}.",
        &[
            ("count", pattern.count.to_string()),
            ("first", pattern.first_date.to_string()),
            ("last", pattern.last_date.to_string()),
        ],
    );
    let period_label = pattern_label(&period);
    period_label.add_css_class("caption");
    content.append(&period_label);

    let values = pattern_label(&transaction_pattern_value_stats(pattern));
    values.add_css_class("caption");
    values.add_css_class("dim-label");
    content.append(&values);

    if matches!(
        pattern.kind,
        analytics::TransactionPatternKind::FullRefund
            | analytics::TransactionPatternKind::BillSplit
    ) {
        let net = pattern_label(&trf(
            "Net effect: {net}.",
            &[("net", signed_money(pattern.net))],
        ));
        net.add_css_class("caption");
        net.add_css_class(if pattern.net == rust_decimal::Decimal::ZERO {
            "dim-label"
        } else if pattern.net > Decimal::ZERO {
            "success"
        } else {
            "error"
        });
        content.append(&net);
    }

    card.append(&content);
    ui::activatable_card(card, on_activate)
}

fn pattern_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_width_chars(1);
    label.set_max_width_chars(72);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label
}

fn transaction_pattern_icon(kind: analytics::TransactionPatternKind) -> &'static str {
    match kind {
        analytics::TransactionPatternKind::Repeating(_) => "view-refresh-symbolic",
        analytics::TransactionPatternKind::FullRefund => "edit-undo-symbolic",
        analytics::TransactionPatternKind::BillSplit => "view-list-symbolic",
        analytics::TransactionPatternKind::Transfer => "folder-transfer-symbolic",
    }
}

fn transaction_pattern_value_stats(pattern: &analytics::TransactionPattern) -> String {
    let mut values = pattern
        .amount_stats
        .iter()
        .take(TRANSACTION_PATTERN_VALUE_PREVIEW_LIMIT)
        .map(|stat| {
            trf(
                "{count} times {amount}",
                &[
                    ("count", stat.count.to_string()),
                    ("amount", money(stat.amount)),
                ],
            )
        })
        .collect::<Vec<_>>();
    let hidden = pattern
        .amount_stats
        .len()
        .saturating_sub(TRANSACTION_PATTERN_VALUE_PREVIEW_LIMIT);
    if hidden > 0 {
        values.push(trf(
            "{count} more value groups",
            &[("count", hidden.to_string())],
        ));
    }
    trf("Values: {values}", &[("values", values.join(", "))])
}

pub(in crate::app) fn transaction_pattern_matches(
    pattern: &analytics::TransactionPattern,
    filter: &SearchFilter,
) -> bool {
    filter.matches(&format!(
        "{} {} {} {} {} {} {} {}",
        transaction_pattern_title(pattern),
        pattern.label,
        pattern.count,
        money(pattern.amount),
        pattern
            .amount_stats
            .iter()
            .map(|stat| format!("{} {}", stat.count, money(stat.amount)))
            .collect::<Vec<_>>()
            .join(" "),
        signed_money(pattern.net),
        pattern.first_date,
        pattern.last_date,
    ))
}

fn transaction_pattern_title(pattern: &analytics::TransactionPattern) -> String {
    match pattern.kind {
        analytics::TransactionPatternKind::Repeating(cadence) => trf(
            "Repeating {cadence} transaction",
            &[("cadence", transaction_pattern_cadence(cadence))],
        ),
        analytics::TransactionPatternKind::FullRefund => tr("Possible refund"),
        analytics::TransactionPatternKind::BillSplit => tr("Possible offsetting group"),
        analytics::TransactionPatternKind::Transfer => tr("Possible transfer"),
    }
}

fn transaction_pattern_cadence(cadence: analytics::RepeatingCadence) -> String {
    tr(match cadence {
        analytics::RepeatingCadence::Weekly => "weekly",
        analytics::RepeatingCadence::Biweekly => "every two weeks",
        analytics::RepeatingCadence::Monthly => "monthly",
        analytics::RepeatingCadence::Quarterly => "quarterly",
        analytics::RepeatingCadence::Yearly => "yearly",
        analytics::RepeatingCadence::Recurring => "recurring",
    })
}
