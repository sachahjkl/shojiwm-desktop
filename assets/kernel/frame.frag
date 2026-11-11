// The window frame of the Kernel theme.
//
// Outside in: a dark line round the whole window, then on the left, right and
// bottom a line of the caption's tint, then the client. The caption is a plate:
// a gradient from left to right, a little lighter at its top, a fine line of
// light along its top edge, its edges darkened inward by a few pixels (the
// plate reads framed, not glossy), and a dark line under it, against the client.
//
// See docs/configuration/paint.md in ShojiWM for the contract.

uniform float radius;     // logical px, the two top corners
uniform float caption_h;  // logical px, window top to client top
uniform float border_w;   // logical px, the other three sides
uniform float outer;      // 1: the outer line is drawn; 0: maximised, it is off the screen
uniform vec4 cap_left;    // the caption at its left, straight alpha
uniform vec4 cap_right;   // ... at its right
uniform vec4 line_outer;
uniform vec4 line_tint;
uniform float light;      // the line of light along the caption's top: its alpha

vec4 paint_main(PaintContext ctx) {
    float s = ctx.scale;
    vec2 p = ctx.frag_phy_px;
    vec2 size = ctx.size_phy_px;
    float line = max(1.0, floor(s));
    float r = radius * s;
    float cap = floor(caption_h * s);
    float bw = floor(border_w * s);

    vec4 whole = vec4(0.0, 0.0, size);
    float cover = shoji_rrect_coverage(p, whole, vec4(r, r, 0.0, 0.0));
    if (cover <= 0.0) { return vec4(0.0); }

    // the body: the tint (the client covers its middle)
    vec4 c = line_tint;

    // the caption plate
    float o = outer > 0.5 ? line : 0.0;
    if (p.y < cap) {
        float t = clamp((p.x - o) / max(size.x - 2.0 * o, 1.0), 0.0, 1.0);
        vec4 g = mix(cap_left, cap_right, smoothstep(0.0, 0.9, t));
        // a little lighter at the top
        float top = 1.0 - smoothstep(0.0, cap * 0.9, p.y);
        g.rgb = mix(g.rgb, vec3(1.0), 0.06 * top);
        // its edges darkened inward: a frame inside the frame
        float dx = min(p.x - o, size.x - o - p.x);
        float dy = p.y - o;
        float edge = min(dx, dy);
        float shade = 1.0 - 0.18 * (1.0 - smoothstep(0.0, 4.0 * s, edge));
        g.rgb *= shade;
        // the line of light along its top
        if (p.y >= o && p.y < o + line) { g.rgb = mix(g.rgb, vec3(1.0), light); }
        // the dark line under it, against the client
        if (p.y >= cap - line) { g = line_outer; }
        c = g;
    }

    // the outer line, round the whole window
    if (outer > 0.5) {
        float inside = shoji_rrect_coverage(p, vec4(line, line, size.x - 2.0 * line, size.y - 2.0 * line), vec4(max(r - line, 0.0), max(r - line, 0.0), 0.0, 0.0));
        c = mix(line_outer, c, inside);
    }
    return shoji_premultiply(c) * cover;
}
