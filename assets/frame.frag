// The window frame of Windows 7 Aero, drawn from measured pixels.
//
// Outside in, on every side (docs/aero-measurements.md):
//
//   1 px dark line, 1 px light line, the glass, 1 px light line, 1 px dark
//   line, then the client.
//
// The glass is 4 px on the left, right and bottom and 26 px at the top. The
// top two corners are rounded by 6 px and the bottom two by 5 px, which a
// style cannot express: `SSDStyle.borderRadius` is one number for all four
// corners. A paint shader decides every pixel itself, so it can.
//
// See docs/configuration/paint.md in ShojiWM for the contract.

uniform float radius_top;     // logical px
uniform float radius_bottom;  // logical px
uniform float caption_h;      // logical px, window top to client top
uniform float border_w;       // logical px, the other three sides
uniform vec4 tint;            // the glass, straight alpha
uniform vec4 line_outer;
uniform vec4 line_inner;
uniform vec4 edge_light;      // the light line around the client
uniform vec4 edge_dark;       // the dark line around the client
// 1 while the window has its outer two lines, 0 when it is maximised and
// they sit off the screen.
uniform float outer_lines;

float sharp_rect(vec2 p, vec4 r) {
    return shoji_rrect_coverage(p, r, vec4(0.0));
}

vec4 grow(vec4 r, float d) {
    return vec4(r.x - d, r.y - d, r.z + 2.0 * d, r.w + 2.0 * d);
}

vec4 paint_main(PaintContext ctx) {
    float s = ctx.scale;
    vec2 p = ctx.frag_phy_px;
    vec2 size = ctx.size_phy_px;

    // One hairline stays one physical pixel at every scale, so the frame
    // keeps crisp edges at 125% instead of blurring over 1.25 px.
    float line = max(1.0, floor(s));

    vec4 radius = vec4(radius_top, radius_top, radius_bottom, radius_bottom) * s;
    float cap = caption_h * s;
    float bw = border_w * s;

    float first = outer_lines * line;
    float second = outer_lines * 2.0 * line;
    float outer = shoji_rrect_coverage(p, vec4(0.0, 0.0, size), radius);
    float after_dark = shoji_rrect_coverage(
        p,
        vec4(first, first, size - 2.0 * first),
        max(radius - first, vec4(0.0))
    );
    float after_light = shoji_rrect_coverage(
        p,
        vec4(second, second, size - 2.0 * second),
        max(radius - second, vec4(0.0))
    );

    // The client area is a plain rectangle; only the window's outline is
    // rounded.
    vec4 client = vec4(
        bw,
        cap,
        max(size.x - 2.0 * bw, 0.0),
        max(size.y - cap - bw, 0.0)
    );
    float client_cov = sharp_rect(p, client);
    float dark_cov = sharp_rect(p, grow(client, line));
    float light_cov = sharp_rect(p, grow(client, 2.0 * line));

    vec4 colour = vec4(0.0);
    colour += shoji_premultiply(line_outer) * (outer - after_dark) * outer_lines;
    colour += shoji_premultiply(line_inner) * (after_dark - after_light) * outer_lines;
    colour += shoji_premultiply(tint) * max(after_light - light_cov, 0.0);
    colour += shoji_premultiply(edge_light) * max(light_cov - dark_cov, 0.0);
    colour += shoji_premultiply(edge_dark) * max(dark_cov - client_cov, 0.0);
    return colour;
}
