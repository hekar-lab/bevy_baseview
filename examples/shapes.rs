//! Shows how to render simple primitive shapes with a single color.

use bevy::app::App;
use bevy::window::RawHandleWrapper;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use bevy_baseview::{DefaultBaseviewPlugins, ParentWindow};

use rwh_05::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

// Window size (logical).
const WINDOW_WIDTH: f64 = 500.0;
const WINDOW_HEIGHT: f64 = 400.0;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();

    let raw_handle = RawHandleWrapper {
        window_handle: window.raw_window_handle(),
        display_handle: window.raw_display_handle()
    };

    let parent_window = ParentWindow::from(raw_handle);
    let window_open_options = baseview::WindowOpenOptions {
        title: "Shapes example".to_string(),
        size: baseview::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        scale: baseview::WindowScalePolicy::SystemScaleFactor,
    };
    bevy_baseview::open_parented(
        parent_window, 
        window_open_options,
        build
    );

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}

fn build(app: &mut App) -> &mut App {
    app.add_plugins(DefaultBaseviewPlugins)
        .add_systems(Startup, setup)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Circle
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
        ..default()
    });

    // Rectangle
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(50.0, 100.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
        ..default()
    });

    // Quad
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(Vec2::new(50., 100.)).into())
            .into(),
        material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
        transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
        ..default()
    });

    // Hexagon
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
        material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
        transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
        ..default()
    });
}
