#define_import_path bevy_sdf::selectors

#import bevy_sdf::utils::{sdSphere, sdEllipsoid, sdBox, sdRoundBox, sdBoxFrame, sdGyroid, sdTorus, sdCappedTorus, sdLink, sdVerticalCapsule, sdCapsule, sdCylinder, sdVerticalCylinder, sdRoundedCylinder, sdInfiniteCylinder, sdCone, sdConeBound, sdInfiniteCone, sdCappedVerticalCone, sdCappedCone, sdRoundVerticalCone, sdRoundCone, sdSolidAngle, sdPlane, sdOctahedron, sdOctahedronBound, sdPyramid, sdHexPrism, sdTriPrism, udTriangle, sdBunny, sdMandelbulb, sdJuliaQuaternion, sdMengerSponge}
#import bevy_sdf::utils::{
        opUnion, opSubtract, opIntersect, opChamferUnion, opChamferSubtract, opChamferIntersect, opSmoothUnion, opSmoothSubtract, opSmoothIntersect,
        opDisplace, opTwist, opCheapBend, opTranslate, op90RotateX, op90RotateY, op90RotateZ, opRotateX, opRotateY, opRotateZ, opRotateE, opRotateEuler, opScale, opSymmetryX, opSymmetryY, opSymmetryZ, opInfArray, opLimArray, opElongate, opElongateCorrect}
#import bevy_sdf::types::{SdShape, SdTransform, SdMod, SdModStack}


fn select_shape(p: vec3f, shape: SdShape, transform: SdTransform, modifiers: SdModStack) -> f32 {
    let shape_data = shape.data * 0.5; //scale it to match blender
    var pos = apply_transform(p, transform);

    for (var i = 0u; i < modifiers.len; i++) {
        pos = apply_mod(pos, sd_mod[modifiers.start_index + i ]);
    }
    switch shape.id {
        case 0u, default {
            return sdSphere(pos, shape_data[0].x);
        }
        case 1u {
            return sdEllipsoid(pos, shape_data[0]);
        }
        case 2u {
            return sdBox(pos, shape_data[0]);
        }
        case 3u {
            return sdRoundBox(pos, shape_data[0], shape_data[1].x);
        }
        case 4u {
            return sdBoxFrame(pos, shape_data[0], shape_data[1].x);
        }
        case 5u {
            return sdGyroid(pos, shape_data[0].x);
        }
        case 6u {
            return sdTorus(pos, shape_data[0].x, shape_data[0].y);
        }
        case 7u {
            return sdCappedTorus(pos, shape_data[0].x, shape_data[0].y, shape_data[1].xy);
        }
        case 8u {
            return sdLink(pos, shape_data[0].x, shape_data[0].y, shape_data[0].z);
        }
        case 9u {
            return sdVerticalCapsule(pos, shape_data[0].x, shape_data[0].y);
        }
        case 10u {
            return sdCapsule(pos, shape_data[0], shape_data[1], shape_data[2].x);
        }
        case 11u {
            return sdCylinder(pos, shape_data[0], shape_data[1], shape_data[2].x);
        }
        case 12u {
            return sdVerticalCylinder(pos, shape_data[0].x, shape_data[0].y);
        }
        case 13u {
            return sdRoundedCylinder(pos, shape_data[0].x, shape_data[0].y, shape_data[0].z);
        }
        case 14u {
            return sdInfiniteCylinder(pos, shape_data[0].xyz);
        }
        case 15u {
            return sdCone(pos, shape_data[0].x, shape_data[1].xy);
        }
        case 16u {
            return sdConeBound(pos, shape_data[0].x, shape_data[1].xy);
        }
        case 17u {
            return sdInfiniteCone(pos, shape_data[0].xy);
        }
        case 18u {
            return sdCappedVerticalCone(pos, shape_data[0].x, shape_data[0].y, shape_data[0].z);
        }
        case 19u {
            return sdCappedCone(pos, shape_data[0], shape_data[1], shape_data[2].x, shape_data[2].y);
        }
        case 20u {
            return sdRoundVerticalCone(pos, shape_data[0].x, shape_data[0].y, shape_data[0].z);
        }
        case 21u {
            return sdRoundCone(pos, shape_data[0], shape_data[1], shape_data[2].x, shape_data[2].y);
        }
        case 22u {
            return sdSolidAngle(pos, shape_data[0].xy, shape_data[0].z);
        }
        case 23u {
            return sdPlane(pos, shape_data[0].xyz, shape_data[1].x);
        }
        case 24u {
            return sdOctahedron(pos, shape_data[0].x);
        }
        case 25u {
            return sdOctahedronBound(pos, shape_data[0].x);
        }
        case 26u {
            return sdPyramid(pos, shape_data[0].x);
        }
        case 27u {
            return sdHexPrism(pos, shape_data[0].xy);
        }
        case 28u {
            return sdTriPrism(pos, shape_data[0].xy);
        }
        case 29u {
            return udTriangle(pos, shape_data[0], shape_data[1], shape_data[2]);
        }
        case 30u {
            return sdBunny(pos / shape_data[0].x) * shape_data[0].x;
        }
        case 31u {
            return sdMandelbulb(pos / shape_data[0].x, shape_data[0].y, shape_data[0].z, shape_data[1].x) * shape_data[0].x;
        }
        case 32u {
            return sdJuliaQuaternion(pos / shape_data[0].x, shape_data[0].y) * shape_data[0].x;
        }
        case 33u {
            return sdMengerSponge(pos / shape_data[0].x, shape_data[0].y) * shape_data[0].x;
        }
    }
}

@group(2) @binding(0) var<storage, read> sd_mod: array<SdMod>;

fn apply_mod(p: vec3f, modifier: SdMod) -> vec3f {
    switch modifier.id {
        case 0u {
            return p - modifier.data.xyz;
        }
        case 1u {
            return op90RotateX(p);
        }
        case 2u {
            return op90RotateY(p);
        }
        case 3u {
            return op90RotateZ(p);
        }
        case 4u {
            return opRotateX(p, modifier.data.x);
        }
        case 5u {
            return opRotateY(p, modifier.data.x);
        }
        case 6u {
            return opRotateZ(p, modifier.data.x);
        }
        case 7u {
            return opRotateEuler(p, modifier.data.xyz);
        }
        case 8u {
            return opTwist(p, modifier.data.x);
        }
        case 9u {
            return opCheapBend(p, modifier.data.x);
        }
        case 10u {
            return opSymmetryX(p);
        }
        case 11u {
            return opSymmetryY(p);
        }
        case 12u {
            return opSymmetryZ(p);
        }
        case 13u {
            return opInfArray(p, modifier.data.xyz);
        }
        case 14u {
            return opLimArray(p, modifier.data.x, modifier.data.yzw);
        }
        case 15u {
            return opElongate(p, modifier.data.xyz);
        }
        case default {
            return p;
        }
    }
}

fn apply_transform(p: vec3f, transform: SdTransform) -> vec3f {
    var new_p = p - transform.pos;
    if !all(transform.rot == vec3f(0.0)) {
        new_p = opRotateEuler(new_p, transform.rot);
    }
    return new_p;
}

fn select_op(op: u32, op_data: vec2f, rev_op: bool, d1: f32, d2: f32) -> vec2f {
    switch op {
        case 0u, default {
            return opUnion(d1, d2);
        }
        case 1u { // Subtract
            if rev_op {
                return opSubtract(d1, d2);
            } else {
                return opSubtract(d2, d1);
            };
        }
        case 2u {
            return opIntersect(d1, d2);
        }
        case 3u {
            return opChamferUnion(d1, d2, op_data.x);
        }
        case 4u { // ChamferSubtract
            if rev_op {
                return opChamferSubtract(d1, d2, op_data.x);
            } else {
                return opChamferSubtract(d2, d1, op_data.x);
            };
        }
        case 5u {
            return opChamferIntersect(d1, d2, op_data.x);
        }
        case 6u {
            return opSmoothUnion(d1, d2, op_data.x);
        }
        case 7u { // SmoothSubtract
            if rev_op {
                return opSmoothSubtract(d1, d2, op_data.x);
            } else {
                return opSmoothSubtract(d2, d1, op_data.x);
            };
        }
        case 8u {
            return opSmoothIntersect(d1, d2, op_data.x);
        }
        case 9u { // Displace
            if rev_op {
                return opDisplace(d1, d2, op_data.x);
            } else {
                return opDisplace(d2, d1, op_data.x);
            };
        }
    }
}
