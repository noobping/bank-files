use super::super::*;
use super::*;

pub(super) fn management_dialog_content_size(window: &adw::ApplicationWindow) -> (i32, i32) {
    management_dialog_content_dimensions(
        effective_parent_dimension(window.width(), window.default_width()),
        effective_parent_dimension(window.height(), window.default_height()),
    )
}

fn effective_parent_dimension(current: i32, default: i32) -> i32 {
    if current > 0 {
        current
    } else {
        default
    }
}

pub(super) fn management_dialog_content_dimensions(
    parent_width: i32,
    parent_height: i32,
) -> (i32, i32) {
    (
        management_dialog_content_dimension(
            parent_width,
            MANAGEMENT_DIALOG_MIN_WIDTH,
            MANAGEMENT_DIALOG_FALLBACK_WIDTH,
        ),
        management_dialog_content_dimension(
            parent_height,
            MANAGEMENT_DIALOG_MIN_HEIGHT,
            MANAGEMENT_DIALOG_FALLBACK_HEIGHT,
        ),
    )
}

fn management_dialog_content_dimension(parent: i32, minimum: i32, fallback: i32) -> i32 {
    if parent > 0 {
        (parent - MANAGEMENT_DIALOG_PARENT_INSET).max(minimum)
    } else {
        fallback
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn management_dialog_size_tracks_parent_with_inset() {
        assert_eq!(management_dialog_content_dimensions(1250, 900), (1202, 852));
    }

    #[test]
    fn management_dialog_size_uses_minimum_for_small_parent() {
        assert_eq!(management_dialog_content_dimensions(350, 380), (320, 360));
    }

    #[test]
    fn management_dialog_size_uses_fallback_before_parent_is_allocated() {
        assert_eq!(
            management_dialog_content_dimensions(0, 0),
            (
                MANAGEMENT_DIALOG_FALLBACK_WIDTH,
                MANAGEMENT_DIALOG_FALLBACK_HEIGHT
            )
        );
    }
}
