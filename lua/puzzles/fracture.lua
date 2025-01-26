puzzles:add{
	id = 'sunrise_sunset',
	version = '0.1.0',
	name = "Sunrise Sunset",
	ndim = 3,
	build = function(self)
		local sym = cd{3,2}
		local side = sym.xoo.unit
		local side_depth = sqrt(1/7)
		local top = sym.oox.unit
		local top_depth = sqrt(3/7)

		-- Helper functions for twists, stored cuts
		local function cw90(fix)
			return rot{fix=fix, angle=-pi/2}
		end
		local function top_cut(fix, a)
			return cw90(fix):transform(plane(a.vector.unit*top_depth))
		end
		local function side_cut(fix, a)
			return cw90(fix):transform(plane(a.vector.unit*side_depth))
		end

		self:carve(sym:orbit(sym.oxx.unit))

		-- Add and name axis system
		self.axes:add(sym:orbit(side), {INF, side_depth})
		self.axes:add(sym:orbit(top), {INF, top_depth})
		self.axes:rename({'R', 'L', 'F', 'U', 'D'})
		lib.utils.unpack_named(_ENV, self.axes)

		-- Add stored cuts
		for t, diag_cut, region_to_cut in sym:orbit(side_cut(R, L), R(1)) do
			self:slice(diag_cut, { region = region_to_cut })
		end
		for t, diag_cut, region_to_cut in sym:orbit(top_cut(R, U), R(1)) do
			self:slice(diag_cut, { region = region_to_cut })
		end

		-- Add twists
		for _, axis, twist_transform in sym.chiral:orbit(R, cw90(R)) do
			self.twists:add(axis, twist_transform, {gizmo_pole_distance = 3/4})
		end
		for _, axis, twist_transform in sym.chiral:orbit(U, sym:thru(2,1)) do
			self.twists:add(axis, twist_transform, {gizmo_pole_distance = 3/4})
		end

		-- Set color-scheme, matching mass-produced precedent (mf8 More Madness Pyraminx)
		self.colors:set_defaults({"Orange", "Green", "Yellow", "White", "Red", "Blue"})
		self.colors:rename({"Orange", "Green", "Yellow", "White", "Red", "Blue"})

		self:mark_piece(F(1) & R(1), 'edge', "Edge")
		self:mark_piece(F(1) & ~side_cut(F, R).region, 'edge')
		self:mark_piece(~R(1) & ~top_cut(F,U).region, 'wing', "Wing")
		self:mark_piece(D(1) & side_cut(F,R).region, 'wing')
		self:mark_piece(U(1) & ~F(1) & ~R(1) & ~L(1), 'top_corner', "Top Corner")
		self:mark_piece(~U(1) & ~D(1) & ~top_cut(R, U).region & ~top_cut(F, D).region, 'side_corner', "Side Corner")
		
		self:unify_piece_types(sym)

	end,
	tags = {
    builtin = nil,
    external = { '!gelatinbrain', '!hof', '!mc4d', museum = 3268, '!wca' },

    author = "Jason White",
    inventor = "David Pitcher",

    'type/puzzle',
    'shape/3d/bipyramid',
    algebraic = {
      'doctrinaire', 'pseudo/doctrinaire',
      '!abelian', '!fused', '!orientations/non_abelian', '!trivial', '!weird_orbits',
    },
    axes = { '3d/prismatic', '!hybrid', '!multicore' },
    colors = { '!multi_per_facet', '!multi_facet_per' },
    completeness = { 'super', '!real', '!laminated', '!complex' },
    cuts = { '!depth', 'stored', '!wedge' },
    'turns_by/vertex',
    'experimental',
    '!canonical',
    '!family',
    '!variant',
    '!meme',
    '!shapeshifting',
	}
}

puzzles:add{
	id = 'solar_eclipse',
	version = '0.1.0',
	name = "Solar Eclipse",
	ndim = 3,
	build = function(self)
		local sym = cd{3,2}
		local side = sym.xoo.unit
		local side_depth = sqrt(1/7)
		local top = sym.oox.unit
		local top_depth = 1/3

		-- Helper functions for twists, stored cuts
		local function cw90(fix)
			return rot{fix=fix, angle=-pi/2}
		end
		local function top_cut(fix, a)
			return cw90(fix):transform(plane(a.vector.unit*top_depth))
		end
		local function side_cut(fix, a)
			return cw90(fix):transform(plane(a.vector.unit*side_depth))
		end

		self:carve(sym:orbit(sym.oxx.unit))

		-- Add and name axis system
		self.axes:add(sym:orbit(side), {INF, side_depth})
		self.axes:add(sym:orbit(top), {INF, top_depth})
		self.axes:rename({'R', 'L', 'F', 'U', 'D'})
		lib.utils.unpack_named(_ENV, self.axes)

		-- Add stored cuts
		for t, diag_cut, region_to_cut in sym:orbit(side_cut(R, L), R(1)) do
			self:slice(diag_cut, { region = region_to_cut })
		end
		for t, diag_cut, region_to_cut in sym:orbit(top_cut(R, U), R(1)) do
			self:slice(diag_cut, { region = region_to_cut })
		end

		-- Add twists
		for _, axis, twist_transform in sym.chiral:orbit(R, cw90(R)) do
			self.twists:add(axis, twist_transform, {gizmo_pole_distance = 3/4})
		end
		for _, axis, twist_transform in sym.chiral:orbit(U, sym:thru(2,1)) do
			self.twists:add(axis, twist_transform, {gizmo_pole_distance = 3/4})
		end

		-- Set color-scheme, matching mass-produced precedent (mf8 More Madness Pyraminx)
		self.colors:set_defaults({"Orange", "Green", "Yellow", "White", "Red", "Blue"})
		self.colors:rename({"Orange", "Green", "Yellow", "White", "Red", "Blue"})

		self:mark_piece(F(1) & R(1) & U(1), 'triangle', "Triangle")
		self:mark_piece(F(1) & ~side_cut(F, R).region & ~top_cut(F, U).region & top_cut(F, D).region, 'triangle')
		self:mark_piece(U(1) & ~L(1) & ~top_cut(F,D).region & side_cut(F,L).region, 'kite', "Kite")
		self:mark_piece(F(1) & R(1) & ~U(1) & ~D(1), 'edge', "Edge")
		self:mark_piece(F(1) & ~side_cut(F, R).region & top_cut(F, U).region & top_cut(F, D).region, 'edge')
		self:mark_piece(~R(1) & ~U(1) & ~D(1) & ~top_cut(F,U).region, 'wing', "Wing")
		self:mark_piece(L(1) & U(1) & side_cut(L,R).region & top_cut(L, U).region & top_cut(L, D).region, 'wing')
		self:mark_piece(U(1) & ~F(1) & ~R(1) & ~L(1), 'top_corner', "Top Corner")
		self:mark_piece(L(1) & ~U(1) & ~D(1) & top_cut(L, U).region & top_cut(L, D).region, 'side_corner', "Side Corner")
		
		self:unify_piece_types(sym)

	end,
	tags = {
    builtin = nil,
    external = { '!gelatinbrain', '!hof', '!mc4d', museum = 12070, '!wca' },

    author = "Jason White",
    inventor = "Sukjae Lee",

    'type/puzzle',
    'shape/3d/bipyramid',
    algebraic = {
      'doctrinaire', 'pseudo/doctrinaire',
      '!abelian', '!fused', '!orientations/non_abelian', '!trivial', '!weird_orbits',
    },
    axes = { '3d/prismatic', '!hybrid', '!multicore' },
    colors = { '!multi_per_facet', '!multi_facet_per' },
    completeness = { '!super', '!real', '!laminated', '!complex' },
    cuts = { '!depth', 'stored', '!wedge' },
    'turns_by/vertex',
    'experimental',
    '!canonical',
    '!family',
    '!variant',
    '!meme',
    '!shapeshifting',
	}
}