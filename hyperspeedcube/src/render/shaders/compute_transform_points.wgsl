struct ProjectionParams {
    sticker_shrink: f32,
    facet_shrink: f32,
    piece_explode: f32,

    w_factor_4d: f32,
    w_factor_3d: f32,
    fov_signum: f32,
}

struct LightingParams {
    dir: vec3<f32>,
    ambient: f32,
    directional: f32,
}

// When compiling the shader in Rust, we will fill in the number of dimensions.
const NDIM: i32 = {{ndim}};

/*
 * VIEW PARAMETERS AND TRANSFORMS
 */
@group(0) @binding(0) var<uniform> projection_params: ProjectionParams;
@group(0) @binding(1) var<uniform> lighting_params: LightingParams;
@group(0) @binding(2) var<storage, read> puzzle_transform: array<f32>;
@group(0) @binding(3) var<storage, read> piece_transforms: array<f32>;

/*
 * STATIC MESH DATA (per-vertex)
 */
@group(1) @binding(0) var<storage, read> vertex_positions: array<f32>;
@group(1) @binding(1) var<storage, read> u_tangents: array<f32>;
@group(1) @binding(2) var<storage, read> v_tangents: array<f32>;
@group(1) @binding(3) var<storage, read> sticker_shrink_vectors: array<f32>;
@group(1) @binding(4) var<storage, read> facet_ids: array<i32>;
@group(1) @binding(5) var<storage, read> piece_ids: array<i32>;

/*
 * STATIC MESH DATA (other)
 */
@group(2) @binding(0) var<storage, read> facet_centroids: array<f32>;
@group(2) @binding(1) var<storage, read> piece_centroids: array<f32>;

/*
 * OUTPUT (per-vertex)
 */
@group(3) @binding(0) var<storage, read_write> vertex_3d_positions: array<vec4<f32>>;
@group(3) @binding(1) var<storage, read_write> vertex_lightings: array<f32>;

/*
 * COMPUTE OFFSETS
 */
var<push_constant> offset: u32;

// When compiling the shader in Rust, we will fill in the workgroup size.
@compute
@workgroup_size({{workgroup_size}})
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = i32(arrayLength(&vertex_3d_positions));
    let index = i32(global_invocation_id.x + offset);
    if (index >= total) {
        return;
    }

    let facet: i32 = facet_ids[index];
    let piece: i32 = piece_ids[index];

    let base_idx = NDIM * index;

    // var new_pos = array<f32, NDIM>();
    var vert_idx = base_idx;
    // var facet_centroid_idx = NDIM * facet;
    // var piece_centroid_idx = NDIM * piece;
    // for (var i = 0; i < NDIM; i++) {
    //     new_pos[i] = vertex_positions[vert_idx];
    //     // Apply sticker shrink.
    //     new_pos[i] += sticker_shrink_vectors[vert_idx] * projection_params.sticker_shrink;
    //     // Apply facet shrink.
    //     new_pos[i] -= facet_centroids[facet_centroid_idx];
    //     new_pos[i] *= 1.0 - projection_params.facet_shrink;
    //     new_pos[i] += facet_centroids[facet_centroid_idx];
    //     // Apply piece explode.
    //     new_pos[i] += piece_centroids[piece_centroid_idx] * projection_params.piece_explode;

    //     vert_idx++;
    //     facet_centroid_idx++;
    //     piece_centroid_idx++;
    // }
    // var old_pos = new_pos;

    // // Apply piece transform.
    // new_pos = array<f32, NDIM>();
    // var new_u = array<f32, NDIM>();
    // var new_v = array<f32, NDIM>();
    // vert_idx = base_idx;
    // var i: i32 = NDIM * NDIM * piece;
    // for (var col = 0; col < NDIM; col++) {
    //     for (var row = 0; row < NDIM; row++) {
    //         new_pos[row] += piece_transforms[i] * old_pos[col];
    //         new_u[row] += piece_transforms[i] * u_tangents[vert_idx];
    //         new_v[row] += piece_transforms[i] * v_tangents[vert_idx];
    //         i++;
    //     }
    //     vert_idx++;
    // }
    // old_pos = new_pos;
    // var old_u = new_u;
    // var old_v = new_v;

    // TODO: REMOVE THIS
    var old_pos = array<f32, NDIM>();
    for (var i = 0; i < NDIM; i++) {
        old_pos[i] = vertex_positions[vert_idx];
        vert_idx++;
    }
    var old_u = array<f32, NDIM>();
    var old_v = array<f32, NDIM>();
    var i = 0;

    // Apply puzzle transformation and collapse to 4D.
    var point_4d = vec4<f32>();
    var u = vec4<f32>();
    var v = vec4<f32>();
    i = 0;
    for (var col = 0; col < NDIM; col++) {
        // TODO: optimize this
        for (var row = 0; row < NDIM; row++) {
            if (row < NDIM) {
                point_4d[row] += puzzle_transform[i] * old_pos[col];
                u[row] += puzzle_transform[i] * old_u[col];
                v[row] += puzzle_transform[i] * old_v[col];
                i++;
            }
        }
    }

    var x = point_4d.x;
    var y = point_4d.y;
    var z = point_4d.z;
    // var w = point_4d.w;
    var w = 1.0;

    // Apply 4D perspective transformation.
    let w_divisor = 1.0 + w * projection_params.w_factor_4d;
    x = x / w_divisor;
    y = y / w_divisor;
    z = z / w_divisor;

    // Apply 3D perspective transformation.
    let z_divisor = 1.0 + (projection_params.fov_signum - z) * projection_params.w_factor_3d;
    w = z_divisor;

    vertex_3d_positions[index] = vec4(x, y, z, w);

    // TODO: Compute lighting.
    vertex_lightings[index] = 1.0;
    // let u = new_u.xyz / new_u.w;
    // let v = new_v.xyz / new_v.w;
    // let normal = cross(u, v);
    // // Apply 4D perspective transformation to figure out which side of the
    // // surface is visible.
    // let projected_normal = normal
    // normal *= normal.z
    // let lighting =
}
