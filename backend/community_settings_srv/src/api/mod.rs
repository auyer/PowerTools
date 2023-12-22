mod get_setting;
mod save_setting;

pub use get_setting::get_setting_handler as get_setting_by_id;
pub use save_setting::save_setting_handler as save_setting_with_new_id;

pub(self) fn is_mime_type_ron_capable(mimetype: &mime::Mime) -> bool {
    (mimetype.type_() == "application" || mimetype.type_() == mime::STAR)
    && (mimetype.subtype() == "ron" || mimetype.subtype() == "cc.replicated.ron" || mimetype.subtype() == "w-ron")
}
