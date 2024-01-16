

struct Body {
    mass: f32,
    pos: vec2<f32>,
    vel: vec2<f32>,
    acc: vec2<f32>
}


@group(0) @binding(0)
var<uniform> g: f32;

@group(0) @binding(1)
var<uniform> delta_time: f32;

@group(0) @binding(2)
var<storage> bodies_src: array<Body>;

@group(0) @binding(3)
var<storage, read_write> bodies_dst: array<Body>;


@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
  
  let total_bodies = arrayLength(&bodies_src);
  let index = invocation_id.x;

  if (index >= total_bodies) {
      return;
  }

  let current_pos = bodies_src[index].pos;
  let current_mass = bodies_src[index].mass;

  var i: u32 = 0u;

  loop {

    if (i >= total_bodies) {
      break;
    }
    
    if (i == index) {
      continue;
    }

    let rhs_pos = bodies_src[i].pos;
    let rhs_mass = bodies_src[i].mass;
  
    let delta = rhs_pos - current_pos;
    let distance_sq = pow(delta.x, 2.) + pow(delta.y, 2.);
    
    let f = g / distance_sq;
    let force_unit_mass = delta * f;
    
    bodies_dst[index].acc += force_unit_mass * rhs_mass;
    bodies_dst[i].acc -= force_unit_mass * current_mass;

  }
}
