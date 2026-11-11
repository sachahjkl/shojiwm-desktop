// A caption button's plate in the Kernel theme: a gradient from top to bottom, a
// fine line of light along its top, and a dark seam on its left that parts it
// from what is before it.

uniform vec4 top;     // straight alpha
uniform vec4 bottom;
uniform float light;  // the line along the top: its alpha
uniform float seam;   // the line on the left: its alpha

vec4 paint_main(PaintContext ctx) {
    float s = ctx.scale;
    vec2 p = ctx.frag_phy_px;
    vec2 size = ctx.size_phy_px;
    float line = max(1.0, floor(s));
    vec4 c = mix(top, bottom, clamp(p.y / max(size.y, 1.0), 0.0, 1.0));
    if (p.y < line) { c.rgb = mix(c.rgb, vec3(1.0), light); c.a = max(c.a, light); }
    vec4 pm = shoji_premultiply(c);
    if (p.x < line && seam > 0.0) { pm = shoji_over(vec4(0.0, 0.0, 0.0, seam), pm); }
    return pm;
}
