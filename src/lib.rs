//! Plugin to simulate and preview different types of
//! color blindness.
//!
//! This lets you ensure that your game is accessible to all players by testing how it
//! will be seen under different conditions. While this is important,
//! please also consider not relying on color alone to convey important information to your players.
//! A common option is to add identifying symbols, like in the game
//! [Hue](https://gameaccessibilityguidelines.com/hue-colorblind-mode/).
//!
//! Based on [Alan Zucconi's post](https://www.alanzucconi.com/2015/12/16/color-blindness/).
//! Supports: Normal, Protanopia, Protanomaly, Deuteranopia, Deuteranomaly,
//! Tritanopia, Tritanomaly, Achromatopsia, and Achromatomaly.
//!
//! # Using
//!
//! First, add the following to your `Cargo.toml`:
//!
//!```toml
//! bevy_color_blindness = "0.1.0"
//!```
//!
//! Then, add the [`ColorBlindnessPlugin`] to your app, and add [`ColorBlindnessCamera`] to
//! your main camera.
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_color_blindness::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         // add the plugin
//!         .add_plugin(ColorBlindnessPlugin)
//!         .add_startup_system(setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     // set up your scene...
//!
//!     // create the camera
//!     commands
//!         .spawn_bundle(Camera3dBundle {
//!             transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
//!             ..Default::default()
//!         })
//!         .insert(ColorBlindnessCamera {
//!             mode: ColorBlindnessMode::Deuteranopia,
//!             enabled: true,
//!         });
//! }
//! ```
//!
//! # Important note
//!
//! This plugin only simulates how color blind players will see your game.
//! It does not correct for color blindness to make your game more accessible.
//! This plugin should only be used during development, and removed on final builds.

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::{Camera, RenderTarget},
        mesh::{shape, Mesh},
        prelude::Image,
        render_resource::{
            AsBindGroup, Extent3d, Shader, ShaderRef, ShaderType, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

/// Plugin to simulate and preview different types of
/// color blindness.
///
/// This lets you ensure that your game is accessible to all players by testing how it
/// will be seen under different conditions. While this is important,
/// please also consider not relying on color alone to convey important information to your players.
/// A common option is to add identifying symbols, like in the game
/// [Hue](https://gameaccessibilityguidelines.com/hue-colorblind-mode/).
///
/// Based on [Alan Zucconi's post](https://www.alanzucconi.com/2015/12/16/color-blindness/).
/// Supports: Normal, Protanopia, Protanomaly, Deuteranopia, Deuteranomaly,
/// Tritanopia, Tritanomaly, Achromatopsia, and Achromatomaly.
///
/// First, add the [`ColorBlindnessPlugin`] to your app, and add [`ColorBlindnessCamera`] to
/// your main camera.
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_color_blindness::*;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         // add the plugin
///         .add_plugin(ColorBlindnessPlugin)
///         .add_startup_system(setup)
///         .run();
/// }
///
/// fn setup(mut commands: Commands) {
///     // set up your scene...
///
///     // create the camera
///     commands
///         .spawn_bundle(Camera3dBundle {
///             transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
///             ..Default::default()
///         })
///         .insert(ColorBlindnessCamera {
///             mode: ColorBlindnessMode::Deuteranopia,
///             enabled: true,
///         });
/// }
/// ```
///
/// # Important note
///
/// This plugin only simulates how color blind players will see your game.
/// It does not correct for color blindness to make your game more accessible.
/// This plugin should only be used during development, and removed on final builds.
pub struct ColorBlindnessPlugin;
impl Plugin for ColorBlindnessPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            COLOR_BLINDNESS_SHADER_HANDLE,
            "color_blindness.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(Material2dPlugin::<ColorBlindnessMaterial>::default())
            .add_system(setup_new_color_blindness_cameras)
            .add_system(update_percentages);
    }
}

/// handle to the color blindness simulation shader
const COLOR_BLINDNESS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 3937837360667146578);

/// The different modes of color blindness simulation supported.
#[derive(Clone, Default, Debug)]
pub enum ColorBlindnessMode {
    /// Normal full color vision
    #[default]
    Normal,
    // Descriptions of the different types of color blindness are sourced from:
    // https://www.nei.nih.gov/learn-about-eye-health/eye-conditions-and-diseases/color-blindness/types-color-blindness
    /// Inability to differentiate between green and red.
    Protanopia,
    /// Condition where red looks more green.
    Protanomaly,
    /// Inability to differentiate between green and red.
    Deuteranopia,
    /// Condition where green looks more red.
    Deuteranomaly,
    /// Inability to differentiate between blue and green, purple and red, and yellow and pink.
    Tritanopia,
    /// Difficulty differentiating between blue and green, and between yellow and red
    Tritanomaly,
    /// Absence of color discrimination.
    Achromatopsia,
    /// All color cones have some form of deficiency.
    Achromatomaly,
}

/// Indicates how to mix the RGB channels to obtain output colors.
///
/// Normal vision corresponds to the following:
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_color_blindness::*;
/// # fn _none() -> ColorBlindnessPercentages {
/// ColorBlindnessPercentages {
///     // red channel output is 100% red, 0% green, 0% blue
///     red: Vec3::X,
///     // green channel is 0% red, 100% green, 0% blue
///     green: Vec3::Y,
///     // blue channel is 0% red, 0% green, 100% blue
///     blue: Vec3::Z
/// }
/// # }
/// ```
#[derive(ShaderType, Clone, Debug)]
pub struct ColorBlindnessPercentages {
    /// Percentages of red, green, and blue to mix on the red channel.
    pub red: Vec3,
    /// Percentages of red, green, and blue to mix on the green channel.
    pub green: Vec3,
    /// Percentages of red, green, and blue to mix on the blue channel.
    pub blue: Vec3,
}

impl ColorBlindnessPercentages {
    /// Creates a new `ColorBlindnessPercentages`
    fn new(red: Vec3, green: Vec3, blue: Vec3) -> Self {
        Self { red, green, blue }
    }
}

impl ColorBlindnessMode {
    /// Returns the percentages of colors to mix corresponding to each type of color blindness.
    ///
    /// [Source](https://web.archive.org/web/20081014161121/http://www.colorjack.com/labs/colormatrix/)
    pub fn percentages(&self) -> ColorBlindnessPercentages {
        // table from https://www.alanzucconi.com/2015/12/16/color-blindness/
        // https://web.archive.org/web/20081014161121/http://www.colorjack.com/labs/colormatrix/

        match self {
            ColorBlindnessMode::Normal => ColorBlindnessPercentages::new(Vec3::X, Vec3::Y, Vec3::Z),
            ColorBlindnessMode::Protanopia => ColorBlindnessPercentages::new(
                [0.56667, 0.43333, 0.0].into(),
                [0.55833, 0.44167, 0.0].into(),
                [0.0, 0.24167, 0.75833].into(),
            ),
            ColorBlindnessMode::Protanomaly => ColorBlindnessPercentages::new(
                [0.81667, 0.18333, 0.0].into(),
                [0.33333, 0.66667, 0.0].into(),
                [0.0, 0.125, 0.875].into(),
            ),
            ColorBlindnessMode::Deuteranopia => ColorBlindnessPercentages::new(
                [0.625, 0.375, 0.0].into(),
                [0.70, 0.30, 0.0].into(),
                [0.0, 0.30, 0.70].into(),
            ),
            ColorBlindnessMode::Deuteranomaly => ColorBlindnessPercentages::new(
                [0.80, 0.20, 0.0].into(),
                [0.25833, 0.74167, 0.0].into(),
                [0.0, 0.14167, 0.85833].into(),
            ),
            ColorBlindnessMode::Tritanopia => ColorBlindnessPercentages::new(
                [0.95, 0.5, 0.0].into(),
                [0.0, 0.43333, 0.56667].into(),
                [0.0, 0.475, 0.525].into(),
            ),
            ColorBlindnessMode::Tritanomaly => ColorBlindnessPercentages::new(
                [0.96667, 0.3333, 0.0].into(),
                [0.0, 0.73333, 0.26667].into(),
                [0.0, 0.18333, 0.81667].into(),
            ),
            ColorBlindnessMode::Achromatopsia => ColorBlindnessPercentages::new(
                [0.299, 0.587, 0.114].into(),
                [0.299, 0.587, 0.114].into(),
                [0.299, 0.587, 0.114].into(),
            ),
            ColorBlindnessMode::Achromatomaly => ColorBlindnessPercentages::new(
                [0.618, 0.32, 0.62].into(),
                [0.163, 0.775, 0.62].into(),
                [0.163, 0.320, 0.516].into(),
            ),
        }
    }

    /// Changes `self` to the next `ColorBlindnessMode`.
    ///
    /// Useful for writing something like the following:
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_color_blindness::*;
    /// fn change_mode(input: Res<Input<KeyCode>>, mut cameras: Query<&mut ColorBlindnessCamera>) {
    ///     for mut camera in &mut cameras {
    ///         // cycle through the modes by pressing N
    ///         if input.just_pressed(KeyCode::N) {
    ///             camera.mode.cycle();
    ///             println!("Changed to {:?}", camera.mode);
    ///         }
    ///
    ///         camera.enabled = input.pressed(KeyCode::Space);
    ///     }
    /// }
    /// ```
    pub fn cycle(&mut self) {
        *self = match self {
            ColorBlindnessMode::Normal => ColorBlindnessMode::Protanopia,
            ColorBlindnessMode::Protanopia => ColorBlindnessMode::Protanomaly,
            ColorBlindnessMode::Protanomaly => ColorBlindnessMode::Deuteranopia,
            ColorBlindnessMode::Deuteranopia => ColorBlindnessMode::Deuteranomaly,
            ColorBlindnessMode::Deuteranomaly => ColorBlindnessMode::Tritanopia,
            ColorBlindnessMode::Tritanopia => ColorBlindnessMode::Tritanomaly,
            ColorBlindnessMode::Tritanomaly => ColorBlindnessMode::Achromatopsia,
            ColorBlindnessMode::Achromatopsia => ColorBlindnessMode::Achromatomaly,
            ColorBlindnessMode::Achromatomaly => ColorBlindnessMode::Normal,
        };
    }
}

/// Post processing material that applies color blindness simulation to `image`
#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "bc2f08eb-a0fb-43f1-a908-54871ea597d5"]
struct ColorBlindnessMaterial {
    /// In this example, this image will be the result of the main camera.
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[uniform(2)]
    percentages: ColorBlindnessPercentages,
}

impl Material2d for ColorBlindnessMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(COLOR_BLINDNESS_SHADER_HANDLE.typed())
    }
}

/// Component to identify your main camera
///
/// Adding this component to a camera will set up the post-processing pipeline
/// which simulates color blindness. This is done by changing the render target
/// to be an image, and then using another camera to render that image.
///
/// Cameras with `ColorBlindnessCamera` will have [`UiCameraConfig`] inserted with
/// `show_ui` set to `false`. This is to ensure that UI elements are not rendered twice.
/// In most cases, you will want to render UI using the final post-processing camera.
/// If for some reason this behavior is not desired, please open an issue.
///
/// [`UiCameraConfig`]: bevy::prelude::UiCameraConfig
#[derive(Component, Default)]
pub struct ColorBlindnessCamera {
    /// Selects the color blindness mode to use
    ///
    /// Defaults to `ColorBlindnessMode::Normal`
    pub mode: ColorBlindnessMode,
    /// Controls whether color blindness simulation is enabled
    ///
    /// Defaults to `false`
    pub enabled: bool,
}

/// updates the percentages in the post processing material when the values in `ColorBlindnessCamera` change
fn update_percentages(
    cameras: Query<
        (&Handle<ColorBlindnessMaterial>, &ColorBlindnessCamera),
        Changed<ColorBlindnessCamera>,
    >,
    mut materials: ResMut<Assets<ColorBlindnessMaterial>>,
) {
    for (handle, camera) in &cameras {
        let mut mat = materials.get_mut(handle).unwrap();

        let mode = if camera.enabled {
            &camera.mode
        } else {
            &ColorBlindnessMode::Normal
        };

        mat.percentages = mode.percentages();
    }
}

/// sets up post processing for cameras that have had `ColorBlindnessCamera` added
fn setup_new_color_blindness_cameras(
    mut commands: Commands,
    windows: Res<Windows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut post_processing_materials: ResMut<Assets<ColorBlindnessMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<(Entity, &mut Camera, &ColorBlindnessCamera), Added<ColorBlindnessCamera>>,
) {
    for (entity, mut camera, color_blindness_camera) in &mut cameras {
        let original_target = camera.target.clone();

        // Get the size the camera is rendering to
        let size = match &camera.target {
            RenderTarget::Window(window_id) => {
                let window = windows.get(*window_id).expect("ColorBlindnessCamera is rendering to a window, but this window could not be found");
                Extent3d {
                    width: window.physical_width(),
                    height: window.physical_height(),
                    ..Default::default()
                }
            }
            RenderTarget::Image(handle) => {
                let image = images.get(handle).expect(
                    "ColorBlindnessCamera is rendering to an Image, but this Image could not be found",
                );
                image.texture_descriptor.size
            }
        };

        // This is the texture that will be rendered to.
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..Default::default()
        };

        // fill image.data with zeroes
        image.resize(size);

        let image_handle = images.add(image);

        // This specifies the layer used for the post processing camera, which will be attached to the post processing camera and 2d quad.
        let post_processing_pass_layer =
            RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

        let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            size.width as f32,
            size.height as f32,
        ))));

        // This material has the texture that has been rendered.
        let material_handle = post_processing_materials.add(ColorBlindnessMaterial {
            source_image: image_handle.clone(),
            percentages: color_blindness_camera.mode.percentages(),
        });

        commands
            .entity(entity)
            // add the handle to the camera so we can access it and change the percentages
            .insert(material_handle.clone())
            // also disable show_ui so UI elements don't get rendered twice
            .insert(UiCameraConfig { show_ui: false });

        camera.target = RenderTarget::Image(image_handle);

        // Post processing 2d quad, with material using the render texture done by the main camera, with a custom shader.
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: quad_handle.into(),
                material: material_handle,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.5),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(post_processing_pass_layer);

        // The post-processing pass camera.
        commands
            .spawn_bundle(Camera2dBundle {
                camera: Camera {
                    // renders after the first main camera which has default value: 0.
                    priority: 1,
                    // set this new camera to render to where the other camera was rendering
                    target: original_target,
                    ..Default::default()
                },
                ..Camera2dBundle::default()
            })
            .insert(post_processing_pass_layer);
    }
}
