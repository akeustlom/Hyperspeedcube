//! Puzzle construction API usable by Rust code.
//!
//! These are all wrapped in `Arc<Mutex<T>>` so that the Lua API can access each
//! independently. These builders are a rare place where we accept mutable
//! aliasing in the Lua API, so the Rust API must also have mutable aliasing.

use std::collections::HashSet;

use eyre::eyre;

mod axis_system;
mod color_system;
mod name;
mod naming_scheme;
mod ordering;
mod puzzle;
mod shape;
mod twist_system;

pub use axis_system::{AxisBuilder, AxisLayerBuilder, AxisSystemBuilder};
pub use color_system::{ColorBuilder, ColorSystemBuilder};
pub use name::NameSet;
pub use naming_scheme::{BadName, Nameable, NamingScheme};
pub use ordering::CustomOrdering;
pub use puzzle::{PieceBuilder, PieceTypeBuilder, PuzzleBuilder};
pub use shape::ShapeBuilder;
pub use twist_system::{TwistBuilder, TwistKey, TwistSystemBuilder};

/// Iterates over elements names in canonical order, assigning unused
/// autogenerated names to unnamed elements.
///
/// The first value in each pair is the name; the second value in each pair is
/// the display name. If no display name is specified, then the canonical name
/// is used instead.
///
/// A warning is emitted if any short or long name is duplicated.
pub fn iter_autonamed<'a, I: hypermath::IndexNewtype>(
    names: &'a NamingScheme<I>,
    order: impl 'a + IntoIterator<Item = I>,
    autonames: impl 'a + IntoIterator<Item = String>,
) -> impl 'a + Iterator<Item = (I, (NameSet, String))> {
    let ids_to_names = names.ids_to_names();
    let ids_to_display_names = names.ids_to_display_names();

    let mut unused_names = autonames
        .into_iter()
        .filter(move |s| !names.names_to_ids().contains_key(s));
    let mut next_unused_name = move || unused_names.next().expect("ran out of names");

    order.into_iter().map(move |id| {
        let name = match ids_to_names.get(&id) {
            Some(s) => s.to_owned(),
            None => NameSet::from(next_unused_name()),
        };
        let display = match ids_to_display_names.get(&id) {
            Some(s) => s.to_owned(),
            None => name.canonical_name().unwrap_or_else(&mut next_unused_name),
        };
        (id, (name, display))
    })
}
