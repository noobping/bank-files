use super::*;

pub(in crate::app) fn print_report(ui: &Rc<UiHandles>, report: PrintReport) {
    let operation = gtk::PrintOperation::new();
    let page_setup = minimal_print_page_setup();
    operation.set_default_page_setup(Some(&page_setup));
    operation.set_job_name(&format!("{} - {}", app_info::display_name(), report.title));
    operation.set_allow_async(false);
    operation.set_show_progress(true);

    let pages: Rc<RefCell<Vec<PrintPage>>> = Rc::new(RefCell::new(Vec::new()));
    let pages_for_begin = Rc::clone(&pages);
    let report_for_begin = report.clone();
    operation.connect_begin_print(move |operation, context| {
        let report_pages =
            paginate_print_report(&report_for_begin, context.width(), context.height());
        operation.set_n_pages(report_pages.len().max(1) as i32);
        *pages_for_begin.borrow_mut() = report_pages;
    });

    let report_for_draw = report.clone();
    let pages_for_draw = Rc::clone(&pages);
    operation.connect_draw_page(move |_, context, page_nr| {
        draw_print_report_page(context, &report_for_draw, &pages_for_draw.borrow(), page_nr);
    });

    let ui_for_done = Rc::clone(ui);
    operation.connect_done(move |operation, result| match result {
        gtk::PrintOperationResult::Apply => {
            show_status(ui_for_done.as_ref(), "Print job started");
        }
        gtk::PrintOperationResult::Error => {
            let message = operation
                .error()
                .map(|err| trf("Printing failed: {error}", &[("error", err.to_string())]))
                .unwrap_or_else(|| tr("Printing failed"));
            show_status(ui_for_done.as_ref(), &message);
        }
        _ => {}
    });

    match operation.run(gtk::PrintOperationAction::PrintDialog, Some(&ui.window)) {
        Ok(gtk::PrintOperationResult::Cancel) => {}
        Ok(gtk::PrintOperationResult::Apply) | Ok(gtk::PrintOperationResult::InProgress) => {}
        Ok(gtk::PrintOperationResult::Error) => {
            show_status(ui.as_ref(), "Printing failed");
        }
        Ok(_) => {}
        Err(err) => {
            show_status(
                ui.as_ref(),
                &trf("Printing failed: {error}", &[("error", err.to_string())]),
            );
        }
    }
}

fn minimal_print_page_setup() -> gtk::PageSetup {
    const PAGE_MARGIN_MM: f64 = 4.0;

    let setup = gtk::PageSetup::new();
    setup.set_top_margin(PAGE_MARGIN_MM, gtk::Unit::Mm);
    setup.set_bottom_margin(PAGE_MARGIN_MM, gtk::Unit::Mm);
    setup.set_left_margin(PAGE_MARGIN_MM, gtk::Unit::Mm);
    setup.set_right_margin(PAGE_MARGIN_MM, gtk::Unit::Mm);
    setup
}

pub(in crate::app) fn paginate_print_report(
    report: &PrintReport,
    width: f64,
    height: f64,
) -> Vec<PrintPage> {
    let top = print_content_top();
    let bottom = print_content_bottom(height);
    let mut pages = vec![PrintPage {
        elements: Vec::new(),
    }];
    let mut y = top;

    if !report.metrics.is_empty() {
        push_print_element(
            &mut pages,
            &mut y,
            width,
            bottom,
            top,
            PrintElement::Metrics(report.metrics.clone()),
        );
    }

    for section in &report.sections {
        match section {
            PrintSection::Paragraph { title, body } => {
                push_print_element(
                    &mut pages,
                    &mut y,
                    width,
                    bottom,
                    top,
                    PrintElement::SectionTitle {
                        title: title.clone(),
                        subtitle: String::new(),
                    },
                );
                push_print_element(
                    &mut pages,
                    &mut y,
                    width,
                    bottom,
                    top,
                    PrintElement::Paragraph { body: body.clone() },
                );
            }
            PrintSection::Table {
                title,
                subtitle,
                columns,
                rows,
            } => {
                let (columns, rows) = compact_print_table(columns, rows);
                push_print_element(
                    &mut pages,
                    &mut y,
                    width,
                    bottom,
                    top,
                    PrintElement::SectionTitle {
                        title: title.clone(),
                        subtitle: subtitle.clone(),
                    },
                );
                let header = PrintElement::TableHeader {
                    columns: columns.clone(),
                };
                push_print_element(&mut pages, &mut y, width, bottom, top, header.clone());

                if rows.is_empty() {
                    push_print_element(
                        &mut pages,
                        &mut y,
                        width,
                        bottom,
                        top,
                        PrintElement::Paragraph {
                            body: tr("No data for this section."),
                        },
                    );
                }

                for (index, row) in rows.iter().enumerate() {
                    let row_element = PrintElement::TableRow {
                        columns: columns.clone(),
                        cells: row.clone(),
                        index,
                    };
                    if y + print_element_height(&row_element, width) > bottom {
                        pages.push(PrintPage {
                            elements: Vec::new(),
                        });
                        y = top;
                        push_print_element(
                            &mut pages,
                            &mut y,
                            width,
                            bottom,
                            top,
                            PrintElement::SectionTitle {
                                title: trf("Continued: {title}", &[("title", title.clone())]),
                                subtitle: subtitle.clone(),
                            },
                        );
                        push_print_element(&mut pages, &mut y, width, bottom, top, header.clone());
                    }
                    push_print_element(&mut pages, &mut y, width, bottom, top, row_element);
                }
            }
        }
    }

    pages
}

fn compact_print_table(
    columns: &[PrintColumn],
    rows: &[Vec<PrintCell>],
) -> (Vec<PrintColumn>, Vec<Vec<PrintCell>>) {
    if rows.is_empty() {
        return (columns.to_vec(), rows.to_vec());
    }

    let keep = columns
        .iter()
        .enumerate()
        .filter_map(|(index, _)| {
            rows.iter()
                .any(|row| {
                    row.get(index)
                        .map(|cell| !cell.text.trim().is_empty())
                        .unwrap_or(false)
                })
                .then_some(index)
        })
        .collect::<Vec<_>>();

    if keep.is_empty() || keep.len() == columns.len() {
        return (columns.to_vec(), rows.to_vec());
    }

    let columns = keep
        .iter()
        .map(|index| columns[*index].clone())
        .collect::<Vec<_>>();
    let rows = rows
        .iter()
        .map(|row| {
            keep.iter()
                .map(|index| {
                    row.get(*index).cloned().unwrap_or(PrintCell {
                        text: String::new(),
                        tone: PrintTone::Normal,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    (columns, rows)
}

pub(in crate::app) fn push_print_element(
    pages: &mut Vec<PrintPage>,
    y: &mut f64,
    width: f64,
    bottom: f64,
    top: f64,
    element: PrintElement,
) {
    let height = print_element_height(&element, width);
    let current_page_empty = pages
        .last()
        .map(|page| page.elements.is_empty())
        .unwrap_or(true);
    if *y + height > bottom && !current_page_empty {
        pages.push(PrintPage {
            elements: Vec::new(),
        });
        *y = top;
    }
    *y += height;
    if let Some(page) = pages.last_mut() {
        page.elements.push(element);
    }
}

#[cfg(test)]
#[path = "operation_tests.rs"]
mod tests;
