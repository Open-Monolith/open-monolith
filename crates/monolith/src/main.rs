use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use egui_tiles::{Behavior, Tile, TileId, Tiles, Tree, UiResponse};

use bevy::{
    camera::RenderTarget,
    render::render_resource::{Extent3d, TextureFormat},
};
use bevy_egui::{EguiGlobalSettings, EguiTextureHandle, PrimaryEguiContext};

const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(24, 24, 24);
const CARD_COLOR: egui::Color32 = egui::Color32::from_rgb(40, 40, 40);
const CORNER_RADIUS: u8 = 8;

#[derive(Resource)]
struct Dock(Tree<&'static str>);

#[derive(Resource)]
struct ViewportImage(Handle<Image>);

struct DockBehavior {
    texture: egui::TextureId,
    viewport_size: Option<egui::Vec2>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, draw_ui)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut egui_settings: ResMut<EguiGlobalSettings>,
) {
    egui_settings.auto_create_primary_context = false;
    commands.spawn((Camera2d, PrimaryEguiContext));

    let image = images.add(Image::new_target_texture(
        512,
        512,
        TextureFormat::Bgra8UnormSrgb,
        None,
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.08, 0.08, 0.10)),

            ..default()
        },
        RenderTarget::Image(image.clone().into()),
        Transform::from_xyz(4.0, 3.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(ViewportImage(image));

    // Test cube.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 0.9),
            ..default()
        })),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mut tiles = Tiles::default();

    let viewport = tiles.insert_pane("Viewport");
    let properties = tiles.insert_pane("Properties");

    let left = tiles.insert_tab_tile(vec![viewport]);
    let right = tiles.insert_tab_tile(vec![properties]);
    let root = tiles.insert_horizontal_tile(vec![left, right]);

    commands.insert_resource(Dock(Tree::new("monolith", root, tiles)));
}

impl Behavior<&'static str> for DockBehavior {
    fn tab_title_for_pane(&mut self, pane: &&'static str) -> egui::WidgetText {
        (*pane).into()
    }

    fn tab_ui(
        &mut self,
        tiles: &mut Tiles<&'static str>,
        ui: &mut egui::Ui,
        id: egui::Id,
        tile_id: TileId,
        state: &egui_tiles::TabState,
    ) -> egui::Response {
        let title = match tiles.get(tile_id) {
            Some(Tile::Pane(name)) => *name,
            _ => "Pane",
        };

        ui.push_id(id, |ui| {
            ui.add(
                egui::Button::new(format!("!{title}"))
                    .frame(false)
                    .selected(state.active)
                    .sense(egui::Sense::click_and_drag()),
            )
        })
        .inner
    }

    fn tab_bar_left_ui(
        &mut self,
        tiles: &Tiles<&'static str>,
        ui: &mut egui::Ui,
        tile_id: TileId,
        _tabs: &egui_tiles::Tabs,
    ) {
        if let Some(rect) = tiles.rect(tile_id) {
            ui.ctx().layer_painter(ui.layer_id()).rect_filled(
                rect,
                egui::CornerRadius::same(CORNER_RADIUS),
                CARD_COLOR,
            );
        }

        ui.add_space(8.0);
    }

    fn tab_bar_color(&self, _visuals: &egui::Visuals) -> egui::Color32 {
        egui::Color32::TRANSPARENT
    }

    fn tab_bar_hline_stroke(&self, _visuals: &egui::Visuals) -> egui::Stroke {
        egui::Stroke::NONE
    }

    fn tab_outline_stroke(
        &self,
        _visuals: &egui::Visuals,
        _tiles: &Tiles<&'static str>,
        _tile_id: TileId,
        _state: &egui_tiles::TabState,
    ) -> egui::Stroke {
        egui::Stroke::NONE
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        5.0
    }

    fn resize_stroke(&self, _style: &egui::Style, state: egui_tiles::ResizeState) -> egui::Stroke {
        match state {
            egui_tiles::ResizeState::Idle => egui::Stroke::NONE,
            egui_tiles::ResizeState::Hovering => {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(75, 78, 85))
            }
            egui_tiles::ResizeState::Dragging => {
                egui::Stroke::new(3.0, egui::Color32::from_rgb(95, 100, 110))
            }
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: TileId,
        pane: &mut &'static str,
    ) -> UiResponse {
        if *pane == "Viewport" {
            egui::Frame::NONE
                .inner_margin(egui::Margin::same(6))
                .show(ui, |ui| {
                    let size = ui.available_size();

                    if size.x > 0.0 && size.y > 0.0 {
                        self.viewport_size = Some(size);
                        ui.image((self.texture, size));
                    }
                });
        } else {
            ui.label(*pane);
        }

        UiResponse::None
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

fn draw_ui(
    mut contexts: EguiContexts,
    mut dock: ResMut<Dock>,
    viewport: Res<ViewportImage>,
    mut images: ResMut<Assets<Image>>,
) -> Result {
    let texture = contexts.add_image(EguiTextureHandle::Strong(viewport.0.clone()));

    let mut behavior = DockBehavior {
        texture,
        viewport_size: None,
    };

    let ctx = contexts.ctx_mut()?;

    let mut root_ui = egui::Ui::new(
        ctx.clone(),
        "monolith_root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::CentralPanel::default()
        .frame(
            egui::Frame::NONE
                .fill(BACKGROUND)
                .inner_margin(egui::Margin::same(6)),
        )
        .show(&mut root_ui, |ui| {
            dock.0.ui(&mut behavior, ui);
        });

    // Match the render resolution to the pane's physical pixel size.
    if let Some(size) = behavior.viewport_size {
        let scale = ctx.pixels_per_point();

        let target_size = Extent3d {
            width: (size.x * scale).round().max(1.0) as u32,
            height: (size.y * scale).round().max(1.0) as u32,
            depth_or_array_layers: 1,
        };

        if let Some(mut image) = images.get_mut(&viewport.0) {
            if image.texture_descriptor.size != target_size {
                image.resize(target_size);
            }
        }
    }

    Ok(())
}
