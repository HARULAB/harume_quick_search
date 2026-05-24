use aviutl2::common::{AnyResult, AviUtl2Info};
use aviutl2::generic::{GenericPlugin, GenericPluginTable, GlobalEditHandle, HostAppHandle};

static GLOBAL_EDIT_HANDLE: GlobalEditHandle = GlobalEditHandle::new();

#[aviutl2::plugin(GenericPlugin)]
pub struct HarumeEffectEditor {}

unsafe impl Send for HarumeEffectEditor {}
unsafe impl Sync for HarumeEffectEditor {}

impl GenericPlugin for HarumeEffectEditor {
    fn new(_info: AviUtl2Info) -> AnyResult<Self> {
        Ok(Self {})
    }

    fn plugin_info(&self) -> GenericPluginTable {
        GenericPluginTable {
            name: "HARUME Quick Search".into(),
            information: "Add/read/edit object effects with a modern native UI (egui)".into(),
        }
    }

    fn register(&mut self, registry: &mut HostAppHandle) {
        GLOBAL_EDIT_HANDLE.init(registry.create_edit_handle());

        registry.register_edit_menu("HARUME Quick Search\\Open Panel", || {
            gui::register_and_show();
        });

        if std::env::var("HARUME_EE_DEBUG_AUTOOPEN").ok().as_deref() == Some("1") {
            aviutl2::lprintln!("HARUME_EE: debug auto-open enabled");
            gui::register_and_show();
        }
    }
}

mod gui;

aviutl2::register_generic_plugin!(HarumeEffectEditor);

