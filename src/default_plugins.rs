use bevy::app::{PluginGroup, PluginGroupBuilder};
pub struct DefaultBaseviewPlugins;

impl PluginGroup for DefaultBaseviewPlugins {
    fn name() -> String {
        "DefaultBaseviewPlugins".to_string()
    }

    fn build(self) -> PluginGroupBuilder  {
        // Disable log plugin as it sets global state and will panic if you re-open the app.
        // NOTE: Load this after renderer initialization so that it knows about the supported
        // compressed texture formats
        PluginGroupBuilder::start::<DefaultBaseviewPlugins>()
            //.add(bevy::log::LogPlugin::default())
            .add(bevy::core::TaskPoolPlugin::default())
            .add(bevy::core::TypeRegistrationPlugin)
            .add(bevy::core::FrameCountPlugin)
            .add(bevy::time::TimePlugin)
            .add(bevy::transform::TransformPlugin)
            .add(bevy::hierarchy::HierarchyPlugin)
            //.add(bevy::diagnostic::DiagnosticsPlugin)
            .add(bevy::input::InputPlugin)
            .add(bevy::window::WindowPlugin::default())

            .add(bevy::asset::AssetPlugin::default())
            .add(bevy::scene::ScenePlugin)
            //.add(crate::BaseviewPlugin::default())
            .add(bevy::render::RenderPlugin::default())
            //.add(crate::bullshit_render::BullshitRenderPlugin::default())
            .add(bevy::render::texture::ImagePlugin::default())
            .add(bevy::core_pipeline::CorePipelinePlugin)
            .add(bevy::sprite::SpritePlugin::default())
            .add(bevy::text::TextPlugin)
            .add(bevy::ui::UiPlugin::default())
            .add(bevy::pbr::PbrPlugin::default())
            //.add(bevy::gltf::GltfPlugin::default())
            //.add(bevy::gilrs::GilrsPlugin)
            .add(bevy::animation::AnimationPlugin)
            .add(bevy::gizmos::GizmoPlugin)
    }
}
