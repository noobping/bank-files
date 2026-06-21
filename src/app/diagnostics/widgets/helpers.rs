use super::*;

pub(in crate::app) fn delimiter_label(delimiter: char) -> String {
    match delimiter {
        ';' => tr("Semicolon delimiter (;)"),
        ',' => tr("Comma delimiter (,)"),
        '\t' => tr("Tab delimiter"),
        other => trf(
            "Delimiter {delimiter}",
            &[("delimiter", format!("{other:?}"))],
        ),
    }
}

pub(in crate::app) fn diagnostic_error_text(errors: usize) -> String {
    match errors {
        0 => tr("no sample errors"),
        1 => tr("1 sample error"),
        count => trf("{count} sample errors", &[("count", count.to_string())]),
    }
}

pub(in crate::app) fn empty_page(
    icon_name: &str,
    title: &str,
    description: &str,
) -> adw::StatusPage {
    adw::StatusPage::builder()
        .icon_name(icon_name)
        .title(tr(title))
        .description(tr(description))
        .build()
}
