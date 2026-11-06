// Clip the blur behind a layer surface to the shape the surface draws.
//
// A backdrop effect covers the whole surface, so a panel with rounded
// corners gets a blurred square behind them. Multiplying the blur by the
// surface's own alpha removes that. The default ShojiWM config does the same
// through `layerSource()`; see docs/configuration/effects.md.

// The surface's own content, bound as a texture by the config.
uniform sampler2D layer_mask;
// Alpha at which the surface counts as drawn, and the width of the ramp.
uniform float opacity_threshold;
uniform float mask_feather;

vec4 shader_main(EffectContext effect) {
    vec4 blurred = texture2D(tex, effect.texture_uv);
    float alpha = texture2D(layer_mask, effect.texture_uv).a;
    float keep = smoothstep(
        opacity_threshold - mask_feather,
        opacity_threshold + mask_feather,
        alpha
    );
    // The result is premultiplied, so one multiplication covers both the
    // colour and the alpha.
    return blurred * keep;
}
