// The ghost tab of a window group (src/themes/tabs.tsx): where a dragged window
// would land. A plate with a dashed outline, square corners.
//
// See docs/configuration/paint.md in ShojiWM for the contract.

uniform vec4 fill;   // straight alpha
uniform vec4 line;   // straight alpha
uniform float dash;  // logical px: a dash, and the gap after it

vec4 paint_main(PaintContext ctx) {
    float s = ctx.scale;
    vec2 p = ctx.frag_phy_px;
    vec2 size = ctx.size_phy_px;
    float w = max(1.0, floor(s));
    float d = max(1.0, floor(dash * s));
    vec4 c = shoji_premultiply(fill);
    bool horizontal = p.y < w || p.y >= size.y - w;
    bool vertical = p.x < w || p.x >= size.x - w;
    // along the top and bottom the dashes follow x, along the sides y
    float along = horizontal ? p.x : p.y;
    if ((horizontal || vertical) && mod(floor(along), 2.0 * d) < d) {
        c = shoji_over(shoji_premultiply(line), c);
    }
    return c;
}
