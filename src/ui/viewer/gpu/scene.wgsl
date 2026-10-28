struct Globals {
    model: mat4x4<f32>, view_projection: mat4x4<f32>, shadow: mat4x4<f32>,
    camera: vec4<f32>, viewport: vec4<f32>, light: vec4<f32>, post: vec4<f32>,
}
@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var<storage, read> effects: array<vec4<f32>>;
@group(0) @binding(2) var shadow_map: texture_depth_2d;
@group(0) @binding(3) var shadow_sampler: sampler_comparison;
struct Material { base: vec4<f32>, properties: vec4<f32> }
@group(1) @binding(0) var<uniform> material: Material;
@group(1) @binding(1) var albedo_map: texture_2d<f32>;
@group(1) @binding(2) var legend_map: texture_2d<f32>;
@group(1) @binding(3) var surface_sampler: sampler;

struct Vertex {
    @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>,
    @location(3) part: u32, @location(4) legend: f32, @location(5) sources: vec4<u32>,
    @location(6) weights: vec4<f32>, @location(7) flags: vec2<f32>,
}
struct Fragment {
    @builtin(position) position: vec4<f32>, @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>, @location(3) emission: vec3<f32>,
    @location(4) spill: vec3<f32>, @location(5) metadata: vec3<f32>, @location(6) light_position: vec4<f32>,
}
fn world_position(v: Vertex) -> vec4<f32> {
    let pressed = effects[v.part].w;
    return g.model * vec4<f32>(v.position - vec3<f32>(0.0, pressed * 0.008, 0.0), 1.0);
}
@vertex fn shadow_vertex(v: Vertex) -> @builtin(position) vec4<f32> { return g.shadow * world_position(v); }
@vertex fn vertex(v: Vertex) -> Fragment {
    var out: Fragment;
    let world = world_position(v);
    out.position = g.view_projection * world;
    out.world = world.xyz;
    out.normal = normalize((g.model * vec4<f32>(v.normal, 0.0)).xyz);
    out.uv = v.uv;
    out.emission = effects[v.part].rgb;
    out.spill = effects[v.sources.x].rgb * v.weights.x + effects[v.sources.y].rgb * v.weights.y
        + effects[v.sources.z].rgb * v.weights.z + effects[v.sources.w].rgb * v.weights.w;
    out.metadata = vec3<f32>(v.legend, v.flags);
    out.light_position = g.shadow * world;
    return out;
}
fn visibility(position: vec4<f32>, normal: vec3<f32>) -> f32 {
    let light = position.xyz / position.w;
    let uv = light.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) { return 1.0; }
    let bias = 0.00035 + 0.001 * (1.0 - max(dot(normal, g.light.xyz), 0.0));
    var sum = 0.0;
    for (var y = -2; y <= 2; y++) { for (var x = -2; x <= 2; x++) {
        sum += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2<f32>(f32(x), f32(y)) * g.viewport.z * 1.35, light.z - bias);
    } }
    return sum / 25.0;
}
fn fresnel(cosine: f32, f0: vec3<f32>) -> vec3<f32> { return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cosine, 5.0); }
fn brdf(base: vec3<f32>, metallic: f32, roughness: f32, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>) -> vec3<f32> {
    let h = normalize(v + l);
    let nl = max(dot(n,l),0.0); let nv = max(dot(n,v),0.001); let nh = max(dot(n,h),0.0);
    let a2 = pow(max(roughness,0.12),4.0);
    let d = a2 / max(3.14159265 * pow(nh*nh*(a2-1.0)+1.0,2.0),0.00001);
    let k = pow(roughness+1.0,2.0)/8.0;
    let geometry = (nv/(nv*(1.0-k)+k)) * (nl/(nl*(1.0-k)+k));
    let f = fresnel(max(dot(v,h),0.0),mix(vec3<f32>(0.04),base,metallic));
    let specular = f*d*geometry/max(4.0*nv*nl,0.001);
    return ((vec3<f32>(1.0)-f)*(1.0-metallic)*base/3.14159265 + specular)*nl;
}
struct Output { @location(0) color: vec4<f32>, @location(1) emission: vec4<f32>, @location(2) normal: vec4<f32> }
@fragment fn fragment(v: Fragment) -> Output {
    let texel = textureSample(albedo_map, surface_sampler, v.uv);
    let alpha = texel.a * material.base.a;
    if alpha < 0.01 { discard; }
    let n = normalize(v.normal);
    let eye = normalize(g.camera.xyz - v.world);
    let mask = textureSample(legend_map, surface_sampler, v.uv).r * v.metadata.x;
    let illuminated = select(0.0,1.0,max(max(v.emission.r,v.emission.g),v.emission.b)>0.01);
    let base = texel.rgb * material.base.rgb * (1.0 - mask * illuminated * 0.9);
    let roughness = max(material.properties.y,0.14);
    let metallic = material.properties.x;
    let shadow = visibility(v.light_position,n);
    let key = brdf(base,metallic,roughness,n,eye,normalize(g.light.xyz))*vec3<f32>(2.9,2.8,2.65)*shadow;
    let fill = brdf(base,metallic,roughness,n,eye,normalize(vec3<f32>(0.7,0.35,0.55)))*vec3<f32>(0.25,0.34,0.5);
    let hemisphere = mix(vec3<f32>(0.025,0.022,0.018),vec3<f32>(0.19,0.22,0.28),clamp(n.y*0.5+0.5,0.0,1.0));
    let reflection = reflect(-eye,n);
    let softbox = pow(max(dot(reflection,normalize(vec3<f32>(-0.35,0.7,0.8))),0.0),20.0);
    let environment = fresnel(max(dot(n,eye),0.0),mix(vec3<f32>(0.04),base,metallic))*(vec3<f32>(0.12)+softbox*2.0)*(1.0-roughness*0.55);
    let source = v.emission * (mask*0.6 + v.metadata.z*0.48);
    let spill = base*v.spill*0.42;
    let color = key + fill + base*hemisphere + environment + spill + source;
    var out: Output;
    out.color = vec4<f32>(color*alpha,alpha);
    out.emission = vec4<f32>(source*alpha,alpha);
    out.normal = vec4<f32>((n*0.5+0.5)*alpha,alpha);
    return out;
}
