// Ref : https://compute.toys/view/407

import super::core::FullscreenVertexOutput
import super::core::{view, lights, clusterable_objects}
import super::core::lighting::getDistanceAttenuation
import super::core::mesh_view_types::ClusterableObject

import super::selectors::{select_shape, select_op};
import super::types::{SdMaterial, SdMaterialPacked, SdShapeInstancePacked, pack_distance_info, unpack_sd_shape_instance, unpack_sd_material, unpack_distance_info, SdShapeInstance, SdOpInstance, SdOpInstancePacked, unpack_sd_op_instance, SdOp, SdOpPacked, unpack_sd_op, MarchOutput, DistanceInfo, DistanceInfoPacked}

@group(1) @binding(0) var depth_texture: texture_depth_2d;
struct RayMarchCamera {
    depth_scale: f32,
    eps: f32,
    max_distance: f32,
    max_steps: u32,
    shadow_eps: f32,
    shadow_max_steps: u32,
    shadow_max_distance: f32,
    normal_eps: f32
}
@group(1) @binding(1) var<uniform> settings: RayMarchCamera;

// TODO: seperate the sd_shapes buffer into multiple buffers for more performance
@group(2) @binding(0) var<storage, read> sd_shapes: array<SdShapeInstancePacked>;
@group(2) @binding(1) var<storage, read> sd_ops: array<SdOpInstancePacked>;

@group(3) @binding(0) var depth_prepass: texture_storage_2d<r32float, write>;
@group(3) @binding(1) var normal_prepass: texture_storage_2d<rgba32float, write>;
@group(3) @binding(2) var material_prepass: texture_storage_2d<rgba32float, write>;
@group(3) @binding(3) var mask_prepass: texture_storage_2d<r32float, write>;
@group(3) @binding(4) var scaled_depth_prepass: texture_storage_2d<r32float, read_write>;
@group(3) @binding(5) var scaled_normal_prepass: texture_storage_2d<rgba32float, read_write>;
@group(3) @binding(6) var scaled_material_prepass: texture_storage_2d<rgba32float, read_write>;
@group(3) @binding(7) var scaled_mask_prepass: texture_storage_2d<r32float, read_write>;

// TODO: Make op_resut recyce te sapce in te array to get a significant perforamce boost when dealing with large amounts of OPS
const MAX_OPS: u32 = 8;
var<private> op_results: array<DistanceInfoPacked, MAX_OPS>;

fn shape_to_dist(instance: SdShapeInstance, p: vec3f) -> DistanceInfo {
    let dist = select_shape(p, instance.shape, instance.transform, instance.modifier);
    return DistanceInfo(dist, instance.material);
}

fn blend_material(a: SdMaterial, b: SdMaterial, m: f32) -> SdMaterial {
    return SdMaterial(mix(b.color, a.color, m),mix(b.roughness, a.roughness, m), mix(b.fresnel, a.fresnel, m), mix(b.metallic, a.metallic, m), mix(b.sss_strength, a.sss_strength, m), mix(b.sss_radius, a.sss_radius, m));
}

fn blend_distance_info(a: DistanceInfo, b: DistanceInfo, op: SdOp) -> DistanceInfo {
    let blend = select_op(op.id, op.data, op.rev, a.dist, b.dist);
    let d_blend = blend.x;
    let m = blend.y;

    return DistanceInfo(d_blend,blend_material(a.material, b.material, m));
}

fn map(p: vec3f) -> DistanceInfo {
    let n_shapes = arrayLength(&sd_shapes);
    let n_ops = arrayLength(&sd_ops);
    
    for (var e = 0u; e < n_ops; e++) {
        let op = unpack_sd_op_instance(sd_ops[e]);
        var lhs_info: DistanceInfo;
        var rhs_info: DistanceInfo;

        // Handle LHS
        if (op.lhs < n_shapes) {
            lhs_info = shape_to_dist(unpack_sd_shape_instance(sd_shapes[op.lhs]), p);
        } else {
            lhs_info = unpack_distance_info(op_results[op.lhs - n_shapes]);
        }

        // Handle RHS
        if (op.rhs < n_shapes) {
            rhs_info = shape_to_dist(unpack_sd_shape_instance(sd_shapes[op.rhs]), p);
        } else {
            rhs_info = unpack_distance_info(op_results[op.rhs - n_shapes]);
        }

        // Blend both sides using the current op
        let result = blend_distance_info(lhs_info, rhs_info, op.op);
        op_results[e] = pack_distance_info(result);
    }

    return unpack_distance_info(op_results[n_ops - 1u]);
}

fn march(ro: vec3f, rd: vec3f) -> MarchOutput {
    var s: f32 = 0.0;
    var p: vec3f;
    var mat: SdMaterial;

    for (var i = 0u; i <= settings.max_steps; i++) {
        p = ro + rd * s;
        let hit = map(p);
        s += hit.dist;
        mat = hit.material;

        if hit.dist < settings.eps || s > settings.max_distance {
            break;
        }
    }

    if s > settings.max_distance {
        return MarchOutput(vec3f(0.0), p, s, mat); // No hit
    }

    // Backstep
    let backstep = 0.5 * settings.eps;
    p = p - rd * backstep;

    return MarchOutput(normal(p), p, s - backstep, mat);
}

fn normal(p: vec3f) -> vec3f {
    let h = settings.normal_eps;
    let k = vec2(1., -1.);
    return normalize(
        k.xyy * map(p + k.xyy * h).dist +
        k.yyx * map(p + k.yyx * h).dist +
        k.yxy * map(p + k.yxy * h).dist +
        k.xxx * map(p + k.xxx * h).dist
    );
}

fn softshadow(
    ro: vec3f,
    rd: vec3f,
    eps: f32,
    max_dist: f32,
    max_steps: u32,
    softness: f32
) -> f32 {
    var res = 1.0;
    var t = eps;

    for (var i = 0u; i < max_steps && t < max_dist; i++) {
        let h = map(ro + rd * t).dist;
        res = min(res, h / (softness * t));
        t += clamp(h, 0.005, 0.5);

        if (res < -1.0 || t > max_dist) {
            break;
        }
    }

    res = max(res, -1.0);
    return 0.25 * (1.0 + res) * (1.0 + res) * (2.0 - res);
}

fn shadow(
    ro: vec3f,
    rd: vec3f, 
    mint: f32, 
    maxt: f32, 
    max_steps: u32
) -> f32 {
    var t = mint;
    for (var i = 0u; i < max_steps && t < maxt; i++) {
        let h = map(ro + rd * t).dist;
        if (h < 0.001) {
            return 0.0;
        }
        t = t + h;
    }
    return 1.0;
}

fn calc_ao(p: vec3f, n: vec3f) -> f32 {
    var ao = 0.0;
    var sca = 1.0;
    for (var i = 0; i < 5; i++) {
        let h = 0.01 + 0.12 * f32(i) / 4.0;
        let d = map(p + n * h).dist;
        ao = ao + (h - d) * sca;
        sca = sca * 0.95;
    }
    return clamp(1.0 - 1.5 * ao, 0.0, 1.0);
}

fn apply_lighting(
    ro: vec3f,
    rd: vec3f,
    p: vec3f,
    normal: vec3f,
    material: SdMaterial,
) -> vec3f {
    var result: vec3f = vec3f(0.0);

    // === Ambient light ===
    result += apply_ambient(material);

    // === Loop over all clusterable lights ===
    for (var i = 0u; i < arrayLength(&clusterable_objects.data); i++) {
        let light = clusterable_objects.data[i];
        if (should_skip_light(light)) {
            continue;
        }

        result += apply_light_contribution(i, ro, rd, p, normal, material);
    }

    return result;
}

fn apply_ambient(material: SdMaterial) -> vec3f {
    let ambient = lights.ambient_color.rgb / 1000.0;
    return material.color.rgb * ambient;
}

fn should_skip_light(light: ClusterableObject) -> bool {
    return light.color_inverse_square_range.w <= 0.0 ||
           all(light.color_inverse_square_range.rgb == vec3f(0.0));
}

fn apply_light_contribution(
    i: u32,
    ro: vec3f,
    rd: vec3f,
    p: vec3f,
    normal: vec3f,
    material: SdMaterial,
) -> vec3f {
    let light = clusterable_objects.data[i];
    let light_pos = light.position_radius.xyz;
    let light_color = light.color_inverse_square_range.rgb / 3000.0;
    let light_range_inv_sq = light.color_inverse_square_range.w;

    let to_light = light_pos - p;
    let dist_sq = dot(to_light, to_light);
    let light_dir = normalize(to_light);
    let dist = sqrt(dist_sq);

    // === AO and Shadowing ===
    let ao = calc_ao(p, normal);
    let visibility = shadow(
        p + normal * settings.shadow_eps,
        light_dir,
        settings.shadow_eps,
        min(dist, settings.shadow_max_distance),
        settings.shadow_max_steps
    );

    let attenuation = getDistanceAttenuation(dist_sq, light_range_inv_sq);
    let diff = max(dot(normal, light_dir), 0.0);
    let mat_color = material.color.rgb;

    // === Subsurface Scattering Approximation ===
    let sss_contrib = compute_sss(ro, rd, p, normal, light_dir, light_color, material);

    // === Diffuse and Specular ===
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(-rd, reflect_dir), 0.0), 64.0);
    let specular_strength = material.roughness;
    let lighting = (diff + specular_strength * spec) * attenuation * visibility * ao;

    // === Combine ===
    let standard_contrib = mat_color * light_color * lighting;
    let blended = mix(standard_contrib, sss_contrib * attenuation * visibility * ao, clamp(material.sss_strength, 0.0, 1.0));
    return blended;
}

fn compute_sss(
    ro: vec3f,
    rd: vec3f,
    p: vec3f,
    normal: vec3f,
    light_dir: vec3f,
    light_color: vec3f,
    material: SdMaterial
) -> vec3f {
    var sss_contrib = vec3f(0.0);
    if (material.sss_strength <= 0.0) {
        return sss_contrib;
    }

    let sss_radius = material.sss_strength;
    let scattering_dist = max(vec3f(0.05), material.sss_radius);
    let tint = material.color.rgb;
    let sigma_s = vec3f(1.0) / scattering_dist;
    let clength = 1.0 / length(sigma_s);
    var dt = 0.01 * clength;
    var sum = vec3f(0.0);
    var norm = vec3f(0.001);
    let mult = 1.1;

    for (var t = dt; t < 5.0; t = t * mult) {
        dt = (mult - 1.0) * t;
        if (t > 20.0 * clength) { break; }

        let d0 = map(ro + t * rd).dist;
        if (d0 > 0.0) { break; }

        let l1 = clength;
        let ds = map(ro + t * rd + l1 * light_dir).dist;
        let denom = ds - d0;
        if (abs(denom) < 0.0001) { continue; }

        let t1 = -d0 * l1 / denom;
        if (t1 < 0.0) { continue; }

        let T0 = exp(-t * sigma_s);
        let T = exp(-(t + t1) * sigma_s);
        sum += T * dt * sigma_s * tint;
        norm += T0 * dt * sigma_s;
    }

    sum += (1.0 - norm) * clamp(dot(normal, light_dir), 0.0, 1.0);
    sss_contrib = light_color * sum;

    return sss_contrib;
}


@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) id: vec3u) {
    
    let buffer_size = view.viewport.zw;

    let frag_coord = vec2f(id.xy);
    let uv = frag_coord / buffer_size;

    var march_uv = uv * 2.0 - 1.0;
    march_uv.y *= -1.0;
    march_uv /= settings.depth_scale;
    
    let downscaled_coord = ( frag_coord - ((buffer_size / 2.0)  * (1.0 - settings.depth_scale)) ) / settings.depth_scale ;

    // prevent overdraw
    if (any(march_uv >= vec2(1.)) || any(march_uv <= vec2(-1.))) {
        return;
    }
    
    let temp = view.world_from_clip * vec4f(march_uv, 1.0, 1.0);
    let ro = temp.xyz / temp.w;
    let rd = normalize(ro * view.world_from_clip[2].w - view.world_from_clip[2].xyz);

    let m = march(ro, rd);
    let world_depth = textureLoad(depth_texture, vec2u(downscaled_coord), 0);

    let material = apply_lighting(m.pos -0.1*m.normal, rd, m.pos, m.normal, m.material);

    let p_clip = view.clip_from_world * vec4f(m.pos, 1.0);
    let p_ndc = p_clip / p_clip.w;
    let ray_depth = p_ndc.z;

    let depth = select(ray_depth, -1.0, m.depth > settings.max_distance);

    textureStore(scaled_depth_prepass, id.xy, vec4f(ray_depth));
    textureStore(scaled_normal_prepass, id.xy, vec4f(vec3f(m.normal), 1.));
    textureStore(scaled_material_prepass, id.xy, vec4f(vec3f(material), 1.));

    let mask = f32(depth < world_depth);

    textureStore(scaled_mask_prepass, id.xy, vec4f(mask));
}

@compute @workgroup_size(8, 8, 1)
fn scale(@builtin(global_invocation_id) id: vec3u) {
    let buffer_size = view.viewport.zw; 
    let frag_coord = vec2f(id.xy);
    let upscaled_coord = (frag_coord - (buffer_size *0.5) ) * settings.depth_scale + (buffer_size *0.5);
    
    let depth_pass = textureLoad(scaled_depth_prepass, vec2u(upscaled_coord));
    let normal_pass = textureLoad(scaled_normal_prepass, vec2u(upscaled_coord));
    let material_pass = textureLoad(scaled_material_prepass, vec2u(upscaled_coord));
    let mask_pass = textureLoad(scaled_mask_prepass, vec2u(upscaled_coord));

    textureStore(depth_prepass, id.xy, depth_pass);
    textureStore(normal_prepass, id.xy, normal_pass);
    textureStore(material_prepass, id.xy, material_pass);
    textureStore(mask_prepass, id.xy, mask_pass);
}
