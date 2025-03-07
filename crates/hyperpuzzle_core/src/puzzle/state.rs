use std::sync::Arc;

use eyre::{OptionExt, Result};
use hypermath::collections::GenericVec;
use hypermath::idx_struct;
use hypermath::prelude::*;
use itertools::Itertools;
use parking_lot::Mutex;

use crate::{Axis, AxisInfo, LayerMask, LayeredTwist, PerAxis, PerPiece, Piece, PieceMask, Puzzle};

type PerCachedTransform<T> = GenericVec<CachedTransform, T>;
idx_struct! {
    struct CachedTransform(usize);
}

// TODO: reconsider this
#[derive(Debug)]
struct CachedTransformData {
    pub motor: pga::Motor,
    pub rev_motor: pga::Motor,
    pub transformed_vectors: PerAxis<Option<Vector>>,
}
impl CachedTransformData {
    fn new(motor: pga::Motor, axes: &PerAxis<AxisInfo>) -> Self {
        let transformed_vectors = axes.map_ref(|_, _| None);
        let rev_motor = motor.reverse();
        Self {
            motor,
            rev_motor,
            transformed_vectors,
        }
    }
    fn reverse_transform_axis_vector(&mut self, axis: Axis, axes: &PerAxis<AxisInfo>) -> &Vector {
        self.transformed_vectors[axis]
            .get_or_insert_with(|| self.rev_motor.transform_vector(&axes[axis].vector))
    }
}

/// Instance of a puzzle with a particular state.
#[derive(Debug, Clone)]
pub struct PuzzleState {
    /// Immutable puzzle type info.
    puzzle_type: Arc<Puzzle>,
    /// Attitude (position & rotation) of each piece.
    piece_transforms: PerPiece<CachedTransform>,
    /// Cached set of possible attitudes of pieces.
    cached_transforms: Arc<Mutex<PerCachedTransform<CachedTransformData>>>,
    cached_transform_by_motor: Arc<Mutex<ApproxHashMap<pga::Motor, CachedTransform>>>,
}
impl PuzzleState {
    /// Constructs a new instance of a puzzle.
    pub fn new(puzzle_type: Arc<Puzzle>) -> Self {
        let ident = pga::Motor::ident(puzzle_type.ndim());
        let piece_transforms = puzzle_type.pieces.map_ref(|_, _| CachedTransform(0));

        let cached_transforms = Arc::new(Mutex::new(PerCachedTransform::from_iter([
            CachedTransformData::new(ident.clone(), &puzzle_type.axes),
        ])));

        let mut by_motor = ApproxHashMap::new();
        by_motor.insert(ident, CachedTransform(0));
        let cached_transform_by_motor = Arc::new(Mutex::new(by_motor));

        PuzzleState {
            puzzle_type,
            piece_transforms,
            cached_transforms,
            cached_transform_by_motor,
        }
    }
    /// Returns the puzzle type
    pub fn ty(&self) -> &Arc<Puzzle> {
        &self.puzzle_type
    }
    /// Returns the position and rotation of each piece.
    pub fn piece_transforms(&self) -> PerPiece<pga::Motor> {
        let cached = self.cached_transforms.lock();
        self.piece_transforms
            .map_ref(|_, &i| cached[i].motor.clone())
    }
    /// Returns the position and rotation of each piece during an arbitrary
    /// animation affecting a subset of pieces.
    pub fn partial_piece_transforms(
        &self,
        grip: &PieceMask,
        transform: &pga::Motor,
    ) -> PerPiece<pga::Motor> {
        self.piece_transforms()
            .map(|piece, static_transform| match grip.contains(piece) {
                true => transform * static_transform,
                _ => static_transform.clone(),
            })
    }

    /// Does a twist, or returns an error containing the set of pieces that
    /// prevented the twist.
    pub fn do_twist(&self, twist: LayeredTwist) -> Result<Self, Vec<Piece>> {
        let twist_info = &self.puzzle_type.twists[twist.transform];
        let grip = self.compute_grip(twist_info.axis, twist.layers);

        // Check for split pieces, which prevent the turn.
        let split_pieces = grip
            .iter_filter(|_piece, &which_side| which_side == WhichSide::Split)
            .collect_vec();
        if !split_pieces.is_empty() {
            return Err(split_pieces);
        }

        let mut cached_transforms = self.cached_transforms.lock();
        let mut cached_transform_by_motor = self.cached_transform_by_motor.lock();

        let piece_transforms = self.piece_transforms.map_ref(|piece, &piece_transform| {
            if grip[piece] == WhichSide::Inside {
                let current_motor = &cached_transforms[piece_transform].motor;
                let new_motor = &twist_info.transform * current_motor;
                *cached_transform_by_motor
                    .entry(new_motor.clone())
                    .or_insert_with(|| {
                        cached_transforms
                            .push(CachedTransformData::new(new_motor, &self.puzzle_type.axes))
                            .expect("out of memory")
                    })
            } else {
                piece_transform
            }
        });

        Ok(Self {
            puzzle_type: Arc::clone(&self.puzzle_type),
            piece_transforms,
            cached_transforms: Arc::clone(&self.cached_transforms),
            cached_transform_by_motor: Arc::clone(&self.cached_transform_by_motor),
        })
    }

    /// Returns the set of pieces on the inside of a grip (axis + layer mask).
    /// This considers blocking pieces to be outside the grip; use
    /// `compute_grip()` to see which pieces are blocking a twist.
    pub fn compute_gripped_pieces(&self, axis: Axis, layers: LayerMask) -> PieceMask {
        PieceMask::from_iter(
            self.puzzle_type.pieces.len(),
            self.compute_grip(axis, layers)
                .iter_filter(|_, &status| status == WhichSide::Inside),
        )
    }

    /// Returns each piece's location with respect to a grip (axis + layer
    /// mask). A piece may be inside the grip, outside the grip, or blocking the
    /// grip. [`WhichSide::Flush`] is not used.
    pub fn compute_grip(&self, axis: Axis, layers: LayerMask) -> PerPiece<WhichSide> {
        let Ok(axis_info) = self.puzzle_type.axes.get(axis) else {
            log::error!("bad axis ID");
            return self.puzzle_type.pieces.map_ref(|_, _| WhichSide::Split);
        };

        let grip_layers = layers
            .iter()
            .filter_map(|layer| Some((layer, axis_info.layers.0.get(layer).ok()?)))
            .collect_vec();

        let mut segments: Vec<(Float, Float)> = vec![];
        for (_layer, layer_info) in grip_layers {
            if let Some((_prev_top, prev_bottom)) = segments.last_mut() {
                if approx_eq(&layer_info.top, prev_bottom) {
                    *prev_bottom = layer_info.bottom;
                    continue;
                }
            }
            segments.push((layer_info.top, layer_info.bottom));
        }

        self.puzzle_type.pieces.map_ref(|piece, _piece_info| {
            let (piece_bottom, piece_top) = match self.piece_min_max_on_axis(piece, axis) {
                Ok((min, max)) => (min, max),
                Err(e) => {
                    log::error!("{e}");
                    return WhichSide::Split;
                }
            };
            for (segment_top, segment_bottom) in &segments {
                if approx_lt_eq(segment_bottom, &piece_bottom)
                    && approx_lt_eq(&piece_top, segment_top)
                {
                    // piece is completely inside the layer segment
                    return WhichSide::Inside;
                } else if approx_lt_eq(segment_top, &piece_bottom)
                    || approx_lt_eq(&piece_top, segment_bottom)
                {
                    // piece is completely outside the layer segment
                    continue;
                } else {
                    // piece is partly inside and partly outside the layer segment
                    return WhichSide::Split;
                }
            }
            // if not inside any segment, it's outside
            WhichSide::Outside
        })
    }

    /// Returns the smallest layer mask on `axis` that contains `piece`.
    pub fn min_layer_mask(&self, axis: Axis, piece: Piece) -> Option<LayerMask> {
        let (piece_bottom, piece_top) = self.piece_min_max_on_axis(piece, axis).ok()?;
        let axis_info = self.puzzle_type.axes.get(axis).ok()?;
        axis_info.layers.contiguous_range(piece_bottom, piece_top)
    }
    /// Returns the smallest unblocked layer mask on `axis` that contains
    /// `piece`.
    pub fn min_drag_layer_mask(&self, axis: Axis, piece: Piece) -> Option<LayerMask> {
        let ty = self.ty();
        let axis_info = self.puzzle_type.axes.get(axis).ok()?;

        let mut floats = axis_info
            .layers
            .0
            .iter_values()
            .flat_map(|layer_info| [layer_info.top, layer_info.bottom])
            .collect_vec();
        floats.insert(0, Float::INFINITY);
        floats.push(-Float::INFINITY);
        let mut i = 0;
        while i < floats.len() - 1 {
            if approx_eq(&floats[i], &floats[i + 1]) {
                floats.remove(i);
            }
            i += 1;
        }

        let mut min_of_all_pieces = Float::INFINITY;
        let mut max_of_all_pieces = -Float::INFINITY;

        for p in ty.pieces.iter_keys() {
            let (min, max) = self.piece_min_max_on_axis(p, axis).ok()?;
            min_of_all_pieces = Float::min(min_of_all_pieces, min);
            max_of_all_pieces = Float::max(max_of_all_pieces, max);
            floats.retain(|f| approx_lt_eq(f, &min) || approx_lt_eq(&max, f));
        }

        let (min, max) = self.piece_min_max_on_axis(piece, axis).ok()?;
        let lo = *floats.iter().find(|&f| approx_lt_eq(f, &min))?;
        let hi = *floats.iter().rfind(|&f| approx_lt_eq(&max, f))?;

        // This includes all pieces
        if approx_lt_eq(&lo, &min_of_all_pieces) && approx_lt_eq(&max_of_all_pieces, &hi) {
            return None;
        }

        axis_info.layers.contiguous_range(lo, hi)
    }

    /// Returns the minimum and maximum coordinates along an axis that a piece's
    /// vertices spans.
    fn piece_min_max_on_axis(&self, piece: Piece, axis: Axis) -> Result<(Float, Float)> {
        let mut cached_transforms = self.cached_transforms.lock();
        let transformed_axis_vector = cached_transforms[self.piece_transforms[piece]]
            .reverse_transform_axis_vector(axis, &self.puzzle_type.axes);

        let space = &self.puzzle_type.space;
        let piece_info = &self.puzzle_type.pieces[piece];
        let vertex_set = space.get(piece_info.polytope).vertex_set();
        let vertex_distances_along_axis = vertex_set.map(|p| p.pos().dot(transformed_axis_vector));
        hypermath::util::min_max(vertex_distances_along_axis).ok_or_eyre("piece has no vertices")
    }

    /// Returns whether the puzzle is in a solved state.
    pub fn is_solved(&self) -> bool {
        let piece_transforms = self.piece_transforms();

        // Each color may appear on at most one set of parallel planes. Track
        // that normal vector.
        let mut color_normals = self.ty().colors.list.map_ref(|_, _| None);

        self.ty().stickers.iter().all(|(_, sticker_info)| {
            let sticker_transform = &piece_transforms[sticker_info.piece];
            let normal_vector = sticker_transform.transform_vector(sticker_info.plane.normal());
            match color_normals.get_mut(sticker_info.color) {
                Ok(Some(color_vector)) => approx_eq(color_vector, &normal_vector),
                Ok(opt_color_plane @ None) => {
                    *opt_color_plane = Some(normal_vector);
                    true
                }
                Err(_) => {
                    log::error!("unknown color encountered during solved state detection");
                    false
                }
            }
        })
    }
}
