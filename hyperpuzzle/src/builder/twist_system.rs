use eyre::Result;
use hypermath::collections::approx_hashmap::FloatHash;
use hypermath::collections::{ApproxHashMap, ApproxHashMapKey, IndexOutOfRange};
use hypermath::pga::Motor;
use hypermath::prelude::*;

use super::{AxisSystemBuilder, CustomOrdering};
use crate::builder::NamingScheme;
use crate::puzzle::{Axis, PerTwist, Twist};

/// Twist during puzzle construction.
#[derive(Debug, Clone)]
pub struct TwistBuilder {
    /// Axis that is twisted.
    pub axis: Axis,
    /// Transform to apply to pieces.
    pub transform: Motor,
    /// Value in the quarter-turn metric (or its contextual equivalent).
    pub qtm: usize,
}
impl ApproxHashMapKey for TwistBuilder {
    type Hash = (Axis, <Motor as ApproxHashMapKey>::Hash);

    fn approx_hash(&self, float_hash_fn: impl FnMut(Float) -> FloatHash) -> Self::Hash {
        let Self {
            axis, transform, ..
        } = self;
        (*axis, transform.approx_hash(float_hash_fn))
    }
}

/// Twist system being constructed.
#[derive(Debug)]
pub struct TwistSystemBuilder {
    /// Axis system being constructed.
    pub axes: AxisSystemBuilder,

    /// Twist data (not including name).
    by_id: PerTwist<TwistBuilder>,
    /// Map from twist data to twist ID for each axis.
    ///
    /// Does not include inverses.
    data_to_id: ApproxHashMap<TwistBuilder, Twist>,
    /// User-specified twist names.
    pub names: NamingScheme<Twist>,
}
impl TwistSystemBuilder {
    /// Constructs a empty twist system with a given axis system.
    pub fn new() -> Self {
        Self {
            axes: AxisSystemBuilder::new(),

            by_id: PerTwist::new(),
            data_to_id: ApproxHashMap::new(),
            names: NamingScheme::new(),
        }
    }

    /// Returns the number of twists in the twist system.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns the twist axes in canonical alphabetical order.
    pub fn alphabetized(&self) -> Vec<Twist> {
        let mut ordering =
            CustomOrdering::with_len(self.len()).expect("error constructing default ordering");
        ordering.sort_by_name(self.names.ids_to_names());
        ordering.ids_in_order().to_vec()
    }

    /// Adds a new twist.
    pub fn add(&mut self, data: TwistBuilder) -> Result<Result<Twist, BadTwist>> {
        // Reject the identity twist.
        if data.transform.is_ident() {
            return Ok(Err(BadTwist::Identity));
        }

        // Check that there is not already an identical twist.
        if let Some(&id) = self.data_to_id.get(&data) {
            let name = self.names.get(id).unwrap_or_default();
            return Ok(Err(BadTwist::DuplicateTwist { id, name }));
        }

        let id = self.by_id.push(data.clone())?;
        self.data_to_id.insert(data, id);

        Ok(Ok(id))
    }
    /// Adds a new twist and assigns it a name.
    ///
    /// If the twist is invalid, `warn_fn` is called with info about what went
    /// wrong and no twist is created.
    pub fn add_named(
        &mut self,
        data: TwistBuilder,
        name: String,
        mut warn_fn: impl FnMut(String),
    ) -> Result<Option<Twist>> {
        let id = match self.add(data)? {
            Ok(ok) => ok,
            Err(err) => {
                warn_fn(err.to_string());
                return Ok(None);
            }
        };
        self.names.set(id, Some(name), |e| warn_fn(e.to_string()));
        Ok(Some(id))
    }

    /// Returns a reference to a twist by ID, or an error if the ID is out of
    /// range.
    pub fn get(&self, id: Twist) -> Result<&TwistBuilder, IndexOutOfRange> {
        self.by_id.get(id)
    }

    /// Returns a twist ID from its axis and transform.
    pub fn data_to_id(&self, axis: Axis, transform: &Motor) -> Option<Twist> {
        None.or_else(|| {
            self.data_to_id.get(&TwistBuilder {
                axis,
                transform: transform.clone(),
                qtm: 0, // should be ignored
            })
        })
        .or_else(|| {
            self.data_to_id.get(&TwistBuilder {
                axis,
                transform: transform.canonicalize()?,
                qtm: 0, // should be ignored
            })
        })
        .copied()
    }

    /// Returns the inverse of a twist, or an error if the ID is out of range.
    pub fn inverse(&self, id: Twist) -> Result<Option<Twist>, IndexOutOfRange> {
        let twist = self.get(id)?;
        let rev_transform = twist.transform.reverse();
        Ok(self.data_to_id(twist.axis, &rev_transform))
    }
}

/// Error indicating a bad twist.
#[derive(thiserror::Error, Debug, Clone)]
#[allow(missing_docs)]
pub enum BadTwist {
    #[error("twist transform cannot be identity")]
    Identity,
    #[error("identical twist already exists with ID {id} and name {name:?}")]
    DuplicateTwist { id: Twist, name: String },
    #[error("bad twist transform")]
    BadTransform,
}
