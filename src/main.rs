// source: https://bevyengine.org/examples/animation/animated-mesh/
use std::f32::consts::PI;
use bevy::prelude::*;
use bevy::scene::{SceneRoot, SceneInstanceReady};
use bevy::prelude::GltfAssetLabel;
use bevy::animation::graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex};


use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::input::mouse::{MouseMotion, MouseButton};

// An example asset that contains a mesh and animation.
const GLTF_PATH: &str = "models/animated/Fox.glb";
const LIGHT_ROTATION_SPEED: f32 = 1.0;
const LIGHT_ELEVATION_SPEED: f32 = 0.5;
const CAMERA_ROTATION_SENSITIVITY: f32 = 0.005;

// Resource to track if mouse is being dragged
#[derive(Resource, Default)]
struct MouseDragState {
    dragging: bool,
}

// Component to identify the light
#[derive(Component)]
struct MainLight;

// Resource to track sun position and time of day
#[derive(Resource)]
struct DayNightCycle {
    // 0 = horizon, PI/2 = noon, -PI/2 = midnight
    sun_elevation: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            sun_elevation: PI / 4.0, // Start at mid-morning
        }
    }
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
            ..default()
        })
        .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 1.0))) // Default sky color
        .init_resource::<MouseDragState>()
        .init_resource::<DayNightCycle>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_mesh_and_animation)
        .add_systems(Startup, setup_camera_and_environment)
        .add_systems(Update, (
            rotate_light_system,
            elevate_light_system,
            update_sky_color,
            mouse_drag_system,
            rotate_camera_system,
        ))
        .run();
}

// A component that stores a reference to an animation we want to play
#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn setup_mesh_and_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Create an animation graph containing a single animation. We want the "run"
    // animation from our example asset, which has an index of two.
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(GLTF_PATH)),
    );

    // Store the animation graph as an asset.
    let graph_handle = graphs.add(graph);

    // Create a component that stores a reference to our animation.
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index,
    };

    // Start loading the asset as a scene and store a reference to it in a
    // SceneRoot component. This component will automatically spawn a scene
    // containing our mesh once it has loaded.
    let mesh_scene = SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(GLTF_PATH)));

    // Spawn an entity with our components, and connect it to an observer that
    // will trigger when the scene is loaded and spawned.
    commands
        .spawn((animation_to_play, mesh_scene))
        .observe(play_animation_when_ready);
}

fn play_animation_when_ready(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    // The entity we spawned in `setup_mesh_and_animation` is the trigger's target.
    // Start by finding the AnimationToPlay component we added to that entity.
    if let Ok(animation_to_play) = animations_to_play.get(trigger.target()) {
        // The SceneRoot component will have spawned the scene as a hierarchy
        // of entities parented to our entity. Since the asset contained a skinned
        // mesh and animations, it will also have spawned an animation player
        // component. Search our entity's descendants to find the animation player.
        for child in children.iter_descendants(trigger.target()) {
            if let Ok(mut player) = players.get_mut(child) {
                // Tell the animation player to start the animation and keep
                // repeating it.
                player.play(animation_to_play.index).repeat();

                // Add the animation graph. This only needs to be done once to
                // connect the animation player to the mesh.
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}

// Spawn a camera and a simple environment with a ground plane and light.
fn setup_camera_and_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    day_night_cycle: Res<DayNightCycle>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(100.0, 100.0, 150.0).looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
    ));

    // Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Light with MainLight component for identification
    // Initialize with the current sun elevation
    let light_transform = Transform::from_rotation(
        Quat::from_euler(
            EulerRot::XYZ,
            -day_night_cycle.sun_elevation, // X rotation controls elevation
            1.0,                           // Y rotation (azimuth)
            0.0,
        )
    );

    commands.spawn((
        MainLight,
        light_transform,
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 100000.0,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));
}

// System to rotate the light with A and D keys (horizontal rotation)
fn rotate_light_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<MainLight>>,
) {
    if let Ok(mut transform) = query.single_mut() {
        let mut rotation = 0.0;
        
        if keyboard_input.pressed(KeyCode::KeyA) {
            rotation += LIGHT_ROTATION_SPEED * time.delta_secs();
        }
        
        if keyboard_input.pressed(KeyCode::KeyD) {
            rotation -= LIGHT_ROTATION_SPEED * time.delta_secs();
        }
        
        if rotation != 0.0 {
            // Rotate around Y axis (horizontal rotation)
            transform.rotate_y(rotation);
        }
    }
}

// System to elevate the light with W and S keys (vertical rotation)
fn elevate_light_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<MainLight>>,
    mut day_night_cycle: ResMut<DayNightCycle>,
) {
    if let Ok(mut transform) = query.single_mut() {
        let mut elevation_change = 0.0;
        
        if keyboard_input.pressed(KeyCode::KeyS) {
            elevation_change -= LIGHT_ELEVATION_SPEED * time.delta_secs();
        }
        
        if keyboard_input.pressed(KeyCode::KeyW) {
            elevation_change += LIGHT_ELEVATION_SPEED * time.delta_secs();
        }
        
        if elevation_change != 0.0 {
            // Update the sun elevation in our resource
            day_night_cycle.sun_elevation = (day_night_cycle.sun_elevation + elevation_change)
                .clamp(-PI / 2.0, PI / 2.0); // Clamp between midnight and noon
            
            // Get the current rotation as euler angles
            let (mut x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
            
            // Update the X rotation (elevation)
            x = -day_night_cycle.sun_elevation;
            
            // Apply the new rotation
            transform.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z);
        }
    }
}

// System to update sky color based on sun position
fn update_sky_color(
    day_night_cycle: Res<DayNightCycle>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient_light: ResMut<AmbientLight>,
    mut query: Query<&mut DirectionalLight, With<MainLight>>,
) {
    if !day_night_cycle.is_changed() {
        return;
    }
    
    // Get the sun elevation (normalized from -PI/2 to PI/2)
    let elevation = day_night_cycle.sun_elevation;
    
    // Calculate a normalized value between 0 and 1 where:
    // 0 = midnight, 0.5 = horizon, 1 = noon
    let time_of_day = (elevation + PI / 2.0) / PI;
    
    // Update sky color based on time of day
    let sky_color = if time_of_day > 0.8 {
        // Noon - bright blue sky
        Color::srgb(0.5, 0.7, 1.0)
    } else if time_of_day > 0.6 {
        // Morning - light blue with a hint of yellow
        Color::srgb(0.6, 0.8, 1.0)
    } else if time_of_day > 0.5 {
        // Sunrise - orange and pink
        Color::srgb(0.9, 0.6, 0.6)
    } else if time_of_day > 0.4 {
        // Just before sunrise - deep blue with a hint of orange
        Color::srgb(0.3, 0.3, 0.6)
    } else if time_of_day > 0.2 {
        // Night - dark blue
        Color::srgb(0.1, 0.1, 0.3)
    } else {
        // Midnight - almost black
        Color::srgb(0.02, 0.02, 0.1)
    };
    
    // Update the clear color (sky)
    clear_color.0 = sky_color;
    
    // Update ambient light brightness based on time of day
    let ambient_brightness = if time_of_day > 0.5 {
        // Day
        2000.0 * (time_of_day - 0.5) * 2.0
    } else {
        // Night
        400.0 * time_of_day * 2.0
    };
    
    ambient_light.brightness = ambient_brightness;
    
    // Update directional light brightness
    if let Ok(mut light) = query.single_mut() {
        // Adjust light brightness based on elevation
        if elevation > 0.0 {
            // Day
            light.illuminance = 100000.0 * (elevation / (PI / 2.0));
        } else {
            // Night - no directional light
            light.illuminance = 0.0;
        }
    }
}

// System to track mouse drag state
fn mouse_drag_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut drag_state: ResMut<MouseDragState>,
) {
    if mouse_button.pressed(MouseButton::Left) {
        drag_state.dragging = true;
    } else {
        drag_state.dragging = false;
    }
}

// System to rotate camera based on mouse movement
fn rotate_camera_system(
    drag_state: Res<MouseDragState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if !drag_state.dragging {
        return;
    }
    
    let mut rotation = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        rotation += event.delta;
    }
    
    if rotation.length_squared() > 0.0 {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            // Get the camera's current focus point (what it's looking at)
            let forward = camera_transform.forward();
            let distance = camera_transform.translation.length();
            
            // Rotate around Y axis for horizontal mouse movement
            camera_transform.rotate_y(-rotation.x * CAMERA_ROTATION_SENSITIVITY);
            
            // Rotate around local X axis for vertical mouse movement
            let right = camera_transform.right();
            camera_transform.rotate_axis(right, -rotation.y * CAMERA_ROTATION_SENSITIVITY);
            
            // Maintain the same distance from origin
            camera_transform.translation = -forward * distance;
        }
    }
}
