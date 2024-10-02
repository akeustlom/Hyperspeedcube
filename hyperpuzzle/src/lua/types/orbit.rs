use hypershape::GeneratorSequence;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use hypermath::pga::Motor;
use hypermath::ApproxHashMap;
use itertools::Itertools;

use crate::lua::lua_warn_fn;
use crate::util::lazy_resolve;

use super::*;

/// Lua orbit object.
#[derive(Debug, Clone)]
pub struct LuaOrbit {
    symmetry: LuaSymmetry,
    init: Vec<Transformable>,

    /// Cosets of the orbit.
    cosets: Vec<OrbitCoset>,

    iter_index: Arc<AtomicUsize>,
}

impl<'lua> FromLua<'lua> for LuaOrbit {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        cast_userdata(lua, &value)
    }
}

impl LuaUserData for LuaOrbit {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_meta_field("type", LuaStaticStr("orbit"));

        fields.add_field_method_get("symmetry", |_lua, this| Ok(this.symmetry.clone()));
        fields.add_field_method_get("init", |lua, this| {
            lua.create_sequence_from(
                this.init
                    .iter()
                    .map(|t| t.into_nillable_lua(lua))
                    .collect::<LuaResult<Vec<_>>>()?,
            )
        });

        fields.add_field_method_get("names", |lua, this| {
            lua.create_sequence_from(this.cosets.iter().map(|elem| elem.name.clone()))
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Len, |_lua, this, ()| Ok(this.cosets.len()));

        methods.add_meta_method(LuaMetaMethod::Call, |lua, this, ()| {
            // Get the index of the Lua iteration.
            let index = this.iter_index.fetch_add(1, Ordering::Relaxed);
            this.get(lua, index)
        });

        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, LuaIndex(index)| {
            Ok(this.get(lua, index)?.get(1).cloned())
        });

        methods.add_method("get", |lua, this, LuaIndex(index)| this.get(lua, index));

        methods.add_method("iter", |_lua, this, ()| {
            Ok(Self {
                iter_index: Arc::new(AtomicUsize::new(0)),
                ..this.clone()
            })
        });

        methods.add_method("named", |lua, this, arg| {
            let Some(names_table) = arg else {
                lua.warning("orbit:named() called with a nil value", false);
                return Ok(this.clone());
            };

            let mut coset_to_name = ApproxHashMap::new();
            for (name, gen_seq) in names_from_table(lua, names_table)? {
                let motor = this.symmetry.motor_for_gen_seq(gen_seq)?;
                let coset = this.init.iter().map(|t| motor.transform(t)).collect_vec();
                if let Some(existing_name) = coset_to_name.insert(coset, name.clone()) {
                    lua.warning(
                        format!("duplicate coset: {name:?} and {existing_name:?}"),
                        false,
                    );
                }
            }

            let mut ret = this.clone();
            for elem in &mut ret.cosets {
                if let Some(name) = coset_to_name.get(&elem.objects) {
                    elem.name = Some(name.clone());
                }
            }
            Ok(ret)
        });

        methods.add_method("prefixed", |_lua, this, prefix: Option<String>| {
            let mut ret = this.clone();
            let Some(prefix) = prefix else {
                return Ok(ret);
            };
            for coset in &mut ret.cosets {
                if let Some(name) = &mut coset.name {
                    *name = format!("{prefix}{name}");
                }
            }
            Ok(ret)
        });

        methods.add_method("intersection", |_lua, this, ()| {
            let mut ret = LuaRegion::Everything;
            for elem in &this.cosets {
                match elem.objects.first() {
                    Some(Transformable::Region(r)) => ret = ret & r.clone(),
                    _ => return Err(LuaError::external("expected orbit of regions")),
                };
            }
            Ok(ret)
        });
        methods.add_method("union", |_lua, this, ()| {
            let mut ret = LuaRegion::Nothing;
            for elem in &this.cosets {
                match elem.objects.first() {
                    Some(Transformable::Region(r)) => ret = ret | r.clone(),
                    _ => return Err(LuaError::external("expected orbit of regions")),
                };
            }
            Ok(ret)
        });
    }
}

impl LuaOrbit {
    /// Returns the orbit of `init` under `symmetry`.
    pub fn new(symmetry: LuaSymmetry, init: Vec<Transformable>) -> Self {
        let orbit_list = symmetry
            .orbit(init.clone())
            .into_iter()
            // Assign empty names.
            .map(|(gen_seq, transform, objects)| OrbitCoset {
                gen_seq,
                transform,
                name: None,
                objects,
            })
            .collect();

        Self {
            symmetry,
            init,

            cosets: orbit_list,

            iter_index: Arc::new(AtomicUsize::new(0)),
        }
    }
    /// Returns the symmetry used to generate the orbit.
    pub fn symmetry(&self) -> &LuaSymmetry {
        &self.symmetry
    }
    /// Returns the initial seed objects that this is the orbit of.
    pub fn init(&self) -> &[Transformable] {
        &self.init
    }

    /// Returns the values for the `index`th element of the orbit.
    pub fn get<'lua>(&self, lua: &'lua Lua, index: usize) -> LuaResult<LuaMultiValue<'lua>> {
        // Return multiple values.
        let mut values = vec![];
        if let Some(element) = self.cosets.get(index) {
            let OrbitCoset {
                gen_seq: _,
                transform,
                name,
                objects,
            } = element;
            // The first value is the transform.
            values.push(LuaTransform(transform.clone()).into_lua(lua)?);
            // Then push the objects.
            for obj in objects {
                values.push(obj.into_nillable_lua(lua)?);
            }
            // Finally push the custom name, if there is one.
            values.push(name.as_deref().into_lua(lua)?);
        }
        Ok(LuaMultiValue::from_vec(values))
    }
}

/// Constructs an assignment of names based on a table for a particular symmetry
/// group.
pub fn names_from_table<'lua>(
    lua: &'lua Lua,
    table: LuaTable<'lua>,
) -> LuaResult<Vec<(String, Vec<u8>)>> {
    let mut key_value_dependencies = vec![];

    for pair in table.pairs() {
        let (k, v) = pair?;
        let (gen_seq, init_name) = gen_seq_and_opt_name_from_value(lua, v)?;
        key_value_dependencies.push((k, (gen_seq, init_name)));
    }

    // Resolve lazy evaluation.
    Ok(lazy_resolve(
        key_value_dependencies,
        |mut seq1, seq2| {
            // TODO: O(n^2)
            seq1.extend(seq2);
            seq1
        },
        lua_warn_fn(lua),
    )
    .into_iter()
    .collect())
}

/// Symmetric set of a particular type of object.
#[derive(Debug, Clone)]
pub enum LuaSymmetricSet<T> {
    /// Single object (using the trivial symmetry).
    Single(T),
    /// Symmetric orbit of an object.
    Orbit(LuaOrbit),
}
impl<'lua, T: LuaTypeName + IntoLua<'lua>> IntoLua<'lua> for LuaSymmetricSet<T> {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match self {
            LuaSymmetricSet::Single(obj) => obj.into_lua(lua),
            LuaSymmetricSet::Orbit(lua_orbit) => lua_orbit.into_lua(lua),
        }
    }
}
impl<'lua, T: LuaTypeName + FromLua<'lua>> FromLua<'lua> for LuaSymmetricSet<T> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        if let Ok(orbit) = <_>::from_lua(value.clone(), lua) {
            Ok(Self::Orbit(orbit))
        } else if let Ok(v) = <_>::from_lua(value.clone(), lua) {
            Ok(Self::Single(v))
        } else {
            // This error isn't quite accurate, but it's close enough. The error
            // message will say that we need a value of type `T`, but in fact we
            // accept an orbit of `T` as well.
            lua_convert_err(&value, T::type_name(lua)?)
        }
    }
}
impl<'lua, T: LuaTypeName + FromLua<'lua> + Clone> LuaSymmetricSet<T> {
    /// Applies a function to each object in the orbit and returns a new orbit.
    pub fn map<U, F>(&self, lua: &'lua Lua, mut f: F) -> LuaResult<LuaSymmetricSet<U>>
    where
        U: Clone + IntoLua<'lua>,
        F: FnMut(GeneratorSequence, Option<String>, T) -> LuaResult<U>,
    {
        match self {
            LuaSymmetricSet::Single(v) => Ok(LuaSymmetricSet::Single(f(
                GeneratorSequence::INIT,
                None,
                v.clone(),
            )?)),
            LuaSymmetricSet::Orbit(orbit) => {
                let orbit_list: Vec<_> = orbit
                    .cosets
                    .iter()
                    .cloned()
                    .map(|element| {
                        let old_value = Self::to_expected_type(lua, element.objects.first())?;
                        let new_value =
                            f(element.gen_seq.clone(), element.name.clone(), old_value)?;

                        // Convert to Lua and then back into a `Transformable`.
                        let lua_value = new_value.into_lua(lua)?;
                        let transformable_new_value = Transformable::from_lua(lua_value, lua)?;

                        LuaResult::Ok(OrbitCoset {
                            gen_seq: element.gen_seq,
                            transform: element.transform,
                            name: element.name,
                            objects: vec![transformable_new_value],
                        })
                    })
                    .try_collect()?;

                let init = orbit_list
                    .get(0)
                    .ok_or_else(|| LuaError::external("empty orbit"))?
                    .objects
                    .iter()
                    .map(|elem| elem.clone().into())
                    .collect();

                Ok(LuaSymmetricSet::Orbit(LuaOrbit {
                    symmetry: orbit.symmetry.clone(),
                    init,
                    cosets: orbit_list,
                    iter_index: Arc::new(AtomicUsize::new(0)),
                }))
            }
        }
    }
    /// Returns a list of all the objects in the orbit.
    pub fn to_vec(&self, lua: &'lua Lua) -> LuaResult<Vec<(GeneratorSequence, Option<String>, T)>> {
        match self {
            LuaSymmetricSet::Single(v) => Ok(vec![(GeneratorSequence::INIT, None, v.clone())]),
            LuaSymmetricSet::Orbit(orbit) => orbit
                .cosets
                .iter()
                .map(|element| {
                    let v = Self::to_expected_type(lua, element.objects.first())?;
                    Ok((element.gen_seq.clone(), element.name.clone(), v))
                })
                .try_collect(),
        }
    }
    /// Returns the initial object from which the others are generated.
    pub fn first(&self, lua: &'lua Lua) -> LuaResult<T> {
        match self {
            LuaSymmetricSet::Single(v) => Ok(v.clone()),
            LuaSymmetricSet::Orbit(orbit) => Self::to_expected_type(lua, orbit.init().first()),
        }
    }

    fn to_expected_type(lua: &'lua Lua, maybe_obj: Option<&Transformable>) -> LuaResult<T> {
        let lua_value = maybe_obj
            .and_then(|obj| obj.into_lua(lua))
            .ok_or_else(|| {
                LuaError::external(format!(
                    "expected orbit of {}",
                    T::type_name(lua).unwrap_or("unknown"),
                ))
            })??;
        T::from_lua(lua_value, lua)
    }
}

/// Coset of an orbit.
#[derive(Debug, Clone)]
struct OrbitCoset {
    gen_seq: GeneratorSequence,
    transform: Motor,
    name: Option<String>,
    objects: Vec<Transformable>,
}

fn gen_seq_and_opt_name_from_value<'lua>(
    lua: &'lua Lua,
    value: LuaValue<'lua>,
) -> LuaResult<(Vec<u8>, Option<String>)> {
    let mut seq: Vec<LuaValue<'_>> = LuaTable::from_lua(value, lua)?
        .sequence_values::<LuaValue<'_>>()
        .try_collect()?;
    let init_name = match seq.last().cloned() {
        Some(LuaValue::String(s)) => {
            seq.pop();
            Some(s.to_string_lossy().to_string())
        }
        _ => None,
    };
    let generator_indices: Vec<u8> = seq
        .into_iter()
        .map(|v| LuaIndex::from_lua(v, lua).map(|LuaIndex(i)| i as u8))
        .try_collect()?;
    Ok((generator_indices, init_name))
}
