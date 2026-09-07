shader_type canvas_item;
render_mode unshaded;
void fragment() {
    vec4 sample_color = texture(TEXTURE, UV);
    // Shared Cattle Trail binary magenta criterion, evaluated at display time.
    bool background = sample_color.r - sample_color.g > 25.0/255.0
        && sample_color.b - sample_color.g > 25.0/255.0;
    COLOR = background ? vec4(0.0) : sample_color;
}
