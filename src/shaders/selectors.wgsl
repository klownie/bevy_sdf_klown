#define_import_path bevy_sdf::selectors

#import bevy_sdf::bindings::{
    sd_mod,
    sd_field_data,
}

#import bevy_sdf::utils::{
    // Signed distance primitives
    sdBunny, sdBox, sdBoxFrame, sdCappedCone, sdCappedTorus, sdCappedVerticalCone,
    sdCapsule, sdCone, sdConeBound, sdEllipsoid, sdGyroid, sdHexPrism, sdInfiniteCone,
    sdInfiniteCylinder, sdJuliaQuaternion, sdLink, sdMandelbulb, sdMengerSponge,
    sdOctahedron, sdOctahedronBound, sdPlane, sdCylinder, sdPyramid, sdRoundBox, sdRoundCone,
    sdRoundVerticalCone, sdRoundedCylinder, sdSolidAngle, sdSphere, sdTorus,
    sdTriPrism, sdVerticalCapsule, sdVerticalCylinder, udTriangle,
}

#import bevy_sdf::utils::{
    // Operations
    opChamferIntersect, opChamferSubtract, opChamferUnion,
    opCheapBend, opDisplace, opElongate, opElongateCorrect, opInfArray, opIntersect,
    opLimArray, opRotateE, opRotateEuler, opRotateX, opRotateY, opRotateZ,
    opScale, opSmoothIntersect, opSmoothSubtract, opSmoothUnion, opSubtract,
    opSymmetryX, opSymmetryY, opSymmetryZ, opTranslate, opTwist,
    opUnion, op90RotateX, op90RotateY, op90RotateZ,
}

#import bevy_sdf::types::{
    SdBlend, SdMod, SdModStack, SdShape, SdTransform,
}

fn select_shape(p: vec3f, shape: SdShape, transform: SdTransform, modifiers: SdModStack) -> f32 {
    var pos = apply_transform(p, transform);

    for (var i = 0u; i < modifiers.len; i = i + 1u) {
        pos = apply_mod(pos, sd_mod[modifiers.start_index + i]);
    }

    switch shape.type_id {
        case 0u, default {
            return sdSphere(pos, sd_field_data[shape.data_index]);
        }
        case 1u {
            return sdEllipsoid(pos, vec3f(
                sd_field_data[shape.data_index + 0],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2]
            ));
        }
        case 2u {
            return sdBox(pos, vec3f(
                sd_field_data[shape.data_index + 0],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2]
            ));
        }
        case 3u {
            return sdRoundBox(pos, vec3f(
                sd_field_data[shape.data_index + 0],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2]
            ), sd_field_data[shape.data_index + 3]);
        }
        case 4u {
            return sdBoxFrame(pos, vec3f(
                sd_field_data[shape.data_index + 0],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2]
            ), sd_field_data[shape.data_index + 3]);
        }
        case 5u {
            return sdGyroid(pos, sd_field_data[shape.data_index]);
        }
        case 6u {
            return sdTorus(pos,
                sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1]
            );
        }
        case 7u {
            return sdCappedTorus(pos,
                sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1],
                vec2f(sd_field_data[shape.data_index + 2], sd_field_data[shape.data_index + 3])
            );
        }
        case 8u {
            return sdLink(pos,
                sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2]
            );
        }
        case 9u {
            return sdVerticalCapsule(pos,
                sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1]
            );
        }
        case 10u {
            return sdCapsule(pos,
                vec3f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                ),
                vec3f(
                    sd_field_data[shape.data_index + 3],
                    sd_field_data[shape.data_index + 4],
                    sd_field_data[shape.data_index + 5]
                ),
                sd_field_data[shape.data_index + 6]
            );
        }
        case 11u {
            return sdCylinder(pos,
                vec3f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                ),
                vec3f(
                    sd_field_data[shape.data_index + 3],
                    sd_field_data[shape.data_index + 4],
                    sd_field_data[shape.data_index + 5]
                ),
                sd_field_data[shape.data_index + 6]
            );
        }
        case 12u {
            return sdVerticalCylinder(pos,
                sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1]
            );
        }
        case 13u {
            return sdRoundedCylinder(pos,
                sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2]
            );
        }
        case 14u {
            return sdInfiniteCylinder(pos,
                vec3f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                )
            );
        }
        case 15u {
            return sdCone(pos,
                sd_field_data[shape.data_index],
                vec2f(
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                )
            );
        }
        case 16u {
            return sdConeBound(pos,
                sd_field_data[shape.data_index],
                vec2f(
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                )
            );
        }
        case 17u {
            return sdInfiniteCone(pos,
                vec2f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1]
                )
            );
        }
        case 18u {
            return sdCappedVerticalCone(pos,
                sd_field_data[shape.data_index + 0],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2]
            );
        }
        case 19u {
            return sdCappedCone(pos,
                vec3f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                ),
                vec3f(
                    sd_field_data[shape.data_index + 3],
                    sd_field_data[shape.data_index + 4],
                    sd_field_data[shape.data_index + 5]
                ),
                sd_field_data[shape.data_index + 6],
                sd_field_data[shape.data_index + 7]
            );
        }
        case 20u {
            return sdRoundVerticalCone(pos,
                sd_field_data[shape.data_index + 0],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2]
            );
        }
        case 21u {
            return sdRoundCone(pos,
                vec3f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                ),
                vec3f(
                    sd_field_data[shape.data_index + 3],
                    sd_field_data[shape.data_index + 4],
                    sd_field_data[shape.data_index + 5]
                ),
                sd_field_data[shape.data_index + 6],
                sd_field_data[shape.data_index + 7]
            );
        }
        case 22u {
            return sdSolidAngle(pos,
                vec2f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1]
                ),
                sd_field_data[shape.data_index + 2]
            );
        }
        case 23u {
            return sdPlane(pos,
                vec3f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                ),
                sd_field_data[shape.data_index + 3]
            );
        }
        case 24u {
            return sdOctahedron(pos, sd_field_data[shape.data_index]);
        }
        case 25u {
            return sdOctahedronBound(pos, sd_field_data[shape.data_index]);
        }
        case 26u {
            return sdPyramid(pos, sd_field_data[shape.data_index]);
        }
        case 27u {
            return sdHexPrism(pos,
                vec2f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1]
                )
            );
        }
        case 28u {
            return sdTriPrism(pos,
                vec2f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1]
                )
            );
        }
        case 29u {
            return udTriangle(pos,
                vec3f(
                    sd_field_data[shape.data_index + 0],
                    sd_field_data[shape.data_index + 1],
                    sd_field_data[shape.data_index + 2]
                ),
                vec3f(
                    sd_field_data[shape.data_index + 3],
                    sd_field_data[shape.data_index + 4],
                    sd_field_data[shape.data_index + 5]
                ),
                vec3f(
                    sd_field_data[shape.data_index + 6],
                    sd_field_data[shape.data_index + 7],
                    sd_field_data[shape.data_index + 8]
                )
            );
        }
        case 30u {
            return sdBunny(pos / sd_field_data[shape.data_index]) * sd_field_data[shape.data_index];
        }
        case 31u {
            return sdMandelbulb(
                pos / sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1],
                sd_field_data[shape.data_index + 2],
                sd_field_data[shape.data_index + 3]
            ) * sd_field_data[shape.data_index];
        }
        case 32u {
            return sdJuliaQuaternion(
                pos / sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1]
            ) * sd_field_data[shape.data_index];
        }
        case 33u {
            return sdMengerSponge(
                pos / sd_field_data[shape.data_index],
                sd_field_data[shape.data_index + 1]
            ) * sd_field_data[shape.data_index];
        }
    }
}
 
fn apply_mod(p: vec3f, modifier: SdMod) -> vec3f {
    switch modifier.type_id {
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

fn select_blend(op: SdBlend, d1: f32, d2: f32) -> vec2f {
    switch op.type_id {
        case 0u, default {
            return opUnion(d1, d2);
        }
        case 1u { // Subtract
            if op.rev {
                return opSubtract(d1, d2);
            } else {
                return opSubtract(d2, d1);
            };
        }
        case 2u {
            return opIntersect(d1, d2);
        }
        case 3u {
            return opChamferUnion(d1, d2, op.data.x);
        }
        case 4u { // ChamferSubtract
            if op.rev {
                return opChamferSubtract(d1, d2, op.data.x);
            } else {
                return opChamferSubtract(d2, d1, op.data.x);
            };
        }
        case 5u {
            return opChamferIntersect(d1, d2, op.data.x);
        }
        case 6u {
            return opSmoothUnion(d1, d2, op.data.x);
        }
        case 7u { // SmoothSubtract
            if op.rev {
                return opSmoothSubtract(d1, d2, op.data.x);
            } else {
                return opSmoothSubtract(d2, d1, op.data.x);
            };
        }
        case 8u {
            return opSmoothIntersect(d1, d2, op.data.x);
        }
        case 9u { // Displace
            if op.rev {
                return opDisplace(d1, d2, op.data.x);
            } else {
                return opDisplace(d2, d1, op.data.x);
            };
        }
    }
}
