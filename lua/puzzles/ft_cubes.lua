local utils = lib.utils

local REALISITIC_PROPORTIONS = true
CORNER_STALK_SIZE = 0.1

local function ft_cube_cut_depths(ndim, layers)
  if layers < 2 then return end

  local outermost_cut
  local aesthetic_limit = 1 - 2/layers
  local mechanical_limit = 0
  if REALISITIC_PROPORTIONS then
    mechanical_limit = 1 / sqrt(ndim-1)
  end
  outermost_cut = min(aesthetic_limit, mechanical_limit - CORNER_STALK_SIZE)
  return utils.layers.inclusive_inf(outermost_cut, -outermost_cut, layers)
end

GIZMO_EDGE_FACTOR = 0.8

-- NxNxN Face-Turning Cube generator
puzzle_generators:add{
  id = 'ft_cube',
  version = '1.0.0',
  name = "NxNxN Face-Turning Cube",
  aliases = {"N^3"},
  colors = 'cube',
  params = {
    { name = "Layers", type = 'int', default = 3, min = 1, max = 49 },
  },
  gen = function(params)
    local size = params[1]
    if size == 1 then return 'cube' end
    return {
      name = size .. "x" .. size .. "x" .. size,
      aliases = { size .. "^" .. 3 },
      ndim = 3,
      build = function(self)
        local shape = lib.symmetries.bc3.cube()
        local cut_depths = ft_cube_cut_depths(3, size)
        local colors, axes = utils.cut_ft_shape(self, shape, cut_depths)

        if size == 1 then
          lib.piece_types.mark_everything_core(self)
          return
        end

        -- Define twists
        for t, ax, rot in shape.sym.chiral:orbit(axes[1], shape.sym:thru(2, 1)) do
          self.twists:add(ax, rot, { gizmo_pole_distance = 1 })
        end

        -- Mark piece types
        lib.piece_types.triacron_subsets.mark_multilayer_UFRL(self, size)
        self:unify_piece_types(shape.sym.chiral) -- chiral because left vs. right obliques
      end,

      tags = {
        'type/puzzle',
        completeness = {
          laminated = size <= 2,
          real = size <= 3,
          super = size <= 2,
        },
        ['cuts/depth/deep/to_adjacent'] = size % 2 == 0,
        ['cuts/depth/half'] = size % 2 == 0,
        ['external/leaderboard'] = size >= 2,
      },
    }
  end,

  examples = {
    {
      params = {2},
      aliases = { "Pocket Cube" },
      tags = {
        external = { gelatinbrain = '3.1.1', museum = 20, wca = '222' },
        inventor = "Ernő Rubik",
      }
    },
    {
      params = {3},
      aliases = { "Rubik's Cube" },
      tags = {
        'canonical',
        external = { gelatinbrain = '3.1.2', museum = 7629, wca = '333' },
        inventor = "Ernő Rubik",
      },
    },
    {
      params = {4},
      aliases = { "Rubik's Revenge" },
      tags = {
        external = { gelatinbrain = '3.1.3', museum = 265, wca = '444' },
        inventor = "Peter Sebesteny",
      },
    },
    {
      params = {5},
      aliases = { "Professor's Cube" },
      tags = {
        external = { gelatinbrain = '3.1.4', museum = 6106, wca = '555' },
        inventor = "Jürgen Hoffmann",
      },
    },
    {
      params = {6},
      tags = {
        external = { museum = 3931, wca = '666' },
        inventor = "Daniel Tseng",
      },
    },
    {
      params = {7},
      tags = {
        external = { museum = 1486, wca = '777' },
        inventor = "Panagiotis Verdes",
      },
    },
  },

  tags = {
    builtin = '2.0.0',
    external = { '!gelatinbrain', '!hof', '!mc4d', '!museum', '!wca' },

    author = { "Andrew Farkas", "Milo Jacquet" },
    '!inventor',

    'shape/3d/platonic/cube',
    algebraic = {
      'doctrinaire', 'pseudo/doctrinaire',
      '!abelian', '!fused', '!orientations/non_abelian', '!trivial', '!weird_orbits',
    },
    axes = { '3d/elementary/cubic', '!hybrid', '!multicore' },
    colors = { '!multi_facet_per', '!multi_per_facet' },
    completeness = { '!complex' },
    cuts = { depth = { 'shallow' }, '!stored', '!wedge' },
    turns_by = { 'face', 'facet' },
    '!experimental',
    '!canonical',
    '!family',
    '!variant',
    '!meme',
    '!shapeshifting',
  },
}

-- NxNxNxN Face-Turning Hypercube generator
puzzle_generators:add{
  id = 'ft_hypercube',
  version = '1.0.0',
  name = "NxNxNxN Face-Turning Hypercube",
  aliases = {"N^4"},
  colors = 'hypercube',
  params = {
    { name = "Layers", type = 'int', default = 3, min = 1, max = 13 },
  },
  gen = function(params)
    local size = params[1]
    if size == 1 then return 'hypercube' end
    return {
      name = size .. "x" .. size .. "x" .. size .. "x" .. size,
      aliases = { size .. "^" .. 4 },
      ndim = 4,
      build = function(self)
        local sym = cd'bc4'
        local shape = lib.symmetries.bc4.hypercube()
        self:carve(shape:iter_poles())

        if size == 1 then
          lib.piece_types.mark_everything_core(self)
          return
        end

        -- Define axes and slices
        self.axes:add(shape:iter_poles(), ft_cube_cut_depths(4, size))

        -- Define twists
        local a1 = self.axes[sym.ooox.unit]
        local a2 = sym:thru(4):transform(a1)
        local a3 = sym:thru(3):transform(a2)
        local a4 = sym:thru(2):transform(a3)
        local t = sym:thru(2, 1)
        for _, axis1, axis2, twist_transform in sym.chiral:orbit(a1, a2, t) do
          self.twists:add(axis1, twist_transform, {
            name = axis2,
            gizmo_pole_distance = 1,
          })
        end

        local ridge = a2.vector + a3.vector -- ridge orthogonal to `a1`
        local init_transform = sym:thru(3, 1) -- rot{fix = a1.vector ^ ridge, angle = PI}
        for t, axis1, _ridge, twist_transform in sym.chiral:orbit(a1, ridge, init_transform) do
          self.twists:add(axis1, twist_transform, {
            name = names.set(t:transform(a2), t:transform(a3)),
            gizmo_pole_distance = (1 + GIZMO_EDGE_FACTOR) / sqrt(2),
          })
        end

        local edge = ridge + a4.vector -- edge orthogonal to `a1`
        local init_transform = sym:thru(3, 2)
        for t, axis1, _edge, twist_transform in sym.chiral:orbit(a1, edge, init_transform) do
          self.twists:add(axis1, twist_transform, {
            name = names.set(t:transform(a2), t:transform(a3), t:transform(a4)),
            gizmo_pole_distance = (1 + 2 * GIZMO_EDGE_FACTOR) / sqrt(3),
          })
        end

        -- Mark piece types
        lib.piece_types.tetrahedracron_subsets.mark_multilayer_UFRLIO(self, size)
        self:unify_piece_types(sym.chiral) -- chiral because left vs. right obliques
      end,

      tags = {
        'type/puzzle',
        completeness = {
          laminated = size <= 2,
          real = size <= 3,
          super = size <= 2,
        },
        ['cuts/depth/deep/to_adjacent'] = size % 2 == 0,
        ['cuts/depth/half'] = size % 2 == 0,
        ['external/leaderboard'] = size >= 2,
      },
    }
  end,

  examples = {
    { params = {2}, tags = { external = { gelatinbrain = '8.1.1' } } },
    { params = {3}, tags = { 'canonical' } },
    { params = {4} },
    { params = {5} },
    { params = {6} },
    { params = {7} },
  },

  tags = {
    builtin = '2.0.0',
    external = { '!gelatinbrain', '!hof', 'mc4d', '!museum', '!wca' },

    author = { "Andrew Farkas", "Milo Jacquet" },
    '!inventor',

    'shape/4d/platonic/hypercube',
    algebraic = {
      'doctrinaire', 'pseudo/doctrinaire',
      '!abelian', '!fused', 'orientations/non_abelian', '!trivial', '!weird_orbits',
    },
    axes = { '4d/elementary/hypercubic', '!hybrid', '!multicore' },
    colors = { '!multi_facet_per', '!multi_per_facet' },
    completeness = { '!complex' },
    cuts = { depth = { 'shallow' }, '!stored', '!wedge' },
    turns_by = { 'cell', 'facet' },
    '!experimental',
    '!family',
    '!variant',
    '!meme',
    '!shapeshifting',
  },
}

-- NxNxNxNxN Face-Turning Hypercube generator
puzzle_generators:add{
  id = 'ft_5_cube',
  version = '1.0.0',
  name = "NxNxNxNxN Face-Turning 5-Cube",
  aliases = {"N^5"},
  colors = '5_cube',
  params = {
    { name = "Layers", type = 'int', default = 3, min = 1, max = 7 },
  },
  gen = function(params)
    local size = params[1]
    if size == 1 then return '5_cube' end
    return {
      name = size .. "x" .. size .. "x" .. size .. "x" .. size .. "x" .. size,
      aliases = { size .. "^" .. 5 },
      ndim = 5,
      build = function(self)
        local sym = cd'bc5'
        local shape = lib.symmetries.bc5.hypercube()
        self:carve(shape:iter_poles())

        if size == 1 then
          lib.piece_types.mark_everything_core(self)
          return
        end

        -- Define axes and slices
        self.axes:add(shape:iter_poles(), ft_cube_cut_depths(5, size))

        -- Define twists
        local a1 = self.axes[sym.oooox.unit]
        local a2 = sym:thru(5):transform(a1)
        local a3 = sym:thru(4):transform(a2)
        local t = rot{fix = a1.vector, from = a2.vector, to = a3.vector}
        for dim1 in ('xyzwv'):gmatch('.') do
          for dim2 in ('xyzwv'):gmatch('.') do
            if dim1 ~= dim2 then
              for _, ax in ipairs(self.axes) do
                if vec(dim1) ~= ax.vector and -vec(dim1) ~= ax.vector and vec(dim2) ~= ax.vector and -vec(dim2) ~= ax.vector then
                  self.twists:add(ax, rot{from = dim1, to = dim2}, {
                    name = dim1 .. dim2,
                  })
                end
              end
            end
          end
        end

        -- Mark piece types
        if size == 3 then
          lib.utils.unpack_named(_ENV, self.axes)
          self:mark_piece(A(1) & O(1) & F(1) & U(1) & R(1), 'corner', "Corner")
          self:mark_piece(A(2) & O(1) & F(1) & U(1) & R(1), 'edge', "Edge")
          self:mark_piece(A(2) & O(2) & F(1) & U(1) & R(1), 'peak', "Peak")
          self:mark_piece(A(2) & O(2) & F(2) & U(1) & R(1), 'ridge', "Ridge")
          self:mark_piece(A(2) & O(2) & F(2) & U(2) & R(1), 'center', "Center")
        end
        -- lib.piece_types.pentachoracron_subsets.mark_multilayer_UFRLIOAP(self, size)
        self:unify_piece_types(sym.chiral) -- chiral because left vs. right obliques
      end,
    }
  end,

  examples = {
    { params = {2} },
    { params = {3}, tags = { 'canonical' } },
    { params = {4} },
    { params = {5} },
  },


  tags = {
  },
}

-- N^D Face-Turning Hypercube generator
puzzle_generators:add{
  id = 'ft_nd_hypercube',
  version = '1.0.0',
  name = "N^D Facet-Turning Hypercube",
  params = {
    { name = "Layers", type = 'int', default = 3, min = 1, max = 13 },
    { name = "Dimensions", type = 'int', default = 3, min = 2, max = 7 },
  },
  gen = function(params)
    local size = params[1]
    local ndim = params[2]

    local name = size
    for i = 2, ndim do
      name = name .. "x" .. size
    end

    return {
      name = name,
      aliases = { size .. "^" .. ndim },
      ndim = ndim,
      build = function(self)
        local sym = cd('bc' .. ndim)
        local facet_pole = sym:vec{[ndim] = 1}.unit
        self:carve(sym:orbit(facet_pole))

        if size > 1 then
          local cut_depths = ft_cube_cut_depths(ndim, size)
          self.axes:add(sym:orbit(facet_pole), cut_depths)
        end
      end,

      tags = {
        ['type/shape'] = size == 1,
        ['type/puzzle'] = size ~= 1,
        algebraic = {
          abelian = size == 1,
          trivial = size == 1,
        },
        canonical = size == 3,
        completeness = {
          complex = size == 1,
          laminated = size <= 2,
          real = size <= 3,
          super = size <= 2,
        },
        ['cuts/depth/deep/to_adjacent'] = size % 2 == 0,
        ['cuts/depth/half'] = size % 2 == 0,
      },
    }
  end,

  tags = {
    builtin = '2.0.0',
    external = { '!gelatinbrain', '!hof', '!mc4d', '!museum', '!wca' },

    author = { "Andrew Farkas" },
    '!inventor',

    algebraic = {
      'doctrinaire', 'pseudo/doctrinaire',
      '!fused', 'orientations/non_abelian', '!trivial', '!weird_orbits',
    },
    axes = { '!hybrid', '!multicore' },
    colors = { '!multi_facet_per', '!multi_per_facet' },
    cuts = { depth = { 'shallow' }, '!stored', '!wedge' },
    turns_by = { 'facet' },
    'experimental',
    '!canonical',
    '!family',
    '!variant',
    '!meme',
    '!shapeshifting',
  },
}
