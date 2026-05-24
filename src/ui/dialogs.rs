use super::*;

#[derive(Clone, Copy)]
pub struct AlertResponse {
    id: &'static str,
    label: &'static str,
    appearance: Option<adw::ResponseAppearance>,
}

impl AlertResponse {
    pub const fn neutral(id: &'static str, label: &'static str) -> Self {
        Self {
            id,
            label,
            appearance: None,
        }
    }

    #[cfg(any(
        target_os = "windows",
        all(target_os = "linux", feature = "setup", not(feature = "flatpak"))
    ))]
    pub const fn suggested(id: &'static str, label: &'static str) -> Self {
        Self {
            id,
            label,
            appearance: Some(adw::ResponseAppearance::Suggested),
        }
    }

    pub const fn destructive(id: &'static str, label: &'static str) -> Self {
        Self {
            id,
            label,
            appearance: Some(adw::ResponseAppearance::Destructive),
        }
    }
}

pub struct AlertDialogBuilder {
    heading: String,
    body: String,
    responses: Vec<AlertResponse>,
    close_response: Option<&'static str>,
    default_response: Option<&'static str>,
}

impl AlertDialogBuilder {
    pub fn responses(mut self, responses: &[AlertResponse]) -> Self {
        self.responses.extend_from_slice(responses);
        self
    }

    pub fn close_response(mut self, response: &'static str) -> Self {
        self.close_response = Some(response);
        self
    }

    pub fn default_response(mut self, response: &'static str) -> Self {
        self.default_response = Some(response);
        self
    }

    pub fn build(self) -> adw::AlertDialog {
        let dialog = adw::AlertDialog::builder()
            .heading(self.heading)
            .body(self.body)
            .build();

        for response in self.responses {
            let label = gettext(response.label);
            dialog.add_response(response.id, &label);
            if let Some(appearance) = response.appearance {
                dialog.set_response_appearance(response.id, appearance);
            }
        }

        if let Some(response) = self.close_response {
            dialog.set_close_response(response);
        }
        if let Some(response) = self.default_response {
            dialog.set_default_response(Some(response));
        }

        dialog
    }
}

pub fn alert_dialog(heading: impl Into<String>, body: impl Into<String>) -> AlertDialogBuilder {
    AlertDialogBuilder {
        heading: heading.into(),
        body: body.into(),
        responses: Vec::new(),
        close_response: None,
        default_response: None,
    }
}

#[cfg(any(
    target_os = "windows",
    all(target_os = "linux", feature = "setup", not(feature = "flatpak"))
))]
pub fn present_alert_dialog(dialog: &adw::AlertDialog, parent: Option<&impl IsA<gtk::Widget>>) {
    if let Some(parent) = parent {
        dialog.present(Some(parent));
    } else {
        dialog.present(None::<&gtk::Widget>);
    }
}

pub struct ContentDialogBuilder {
    title: String,
    child: gtk::Widget,
    content_width: Option<i32>,
    content_height: Option<i32>,
    width_request: Option<i32>,
    height_request: Option<i32>,
    follows_content_size: bool,
    default_widget: Option<gtk::Widget>,
}

impl ContentDialogBuilder {
    pub fn content_width(mut self, width: i32) -> Self {
        self.content_width = Some(width);
        self
    }

    pub fn content_height(mut self, height: i32) -> Self {
        self.content_height = Some(height);
        self
    }

    pub fn width_request(mut self, width: i32) -> Self {
        self.width_request = Some(width);
        self
    }

    pub fn height_request(mut self, height: i32) -> Self {
        self.height_request = Some(height);
        self
    }

    #[cfg(any(
        target_os = "windows",
        all(target_os = "linux", feature = "setup", not(feature = "flatpak"))
    ))]
    pub fn follows_content_size(mut self) -> Self {
        self.follows_content_size = true;
        self
    }

    pub fn default_widget(mut self, widget: &impl IsA<gtk::Widget>) -> Self {
        self.default_widget = Some(widget.clone().upcast::<gtk::Widget>());
        self
    }

    pub fn build(self) -> adw::Dialog {
        let mut builder = adw::Dialog::builder()
            .title(self.title)
            .child(&self.child)
            .follows_content_size(self.follows_content_size);

        if let Some(width) = self.content_width {
            builder = builder.content_width(width);
        }
        if let Some(height) = self.content_height {
            builder = builder.content_height(height);
        }
        if let Some(width) = self.width_request {
            builder = builder.width_request(width);
        }
        if let Some(height) = self.height_request {
            builder = builder.height_request(height);
        }
        if let Some(default_widget) = self.default_widget {
            builder = builder.default_widget(&default_widget);
        }

        builder.build()
    }
}

pub fn content_dialog(
    title: impl Into<String>,
    child: &impl IsA<gtk::Widget>,
) -> ContentDialogBuilder {
    ContentDialogBuilder {
        title: title.into(),
        child: child.clone().upcast::<gtk::Widget>(),
        content_width: None,
        content_height: None,
        width_request: None,
        height_request: None,
        follows_content_size: false,
        default_widget: None,
    }
}

pub struct AboutDialogDetails<'a> {
    pub application_name: &'a str,
    pub application_icon: &'a str,
    pub developer_name: &'a str,
    pub version: &'a str,
    pub comments: &'a str,
    pub copyright: &'a str,
    pub website: &'a str,
    pub issue_url: &'a str,
    pub license_type: gtk::License,
}

pub fn about_dialog(details: AboutDialogDetails<'_>) -> adw::AboutDialog {
    adw::AboutDialog::builder()
        .application_name(details.application_name)
        .application_icon(details.application_icon)
        .developer_name(details.developer_name)
        .version(details.version)
        .comments(details.comments)
        .copyright(details.copyright)
        .website(details.website)
        .issue_url(details.issue_url)
        .license_type(details.license_type)
        .build()
}

pub fn shortcuts_dialog(
    title: impl Into<String>,
    content_width: i32,
    content_height: i32,
) -> adw::ShortcutsDialog {
    adw::ShortcutsDialog::builder()
        .title(title.into())
        .content_width(content_width)
        .content_height(content_height)
        .build()
}
