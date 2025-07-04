import super::utils::{sdSphere, sdEllipsoid, sdBox, sdRoundBox, sdBoxFrame, sdGyroid, sdTorus, sdCappedTorus, sdLink, sdVerticalCapsule, sdCapsule, sdCylinder, sdVerticalCylinder, sdRoundedCylinder, sdInfiniteCylinder, sdCone, sdConeBound, sdInfiniteCone, sdCappedVerticalCone, sdCappedCone, sdRoundVerticalCone, sdRoundCone, sdSolidAngle, sdPlane, sdOctahedron, sdOctahedronBound, sdPyramid, sdHexPrism, sdTriPrism, udTriangle, sdBunny, sdMandelbulb, sdJuliaQuaternion, sdMengerSponge}
import super::utils::{opUnion, opSubtract, opIntersect, opChamferUnion, opChamferSubtract, opChamferIntersect, opSmoothUnion, opSmoothSubtract, opSmoothIntersect, opDisplace, opTwist, opCheapBend, opTranslate, op90RotateX, op90RotateY, op90RotateZ, opRotateX, opRotateY, opRotateZ, opRotateE, OpRotateEuler, opScale, opSymmetryX, opSymmetryY, opSymmetryZ, opInfArray, opLimArray, opElongate, opElongateCorrect, opOnion, opExtrusion, opRevolution}
import super::types::{SdShape, SdTransform, SdMod}

fn select_shape(p: vec3f, shape: SdShape, transform: SdTransform, modifier: SdMod) -> f32 {
    let shape_data = shape.data * 0.5; //scale it to match blender
    let pos = apply_mod(p - transform.pos, modifier);
    let rot = transform.rot;
    switch shape.id {
        case 0u, default { // Sphere
            return sdSphere(pos, shape_data[0].x);
        }
        case 1u {
            return sdEllipsoid(OpRotateEuler(pos, rot), shape_data[0]);
        }
        case 2u {
            return sdBox(OpRotateEuler(pos, rot), shape_data[0]);
        }
        case 3u {
            return sdRoundBox(OpRotateEuler(pos, rot), shape_data[0], shape_data[1].x);
        }
        case 4u {
            return sdBoxFrame(OpRotateEuler(pos, rot), shape_data[0], shape_data[1].x);
        }
        case 5u {
            return sdGyroid(pos, shape_data[0].x);
        }
        case 6u {
            return sdTorus(OpRotateEuler(pos, rot), shape_data[0].x, shape_data[0].y);
        }
        case 7u {
            return sdCappedTorus(OpRotateEuler(pos, rot), shape_data[0].x, shape_data[0].y, shape_data[1].xy);
        }
        case 8u {
            return sdLink(OpRotateEuler(pos, rot), shape_data[0].x, shape_data[0].y, shape_data[0].z);
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
            return sdBunny(OpRotateEuler(pos, rot) / shape_data[0].x) * shape_data[0].x;
        }
        case 31u {
            return sdMandelbulb(OpRotateEuler(pos, rot) / shape_data[0].x ,shape_data[0].y, shape_data[0].z, shape_data[1].x) * shape_data[0].x;
        }
        case 32u {
            return sdJuliaQuaternion(OpRotateEuler(pos, rot) / shape_data[0].x ,shape_data[0].y) * shape_data[0].x;
        }
        case 33u {
            return sdMengerSponge(OpRotateEuler(pos, rot) / shape_data[0].x ,shape_data[0].y) * shape_data[0].x;
        }
    }
}

fn apply_mod(p: vec3f, modifier: SdMod) -> vec3f {
    switch modifier.id {
        case 0u { 
            return opTwist(p, modifier.data.x);
        }
        case 1u {
            return opCheapBend(p, modifier.data.x);
        }
        case 2u {
            return opSymmetryX(p);
        }
        case 3u {
            return opSymmetryY(p);
        }
        case 4u {
            return opSymmetryZ(p);
        }
        case 5u {
            return opInfArray(p, modifier.data.xyz);
        }
        case 6u {
            return opLimArray(p, modifier.data.x, modifier.data.yzw);
        }
        case 7u {
            return opElongate(p, modifier.data.xyz);
        }
        case default {
            return p;
        }
    }
}

fn select_op(op: u32, op_data: f32, rev_op: bool, d1: f32, d2: f32) -> vec2f {
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
            return opChamferUnion(d1, d2, op_data);
        }
        case 4u { // ChamferSubtract
            if rev_op {
                return opChamferSubtract(d1, d2, op_data);
            } else {
                return opChamferSubtract(d2, d1, op_data);
            };
        }
        case 5u { 
            return opChamferIntersect(d1, d2, op_data);
        }
        case 6u { 
            return opSmoothUnion(d1, d2, op_data);
        }
        case 7u { // SmoothSubtract
            if rev_op {
                return opSmoothSubtract(d1, d2, op_data);
            } else {
                return opSmoothSubtract(d2, d1, op_data);
            };
        }
        case 8u { 
            return opSmoothIntersect(d1, d2, op_data);
        }
        case 9u { // Displace
            if rev_op {
                return opDisplace(d1, d2, op_data);
            } else {
                return opDisplace(d2, d1, op_data);
            };
        }
    }
}
