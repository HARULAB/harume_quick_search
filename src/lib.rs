use aviutl2::common::{AnyResult, AviUtl2Info};
use aviutl2::generic::{GenericPlugin, GenericPluginTable, GlobalEditHandle, HostAppHandle};

static GLOBAL_EDIT_HANDLE: GlobalEditHandle = GlobalEditHandle::new();

mod config;
mod gui;
mod i18n;

#[aviutl2::plugin(GenericPlugin)]
pub struct QuickSearch {}

// 状態を持たない（HWNDはgui側のstaticが保持する）ため
// unsafe impl Send/Sync は不要。

impl GenericPlugin for QuickSearch {
    fn new(_info: AviUtl2Info) -> AnyResult<Self> {
        Ok(Self {})
    }

    fn plugin_info(&self) -> GenericPluginTable {
        GenericPluginTable {
            name: "Quick Search".into(),
            information: "Quickly search and add AviUtl2 effects with a native UI (egui)".into(),
        }
    }

    fn register(&mut self, registry: &mut HostAppHandle) {
        GLOBAL_EDIT_HANDLE.init(registry.create_edit_handle());

        registry.register_edit_menu("Quick Search\\Open Panel", || {
            gui::register_and_show();
        });

        registry.register_edit_menu("Quick Search\\Settings", || {
            gui::register_and_show_settings();
        });

        if std::env::var("QUICK_SEARCH_DEBUG_AUTOOPEN").ok().as_deref() == Some("1") {
            aviutl2::lprintln!("QuickSearch: debug auto-open enabled");
            gui::register_and_show();
        }
    }
}

aviutl2::register_generic_plugin!(QuickSearch);
