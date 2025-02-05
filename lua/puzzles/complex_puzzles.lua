local utils = lib.utils
local linear = lib.symmetries.linear
local polygonal = lib.symmetries.polygonal
local prisms = lib.puzzles.prisms -- Helpers to integrate triangular prism; should i pull from puzzles?

function piecewarning()
    print("warning(s) intentional, some pieces are disjoint")
end

puzzles:add{
  id = 'complex_3x3x3',
  name = "Complex 3x3x3",
  version = '1.0.2',
  ndim = 3,
  colors = 'cube',
  remove_internals = false,
  build = function(self)
    local sym = cd'bc3'
    local shape = lib.symmetries.bc3.cube()
    self:carve(shape:iter_poles())

    -- Define axes and slices
    self.axes:add(shape:iter_poles(), {3/5, -1/5})

    -- Define twists
    for _, axis, twist_transform in sym.chiral:orbit(self.axes[sym.oox.unit], sym:thru(2, 1)) do
      self.twists:add(axis, twist_transform, {gizmo_pole_distance = 1})
    end

    --Give axes labels for filters, twists, and to simplify piece filters
    utils.unpack_named(_ENV, self.axes)

    -- Add super-stickers on internal faces
    for i=3,-3,-2 do
        self:slice(plane(vec('x'), i/5), {stickers=self.colors.R})
        self:slice(plane(vec('x')*-1, i/5), {stickers=self.colors.L})
        self:slice(plane(vec('y'), i/5), {stickers=self.colors.U})
        self:slice(plane(vec('y')*-1, i/5), {stickers=self.colors.D})
        self:slice(plane(vec('z'), i/5), {stickers=self.colors.F})
        self:slice(plane(vec('z')*-1, i/5), {stickers=self.colors.B})
    end

    -- Mark one copy of each piece-type
    self:mark_piece(~R(1) & ~L(1) & ~U(1) & ~D(1) & ~F(1) & ~B(1), 'core', "Core")
    self:mark_piece(R(1) & ~L(1) & ~U(1) & ~D(1) & ~F(1) & ~B(1), 'center', "Center")
    self:mark_piece(R(1) & ~L(1) & U(1) & ~D(1) & ~F(1) & ~B(1), 'edge', "Edge")
    self:mark_piece(R(1) & L(1) & ~U(1) & ~D(1) & ~F(1) & ~B(1), 'axle', "Axle")
    self:mark_piece(R(1) & ~L(1) & U(1) & ~D(1) & F(1) & ~B(1), 'corner', "Corner")
    self:mark_piece(R(1) & L(1) & U(1) & ~D(1) & ~F(1) & ~B(1), 'triwall', "Triwall")
    self:mark_piece(R(1) & L(1) & U(1) & ~D(1) & F(1) & ~B(1), 'antiedge', "Anti-Edge")
    self:mark_piece(R(1) & L(1) & U(1) & D(1) & ~F(1) & ~B(1), 'antiaxle', "Anti-Axle")
    self:mark_piece(R(1) & L(1) & U(1) & ~D(1) & F(1) & B(1), 'anticenter', "Anti-Center")
    self:mark_piece(R(1) & L(1) & U(1) & D(1) & F(1) & B(1), 'anticore', "Anti-Core")
    piecewarning()

    -- Pattern piece-types around the puzzle
    self:unify_piece_types(sym)

  end,

  tags = {
    builtin = nil,
    external = { '!gelatinbrain', '!hof', '!mc4d', museum = 6777, '!wca' },

    author = "Jason White",
    '!inventor',

    'type/puzzle',
    'shape/3d/platonic/cube',
    algebraic = {
      'doctrinaire', 'pseudo/doctrinaire',
      '!abelian', '!fused', '!orientations/non_abelian', '!trivial', '!weird_orbits',
    },
    axes = { '3d/elementary/cubic', '!hybrid', '!multicore' },
    colors = { '!multi_per_facet', '!multi_facet_per' },
    completeness = { 'super', '!real', '!laminated', 'complex' },
    cuts = { '!depth', '!stored', '!wedge' },
    turns_by = {'face', 'facet'},
    'experimental',
    '!canonical',
    '!family',
    '!variant',
    '!meme',
    '!shapeshifting',
  },
}

puzzles:add{
  id = 'complex_tetrahedron',
  name = "Complex Tetrahedron",
  aliases = {"Laminated Tetrahedron"},
  version = '1.0.2',
  ndim = 3,
  colors = 'tetrahedron',
  remove_internals = false,
  build = function(self)
    local sym = cd'a3'
    local shape = lib.symmetries.tetrahedral.tetrahedron()
    local d = 1/5 -- cut depth parameter, currently set so core-segments and anticore appear* identical in size

    self:carve(shape:iter_poles())

    -- Define axes and slices
    self.axes:add(shape:iter_poles(), {d, -(2+d)})
    self.axes:add(shape:iter_vertices(), {2+d, -d})

    -- Define twists
    for _, axis, twist_transform in sym.chiral:orbit(self.axes[sym.xoo.unit], sym:thru(3, 2)) do
      self.twists:add(axis, twist_transform, {gizmo_pole_distance = 1.4})
    end

    for _, axis, twist_transform in sym.chiral:orbit(self.axes[sym.oox.unit], sym:thru(2, 1)) do
      self.twists:add(axis, twist_transform, {gizmo_pole_distance = 1})
    end

    --Give axes labels for filters, twists, and to simplify piece filters
    utils.unpack_named(_ENV, self.axes)

    -- Add super-stickers on internal faces
    -- get vectors that point to faces, then slice along cutplanes
    local F_v = sym.oox.unit
    local U_v = sym:thru(3):transform(sym.oox.unit)
    local R_v = sym:thru(2, 3):transform(sym.oox.unit)
    local L_v = sym:thru(1, 2, 3):transform(sym.oox.unit)

    self:slice(plane(F_v, d), {stickers = self.colors.F})
    self:slice(plane(F_v, -2-d), {stickers = self.colors.F})
    self:slice(plane(U_v, d), {stickers = self.colors.U})
    self:slice(plane(U_v, -2-d), {stickers = self.colors.U})
    self:slice(plane(R_v, d), {stickers = self.colors.R})
    self:slice(plane(R_v, -2-d), {stickers = self.colors.R})
    self:slice(plane(L_v, d), {stickers = self.colors.L})
    self:slice(plane(L_v, -2-d), {stickers = self.colors.L})

    -- Mark one copy of each piece-type
    self:mark_piece(~R(1) & ~L(1) & ~U(1) & ~F(1), 'core', "Core")
    self:mark_piece(R(1) & ~L(1) & ~U(1) & ~F(1), 'center', "Center")
    self:mark_piece(R(1) & L(1) & ~U(1) & ~F(1), 'edge', "Edge")
    self:mark_piece(R(1) & L(1) & U(1) & ~F(1), 'corner', "Corner")
    self:mark_piece(R(1) & L(1) & U(1) & F(1), 'anticore', "Anticore")
    piecewarning()

    -- Pattern piece-types around the puzzle
    self:unify_piece_types(sym)

  end,

  tags = {
    builtin = nil,
    external = { '!gelatinbrain', '!hof', '!mc4d', museum = 6130, '!wca' },

    author = "Jason White",
    '!inventor',

    'type/puzzle',
    'shape/3d/platonic/tetrahedron',
    algebraic = {
      'doctrinaire', 'pseudo/doctrinaire',
      '!abelian', '!fused', '!orientations/non_abelian', '!trivial', '!weird_orbits',
    },
    axes = { '3d/elementary/tetrahedral', '!hybrid', '!multicore' },
    colors = { '!multi_per_facet', '!multi_facet_per' },
    completeness = { 'super', '!real', 'laminated', 'complex' },
    cuts = { '!depth', '!stored', '!wedge' },
    turns_by = {'face', 'facet', 'vertex'},
    'experimental',
    '!canonical',
    '!family',
    '!variant',
    '!meme',
    '!shapeshifting',
  },
}

puzzles:add{
  id = 'complex_triprism',
  name = "Complex Triangular Prism",
  version = '1.0.1',
  colors = 'prism:6',
  ndim = 3,
  remove_internals = false,
  build = function(self)
    local triangle = polygonal.ngon(3,1)
    local side_cut_depths = {1/4, -5/4}
    local height = triangle.edge_length/2
    local line = linear.line(height, 'z', 'U', 'D')
    local sym = cd{3, 2}

    local line_cut_depths = {height*3/5, -height/5}

    local base_colors, base_axes = utils.cut_ft_shape(self, line, line_cut_depths, 'U', 'D')
    local side_colors, side_axes = utils.cut_ft_shape(self, triangle, side_cut_depths, 'F')
    self.axes:reorder(prisms.facet_order)

    local U = base_axes[1]
    local F1 = side_axes[1]

    local function add_twist_set(axis, twist_transform, twist_data)
        for t in sym:orbit(axis) do
            self.twists:add(t:transform(axis), t:transform_oriented(twist_transform), twist_data)
        end
    end

    add_twist_set(U, sym:thru(2, 1), {gizmo_pole_distance = height})
    add_twist_set(F1, sym:thru(3, 1), {gizmo_pole_distance = 1})

    utils.unpack_named(_ENV, self.axes)
    
    -- internal stickers; let the record show that snek tried to compress the loops
    for j=1,2,1 do
        self:slice(plane(FA.vector, side_cut_depths[j]), {stickers = self.colors.FA}) -- doesn't seem to be another way to access the color (probably skill issue)
        self:slice(plane(FB.vector, side_cut_depths[j]), {stickers = self.colors.FB})
        self:slice(plane(FC.vector, side_cut_depths[j]), {stickers = self.colors.FC})
        self:slice(plane(-FA.vector, -side_cut_depths[j]), {stickers = self.colors.FD})
        self:slice(plane(-FB.vector, -side_cut_depths[j]), {stickers = self.colors.FE})
        self:slice(plane(-FC.vector, -side_cut_depths[j]), {stickers = self.colors.FF})
    end
    for k=3,-3,-2 do
        self:slice(plane(U.vector, height*k/5), {stickers = self.colors.U})
        self:slice(plane(D.vector, height*k/5), {stickers = self.colors.D})
    end

    -- Mark piece-types
    self:mark_piece(~U(1) & ~FA(1) & ~FB(1) & ~FC(1) & ~D(1), 'core', "Core")
    self:add_piece_type('center', "Center")
    self:mark_piece(U(1) & ~FA(1) & ~FB(1) & ~FC(1) & ~D(1), 'center/top_center', "Top Center")
    self:mark_piece(~U(1) & FA(1) & ~FB(1) & ~FC(1) & ~D(1), 'center/side_center', "Side Center")
    self:add_piece_type('edge', "Edge")
    self:mark_piece(U(1) & FA(1) & ~FB(1) & ~FC(1) & ~D(1), 'edge/top_edge', "Top Edge")
    self:mark_piece(~U(1) & FA(1) & FB(1) & ~FC(1) & ~D(1), 'edge/side_edge', "Side Edge")
    self:mark_piece(U(1) & ~FA(1) & ~FB(1) & ~FC(1) & D(1), 'axle', "Axle")
    self:mark_piece(U(1) & FA(1) & FB(1) & ~FC(1) & ~D(1), 'corner', "Corner")
    self:mark_piece(U(1) & FA(1) & ~FB(1) & ~FC(1) & D(1), 'triwall', "Triwall")
    self:mark_piece(~U(1) & FA(1) & FB(1) & FC(1) & ~D(1), 'antiaxle', "Anti-Axle")
    self:add_piece_type('anticenter', "Anti-Center")
    self:mark_piece(~U(1) & FA(1) & FB(1) & FC(1) & D(1), 'anticenter/top_anticenter', "Top Anti-Center")
    self:mark_piece(U(1) & ~FA(1) & FB(1) & FC(1) & D(1), 'anticenter/side_anticenter', "Side Anti-Center")
    self:mark_piece(U(1) & FA(1) & FB(1) & FC(1) & D(1), 'anticore', "Anti-Core")
    piecewarning()

    self:unify_piece_types(sym)
  end,

  tags = {
    builtin = nil,
    external = { '!gelatinbrain', '!hof', '!mc4d', '!museum', '!wca' },

    author = "Jason White",
    inventor = "Luna Harran",

    'type/puzzle',
    'shape/3d/prism',
    algebraic = {
      'doctrinaire', 'pseudo/doctrinaire',
      '!abelian', '!fused', '!orientations/non_abelian', '!trivial', '!weird_orbits',
    },
    axes = { '3d/prismatic', '!hybrid', '!multicore' },
    colors = { '!multi_per_facet', '!multi_facet_per' },
    completeness = { 'super', '!real', '!laminated', 'complex' },
    cuts = { '!depth', '!stored', '!wedge' },
    turns_by = {'face', 'facet'},
    'experimental',
    '!canonical',
    '!family',
    '!variant',
    '!meme',
    '!shapeshifting',
  },
}
