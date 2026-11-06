// The window frame: rounded at the top, square at the bottom, as on
// Windows 7.
//
// A ShojiWM style has one `borderRadius` for all four corners
// (SSDStyle.borderRadius, packages/shoji_wm/src/types.ts). A paint shader
// decides the colour of every pixel itself, so it can use its own radius per
// corner. See docs/configuration/paint.md for the contract.

// The radius of the two top corners, in logical pixels.
uniform float top_radius;

vec4 paint_main(PaintContext ctx) {
    float r = top_radius * ctx.scale;
    vec4 radius = vec4(r, r, 0.0, 0.0);

    vec4 outer_rect = vec4(0.0, 0.0, ctx.size_phy_px.x, ctx.size_phy_px.y);
    float outer = shoji_rrect_coverage(ctx.frag_phy_px, outer_rect, radius);

    // border_phy_px is top, right, bottom, left.
    vec4 b = ctx.border_phy_px;
    vec4 inner_rect = vec4(
        b.w,
        b.x,
        max(ctx.size_phy_px.x - b.w - b.y, 0.0),
        max(ctx.size_phy_px.y - b.x - b.z, 0.0)
    );
    vec4 inner_radius = max(
        radius - vec4(max(b.x, b.w), max(b.x, b.y), max(b.z, b.y), max(b.z, b.w)),
        vec4(0.0)
    );
    float inner = shoji_rrect_coverage(ctx.frag_phy_px, inner_rect, inner_radius);

    vec4 border = ctx.border_color * (outer - inner);
    vec4 fill = ctx.background * inner;
    return shoji_over(border, fill);
}
