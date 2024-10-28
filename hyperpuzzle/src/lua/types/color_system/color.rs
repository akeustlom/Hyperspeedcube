use hypermath::pga::Motor;

use super::*;
use crate::builder::PuzzleBuilder;
use crate::puzzle::Color;

/// Lua handle to a color in the color system of a shape under construction.
pub type LuaColor = LuaDbEntry<Color, PuzzleBuilder>;

impl LuaUserData for LuaColor {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field("type", LuaStaticStr("color"));

        LuaNamedIdDatabase::add_named_db_entry_fields(fields);
        LuaOrderedIdDatabase::add_ordered_db_entry_fields(fields);

        fields.add_field_method_get("default", |_lua, this| {
            let puz = this.db.lock();
            let colors = &puz.shape.colors;
            Ok(colors
                .get_default_color(this.id)
                .map(|default_color| default_color.to_string()))
        });
        fields.add_field_method_set("default", |lua, this, new_default_color| {
            let mut puz = this.db.lock();
            let colors = &mut puz.shape.colors;
            colors.set_default_color(
                this.id,
                super::default_color_from_str(lua, new_default_color),
            );
            Ok(())
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::ToString, |_lua, this, ()| {
            if let Some(name) = this.db.lock().names().get(this.id) {
                Ok(format!("color({name:?})"))
            } else {
                Ok(format!("color({})", this.id))
            }
        });

        methods.add_meta_method(LuaMetaMethod::Eq, |_lua, lhs, rhs: Self| Ok(lhs == &rhs));
    }
}

impl LuaColor {
    /// Returns the color generated by a similar cut as this color, but
    /// transformed by `t`.
    pub fn transform(&self, _t: &Motor) -> LuaResult<Option<Self>> {
        // TODO: record color relations
        Err(LuaError::external(
            "transforming colors is not yet supported",
        ))
    }
}
