struct Globals {
    model: mat4x4<f32>, view_projection: mat4x4<f32>, shadow: mat4x4<f32>,
    camera: vec4<f32>, viewport: vec4<f32>, light: vec4<f32>, post: vec4<f32>,
}
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var glow_map: texture_2d<f32>;
@group(0) @binding(3) var depth_map: texture_depth_multisampled_2d;
@group(0) @binding(4) var normal_map: texture_2d<f32>;
@group(0) @binding(5) var<uniform> g: Globals;
struct Fragment { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vertex(@builtin(vertex_index) index: u32) -> Fragment {
    let xy = array<vec2<f32>,3>(vec2<f32>(-1.0,-1.0),vec2<f32>(3.0,-1.0),vec2<f32>(-1.0,3.0));
    var out: Fragment; out.position=vec4<f32>(xy[index],0.0,1.0); out.uv=xy[index]*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5); return out;
}
fn blur(uv: vec2<f32>, direction: vec2<f32>) -> vec4<f32> {
    let texel=1.0/vec2<f32>(textureDimensions(source));
    let at=uv*g.viewport.w;
    return textureSample(source,linear_sampler,at)*0.227027
        +(textureSample(source,linear_sampler,at+direction*texel*1.384615)+textureSample(source,linear_sampler,at-direction*texel*1.384615))*0.316216
        +(textureSample(source,linear_sampler,at+direction*texel*3.230769)+textureSample(source,linear_sampler,at-direction*texel*3.230769))*0.070270;
}
@fragment fn horizontal(v: Fragment) -> @location(0) vec4<f32> { return blur(v.uv,vec2<f32>(1.0,0.0)); }
@fragment fn vertical(v: Fragment) -> @location(0) vec4<f32> { return blur(v.uv,vec2<f32>(0.0,1.0)); }
fn linear_depth(depth: f32) -> f32 { return g.post.x*g.post.y/(g.post.y-depth*(g.post.y-g.post.x)); }
fn aces(v: vec3<f32>) -> vec3<f32> { return clamp((v*(2.51*v+vec3<f32>(0.03)))/(v*(2.43*v+vec3<f32>(0.59))+vec3<f32>(0.14)),vec3<f32>(0.0),vec3<f32>(1.0)); }
fn gamma(v: vec3<f32>) -> vec3<f32> { return select(12.92*v,1.055*pow(v,vec3<f32>(1.0/2.4))-vec3<f32>(0.055),v>vec3<f32>(0.0031308)); }
@fragment fn composite(v: Fragment) -> @location(0) vec4<f32> {
    let at=v.uv*g.viewport.w;
    let hdr=textureSample(source,linear_sampler,at);
    let glow=textureSample(glow_map,linear_sampler,at).rgb;
    let normal_sample=textureSample(normal_map,linear_sampler,at);
    let size=vec2<i32>(textureDimensions(depth_map));
    let pixel=clamp(vec2<i32>(v.position.xy),vec2<i32>(0),size-vec2<i32>(1));
    let depth=linear_depth(textureLoad(depth_map,pixel,0));
    var occlusion=0.0;
    let offsets=array<vec2<i32>,8>(vec2<i32>(-7,0),vec2<i32>(7,0),vec2<i32>(0,-7),vec2<i32>(0,7),vec2<i32>(-5,-5),vec2<i32>(5,5),vec2<i32>(-5,5),vec2<i32>(5,-5));
    for(var i=0;i<8;i++) {
        let q=clamp(pixel+offsets[i],vec2<i32>(0),size-vec2<i32>(1));
        let neighbor=linear_depth(textureLoad(depth_map,q,0));
        let delta=depth-neighbor;
        if delta>0.018 && delta<0.25 { occlusion+=1.0; }
    }
    let ao=1.0-occlusion*0.045*select(0.0,1.0,normal_sample.a>0.5);
    let halo=min(max(max(glow.r,glow.g),glow.b)*0.035,0.18);
    let alpha=hdr.a+halo*(1.0-hdr.a);
    if alpha<0.0001 { return vec4<f32>(0.0); }
    let radiance=(hdr.rgb*ao+glow*0.065)/alpha;
    return vec4<f32>(gamma(aces(radiance*g.post.z))*alpha,alpha);
}
