use super::warnings::{attention_warning_card_message, AttentionWarning};
use crate::ui;
use adw::gtk;
use adw::prelude::*;

pub(in crate::app) fn append_attention_warning_card(
    container: &gtk::Box,
    warnings: &[AttentionWarning],
) {
    if let Some(message) = attention_warning_card_message(warnings) {
        container.append(&ui::warning_card("Check your budget", &message));
    }
}
