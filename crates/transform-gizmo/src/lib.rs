//! Provides a 3d transformation gizmo that can be used to manipulate 4x4
//! transformation matrices. Such gizmos are commonly used in applications
//! such as game engines and 3d modeling software.
//!
//! # Creating a gizmo
//! For a more complete example, see the online demo at <https://urholaukkarinen.github.io/transform-gizmo/>.
//! The demo sources can be found at <https://github.com/urholaukkarinen/transform-gizmo/blob/main/crates/transform-gizmo-demo/src/main.rs>.
//! ```

pub mod config;
pub mod gizmo;
pub mod math;
mod shape;
mod subgizmo;

pub mod prelude;

pub use config::{GizmoConfig, GizmoDirection, GizmoMode, GizmoOrientation, GizmoVisuals};
pub use gizmo::{Gizmo, GizmoDrawData, GizmoInteraction, GizmoResult};
