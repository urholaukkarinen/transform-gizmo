//! Provides a feature-rich and configurable gizmo that can be used for 3d transformations (translation, rotation, scale).
//!
//! Such gizmos are commonly used in applications such as game engines and 3d modeling software.
//!
//! # Usage
//!
//! If you are using the [Bevy](https://bevyengine.org/) game engine or [Egui](https://github.com/emilk/egui) library in your
//! application, you will most likely want to use [transform-gizmo-bevy](https://docs.rs/transform-gizmo-bevy)
//! or [transform-gizmo-egui](https://docs.rs/transform-gizmo-egui).
//!
//! Alternatively, this library can be easily used with any framework. For interacting with the gizmo,
//! all you will need to do is give [`Gizmo::update`] sufficient
//! information about user interaction, in the form of [`GizmoInteraction`].
//!
//! For rendering the gizmo, [`Gizmo::draw`] provides vertices in viewport coordinates that can be easily rendered
//! with your favorite graphics APIs.
//!
//! For a more complete example, see the online demo at <https://urholaukkarinen.github.io/transform-gizmo/>.
//! The demo sources can be found at <https://github.com/urholaukkarinen/transform-gizmo/blob/main/examples/bevy/src/main.rs>.

mod shape;
mod subgizmo;

pub mod config;
pub mod gizmo;
pub mod math;

pub mod prelude;

pub use prelude::*;
