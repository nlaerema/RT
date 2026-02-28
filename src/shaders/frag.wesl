
struct Immediates {
    window_size: vec2u,
	aspect_ratio: vec2f,
}

var<immediate> imm: Immediates;

fn color_from_position(pos: vec2f) -> vec4f {
	let ndc = pos * 2.0 / vec2f(imm.window_size) - 1.0;
	let aspectNDC = ndc * imm.aspect_ratio;
	let dist = distance(aspectNDC, vec2f(0, 0));
	return select(
		select(
			vec4f(0.0, 0.0, 1.0, 1.0),
			vec4f(0.0, 1.0, 0.0, 1.0),
			dist < 1.0
		),
		vec4f(1.0, 0.0, 0.0, 1.0),
		dist < 0.5
	);
}

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
	return (color_from_position(pos.xy)); 
}