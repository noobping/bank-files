use super::*;

pub(super) fn append_warnings_section(
    data: &AppData,
    search: Option<&SearchFilter>,
    ui_handles: &Rc<UiHandles>,
) -> bool {
    let warnings_section_search_matches = search.map(warning_section_matches).unwrap_or(false);
    let warnings = data
        .warnings
        .iter()
        .filter(|warning| {
            warnings_section_search_matches
                || search.map(|filter| filter.matches(warning)).unwrap_or(true)
        })
        .collect::<Vec<_>>();
    if warnings.is_empty() && !warnings_section_search_matches {
        return false;
    }

    ui_handles.debug.append(&ui::section_title(
        "Warnings",
        "You can select these messages or include them through Copy Page.",
    ));
    let warnings_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    if warnings.is_empty() {
        warnings_box.append(&ui::text_card(&tr("No warnings found.")));
    } else {
        for warning in warnings {
            warnings_box.append(&ui::text_card(warning));
        }
    }
    ui_handles.debug.append(&warnings_box);
    true
}

fn warning_section_matches(filter: &SearchFilter) -> bool {
    filter.matches("warnings warning alerts problems import checks")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warnings_preset_matches_warning_section() {
        let warning_search = SearchFilter::from_text("warnings").unwrap();
        let unrelated_search = SearchFilter::from_text("groceries").unwrap();

        assert!(warning_section_matches(&warning_search));
        assert!(!warning_section_matches(&unrelated_search));
    }
}
