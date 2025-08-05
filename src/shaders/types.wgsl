#define_import_path bevy_sdf::types

struct SdObject {
    shape: SdShape,
    material: SdMaterial,
    modifiers: SdModStack,
    transform: SdTransform,
}

struct SdObjectPacked {
    shape: SdShape,
    material: SdMaterialPacked,
    modifiers: SdModStackPacked,
    transform: SdTransform,
}

fn unpack_sd_object(packed: SdObjectPacked) -> SdObject {
    return SdObject(
        packed.shape,
        unpack_sd_material(packed.material),
        unpack_sd_mod_stack(packed.modifiers),
        packed.transform
    );
}

struct SdShape {
    id: u32,
    data: mat3x3f,
}

struct SdMaterial {
    color: vec4f,
    roughness: f32,
    fresnel: f32,
    metallic: f32,
    sss_strength: f32,
    sss_radius: vec3f,
}

struct SdMaterialPacked {
    color: u32,
    rough_fres_metal: u32,
    sss_strength_radius: u32,
}

fn unpack_sd_material(packed: SdMaterialPacked) -> SdMaterial {
    let rgba = unpack4x8unorm(packed.color);
    let r_f_m = unpack4x8unorm(packed.rough_fres_metal);
    let sss_st_rgb = unpack4x8unorm(packed.sss_strength_radius);
    return SdMaterial(rgba, r_f_m.r, r_f_m.g, r_f_m.b, sss_st_rgb.r, sss_st_rgb.gba);
}

fn pack_sd_material(material: SdMaterial) -> SdMaterialPacked {
    let color = pack4x8unorm(material.color);
    let rough_fres_metal = pack4x8unorm(vec4f(
        material.roughness,
        material.fresnel,
        material.metallic,
        0.
    ));
    let sss_strength_radius = pack4x8unorm(vec4f(
        material.sss_strength,
        material.sss_radius.x,
        material.sss_radius.y,
        material.sss_radius.z,
    ));
    return SdMaterialPacked(color, rough_fres_metal, sss_strength_radius);
}

struct SdMod {
    id: u32,
    data: vec4f,
}

struct SdModStack {
    start_index: u32,
    len: u32,
}

struct SdModStackPacked {
    start_index_and_length: u32,
}

fn unpack_sd_mod_stack(packed: SdModStackPacked) -> SdModStack {
    let start_index = packed.start_index_and_length >> 16u;
    let len = packed.start_index_and_length & 0xFFFFu;

    return SdModStack(
        start_index,
        len,
    );
}

struct SdTransform {
    pos: vec3f,
    rot: vec3f,
}

struct SdOpInstance {
    op: SdOp,
    lhs: u32,
    rhs: u32,
}

struct SdOpInstancePacked {
    op: SdOpPacked,
    lhs_rhs: u32,
}

fn unpack_sd_op_instance(packed: SdOpInstancePacked) -> SdOpInstance {
    let op = unpack_sd_op(packed.op);

    let lhs = packed.lhs_rhs & 0x0000FFFFu;              // lower 16 bits
    let rhs = (packed.lhs_rhs >> 16) & 0x0000FFFFu;      // upper 16 bits

    return SdOpInstance(op, lhs, rhs);
}

struct SdOp {
    id: u32,
    rev: bool,
    data: vec2f,
}

struct SdOpPacked {
    id_data: u32
}

fn unpack_sd_op(packed: SdOpPacked) -> SdOp {
    let id = packed.id_data & 0x000000FFu;               // bits 0..7
    let rev = (packed.id_data >> 8) & 0x000000FFu;       // bits 8..15

    let data_approx = f32((packed.id_data >> 16) & 0x0000FFFFu) / 255.0;
    return SdOp(id, bool(rev), vec2f(data_approx, 0.0)); // second float unused here
}

struct MarchOutput {
    normal: vec3f,
    pos: vec3f,
    depth: f32,
    material: SdMaterial,
}

struct DistanceInfo {
    dist: f32,
    material: SdMaterial,
}

struct DistanceInfoPacked {
    dist: f32,
    material: SdMaterialPacked,
}

fn unpack_distance_info(packed: DistanceInfoPacked) -> DistanceInfo {
    return DistanceInfo(packed.dist, unpack_sd_material(packed.material));
}

fn pack_distance_info(info: DistanceInfo) -> DistanceInfoPacked {
    return DistanceInfoPacked(
        info.dist,
        pack_sd_material(info.material)
    );
}


