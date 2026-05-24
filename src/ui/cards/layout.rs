use super::*;

pub fn responsive_columns(sections: Vec<gtk::Box>) -> gtk::FlowBox {
    responsive_columns_with_limits(sections, 1, 3)
}

pub fn responsive_columns_three_or_one(sections: Vec<gtk::Box>) -> gtk::FlowBox {
    let max_columns = if sections.len() == 3 { 3 } else { 2 };
    responsive_columns_with_limits(sections, 1, max_columns)
}

pub fn responsive_chart_columns(sections: Vec<gtk::Box>) -> gtk::FlowBox {
    responsive_columns_with_limits(sections, 1, 2)
}

fn responsive_columns_with_limits(
    sections: Vec<gtk::Box>,
    min_children_per_line: u32,
    max_children_per_line: u32,
) -> gtk::FlowBox {
    let flow = gtk::FlowBox::builder()
        .column_spacing(12)
        .row_spacing(12)
        .homogeneous(false)
        .selection_mode(gtk::SelectionMode::None)
        .min_children_per_line(min_children_per_line)
        .max_children_per_line(max_children_per_line)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .build();

    for section in sections {
        section.set_hexpand(true);
        section.set_halign(gtk::Align::Fill);
        section.set_valign(gtk::Align::Start);
        section.set_width_request(320);
        section.set_focusable(false);
        let child = gtk::FlowBoxChild::builder()
            .child(&section)
            .focusable(false)
            .hexpand(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Start)
            .build();
        child.set_width_request(320);
        flow.insert(&child, -1);
    }

    flow
}
