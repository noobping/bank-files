use super::*;

pub(in crate::app) fn connect_rule_form_reorder(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<RuleForm>>>,
    drag_handle: &gtk::Button,
    form_box: &gtk::Box,
) {
    connect_form_reorder(container, forms, drag_handle, form_box);
}

pub(in crate::app) fn connect_budget_form_reorder(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<BudgetForm>>>,
    drag_handle: &gtk::Button,
    form_box: &gtk::Box,
) {
    connect_form_reorder(container, forms, drag_handle, form_box);
}

trait ReorderableForm {
    fn form_box(&self) -> &gtk::Box;
}

impl ReorderableForm for RuleForm {
    fn form_box(&self) -> &gtk::Box {
        &self.form_box
    }
}

impl ReorderableForm for BudgetForm {
    fn form_box(&self) -> &gtk::Box {
        &self.form_box
    }
}

fn connect_form_reorder<T: ReorderableForm + 'static>(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<T>>>,
    drag_handle: &gtk::Button,
    form_box: &gtk::Box,
) {
    let source = gtk::DragSource::builder()
        .actions(gtk::gdk::DragAction::MOVE)
        .build();
    let forms_for_prepare = Rc::clone(forms);
    let form_box_for_prepare = form_box.clone();
    source.connect_prepare(move |_, _, _| {
        form_index(&forms_for_prepare.borrow(), &form_box_for_prepare)
            .map(|index| gtk::gdk::ContentProvider::for_value(&(index as u32).to_value()))
    });
    drag_handle.add_controller(source);

    let target = gtk::DropTarget::new(u32::static_type(), gtk::gdk::DragAction::MOVE);
    let forms_for_drop = Rc::clone(forms);
    let container_for_drop = container.clone();
    let form_box_for_drop = form_box.clone();
    target.connect_drop(move |_, value, _, y| {
        let Ok(source_index) = value.get::<u32>() else {
            return false;
        };
        let Some(target_index) = form_index(&forms_for_drop.borrow(), &form_box_for_drop) else {
            return false;
        };
        let target_boundary = drop_boundary(target_index, form_box_for_drop.height(), y);
        reorder_forms(
            &container_for_drop,
            &forms_for_drop,
            source_index as usize,
            target_boundary,
        )
    });
    form_box.add_controller(target);
}

fn form_index<T: ReorderableForm>(forms: &[T], form_box: &gtk::Box) -> Option<usize> {
    forms.iter().position(|form| form.form_box() == form_box)
}

fn drop_boundary(target_index: usize, widget_height: i32, y: f64) -> usize {
    if y > f64::from(widget_height) / 2.0 {
        target_index + 1
    } else {
        target_index
    }
}

fn reorder_forms<T: ReorderableForm>(
    container: &gtk::Box,
    forms: &Rc<RefCell<Vec<T>>>,
    source_index: usize,
    target_boundary: usize,
) -> bool {
    let mut forms = forms.borrow_mut();
    if !move_item_to_boundary(&mut forms, source_index, target_boundary) {
        return false;
    }
    reorder_form_widgets(container, &forms);
    true
}

fn reorder_form_widgets<T: ReorderableForm>(container: &gtk::Box, forms: &[T]) {
    let mut previous: Option<gtk::Widget> = None;
    for form in forms {
        let form_box = form.form_box();
        container.reorder_child_after(form_box, previous.as_ref());
        previous = Some(form_box.clone().upcast::<gtk::Widget>());
    }
}

fn move_item_to_boundary<T>(
    items: &mut Vec<T>,
    source_index: usize,
    target_boundary: usize,
) -> bool {
    if source_index >= items.len()
        || target_boundary > items.len()
        || target_boundary == source_index
        || target_boundary == source_index + 1
    {
        return false;
    }

    let item = items.remove(source_index);
    let insert_index = if source_index < target_boundary {
        target_boundary - 1
    } else {
        target_boundary
    };
    items.insert(insert_index, item);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_item_to_boundary_moves_before_target() {
        let mut items = vec!["a", "b", "c"];

        assert!(move_item_to_boundary(&mut items, 0, 2));

        assert_eq!(items, vec!["b", "a", "c"]);
    }

    #[test]
    fn move_item_to_boundary_moves_after_target() {
        let mut items = vec!["a", "b", "c"];

        assert!(move_item_to_boundary(&mut items, 0, 3));

        assert_eq!(items, vec!["b", "c", "a"]);
    }

    #[test]
    fn move_item_to_boundary_ignores_noop_and_invalid_moves() {
        let mut items = vec!["a", "b", "c"];

        assert!(!move_item_to_boundary(&mut items, 1, 1));
        assert!(!move_item_to_boundary(&mut items, 1, 2));
        assert!(!move_item_to_boundary(&mut items, 3, 1));
        assert!(!move_item_to_boundary(&mut items, 1, 4));

        assert_eq!(items, vec!["a", "b", "c"]);
    }

    #[test]
    fn drop_boundary_uses_target_half() {
        assert_eq!(drop_boundary(2, 100, 40.0), 2);
        assert_eq!(drop_boundary(2, 100, 50.0), 2);
        assert_eq!(drop_boundary(2, 100, 51.0), 3);
    }
}
