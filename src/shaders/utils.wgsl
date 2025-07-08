#define_import_path bevy_sdf::utils

// Sphere - exact
fn sdSphere(p: vec3f, r: f32) -> f32 {
    return length(p) - r;
}

// Ellipsoid - bound (not exact)
fn sdEllipsoid(p: vec3f, r: vec3f) -> f32 {
    let k0 = length(p / r);
    let k1 = length(p / (r * r));
    return k0 * (k0 - 1.) / k1;
}

// Box - exact
fn sdBox(p: vec3f, b: vec3f) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3f(0.))) + min(max(q.x, max(q.y, q.z)), 0.);
}

// Round Box - exact
fn sdRoundBox(p: vec3f, b: vec3f, r: f32) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3f(0.))) + min(max(q.x, max(q.y, q.z)), 0.) - r;
}

// Box Frame - exact
fn sdBoxFrame(p: vec3f, b: vec3f, e: f32) -> f32 {
    let q = abs(p) - b;
    let w = abs(q + e) - e;
    return min(min(
        length(max(vec3f(q.x, w.y, w.z), vec3f(0.))) + min(max(q.x, max(w.y, w.z)), 0.),
        length(max(vec3f(w.x, q.y, w.z), vec3f(0.))) + min(max(w.x, max(q.y, w.z)), 0.)
    ),
        length(max(vec3f(w.x, w.y, q.z), vec3f(0.))) + min(max(w.x, max(w.y, q.z)), 0.));
}

// Gyroid - bound
fn sdGyroid(p: vec3f, h: f32) -> f32 {
    return abs(dot(sin(p), cos(p.zxy))) - h;
}

// Torus - exact
fn sdTorus(p: vec3f, R: f32, r: f32) -> f32 {
    let q = vec2f(length(p.xz) - R, p.y);
    return length(q) - r;
}

// Capped Torus - exact
fn sdCappedTorus(p: vec3f, R: f32, r: f32, sincos: vec2f) -> f32 {
    let q = vec3f(abs(p.x), p.y, p.z);
    let k = select(length(q.xy), dot(q.xy, sincos), sincos.y * q.x > sincos.x * q.y);
    return sqrt(dot(q, q) + R * R - 2. * R * k) - r;
}

// Link - exact
fn sdLink(p: vec3f, R: f32, r: f32, le: f32) -> f32 {
    let q = vec3f(p.x, max(abs(p.y) - le, 0.), p.z);
    return length(vec2f(length(q.xy) - R, q.z)) - r;
}

// Vertical Capsule / Line - exact
fn sdVerticalCapsule(p: vec3f, h: f32, r: f32) -> f32 {
    let q = vec3f(p.x, p.y - clamp(p.y, 0., h), p.z);
    return length(q) - r;
}

// Capsule / Line - exact
fn sdCapsule(p: vec3f, a: vec3f, b: vec3f, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0., 1.);
    return length(pa - ba * h) - r;
}

// Cylinder - exact
fn sdCylinder(p: vec3f, a: vec3f, b: vec3f, r: f32) -> f32 {
    let ba = b - a;
    let pa = p - a;
    let baba = dot(ba, ba);
    let paba = dot(pa, ba);
    let x = length(pa * baba - ba * paba) - r * baba;
    let y = abs(paba - baba * 0.5) - baba * 0.5;
    let x2 = x * x;
    let y2 = y * y * baba;
    let d = x2 * step(0., x) + y2 * step(0., y);
    let d2 = select(d, -min(x2, y2), max(x, y) < 0.);
    return sign(d2) * sqrt(abs(d2)) / baba;
}

// Vertical Cylinder - exact
fn sdVerticalCylinder(p: vec3f, h: f32, r: f32) -> f32 {
    let d = abs(vec2f(length(p.xz), p.y)) - vec2f(r, h);
    return min(max(d.x, d.y), 0.) + length(max(d, vec2f(0.)));
}

// Rounded Cylinder - exact
fn sdRoundedCylinder(p: vec3f, h: f32, r: f32, re: f32) -> f32 {
    let d = vec2f(length(p.xz) - 2. * r + re, abs(p.y) - h);
    return min(max(d.x, d.y), 0.) + length(max(d, vec2f(0.))) - re;
}

// Infinite Cylinder - exact
fn sdInfiniteCylinder(p: vec3f, c: vec3f) -> f32 {
    return length(p.xz - c.xy) - c.z;
}

// Cone - exact
fn sdCone(p: vec3f, h: f32, sincos: vec2f) -> f32 {
  // Alternatively pass q instead of (sin(alpha), cos(alpha))
    let q = h * vec2f(sincos.x / sincos.y, -1.);
    let w = vec2f(length(p.xz), p.y);
    let a = w - q * clamp(dot(w, q) / dot(q, q), 0., 1.);
    let b = w - q * vec2f(clamp(w.x / q.x, 0., 1.), 1.);
    let k = sign(q.y);
    let d = min(dot(a, a), dot(b, b));
    let s = max(k * (w.x * q.y - w.y * q.x), k * (w.y - q.y));
    return sqrt(d) * sign(s);
}

// Cone - bound (not exact)
fn sdConeBound(p: vec3f, h: f32, sincos: vec2f) -> f32 {
    return max(dot(sincos.yx, vec2f(length(p.xz), p.y)), -h - p.y);
}

// Infinite Cone - exact
fn sdInfiniteCone(p: vec3f, sincos: vec2f) -> f32 {
    let q = vec2f(length(p.xz), -p.y);
    let d = length(q - sincos * max(dot(q, sincos), 0.));
    return d * select(-1., 1., q.x * sincos.y - q.y * sincos.x > 0.0);
}

// Capped Vertical Cone - exact
fn sdCappedVerticalCone(p: vec3f, h: f32, r1: f32, r2: f32) -> f32 {
    let q = vec2f(length(p.xz), p.y);
    let k1 = vec2f(r2, h);
    let k2 = vec2f(r2 - r1, 2. * h);
    let ca = vec2f(q.x - min(q.x, select(r2, r1, q.y < 0.)), abs(q.y) - h);
    let cb = q - k1 + k2 * clamp(dot(k1 - q, k2) / dot(k2, k2), 0., 1.);
    let s = select(1., -1., cb.x < 0. && ca.y < 0.);
    return s * sqrt(min(dot(ca, ca), dot(cb, cb)));
}

// Capped Cone - exact
fn sdCappedCone(p: vec3f, a: vec3f, b: vec3f, ra: f32, rb: f32) -> f32 {
    let rba = rb - ra;
    let baba = dot(b - a, b - a);
    let papa = dot(p - a, p - a);
    let paba = dot(p - a, b - a) / baba;
    let x = sqrt(papa - paba * paba * baba);
    let cax = max(0.0, x - select(rb, ra, paba < 0.5));
    let cay = abs(paba - 0.5) - 0.5;
    let k = rba * rba + baba;
    let f = clamp((rba * (x - ra) + paba * baba) / k, 0.0, 1.0);
    let cbx = x - ra - f * rba;
    let cby = paba - f;
    let s = select(1., -1., cbx < 0.0 && cay < 0.0);
    return s * sqrt(min(cax * cax + cay * cay * baba, cbx * cbx + cby * cby * baba));
}

// Round Vertical cone - exact
fn sdRoundVerticalCone(p: vec3f, h: f32, r1: f32, r2: f32) -> f32 {
    let q = vec2f(length(p.xz), p.y);
    let b = (r1 - r2) / h;
    let a = sqrt(1. - b * b);
    let k = dot(q, vec2f(-b, a));
    if k < 0. { return length(q) - r1; }
    if k > a * h { return length(q - vec2f(0., h)) - r2; }
    return dot(q, vec2f(a, b)) - r1;
}

// Round cone - exact
fn sdRoundCone(p: vec3f, a: vec3f, b: vec3f, r1: f32, r2: f32) -> f32 {
    let ba = b - a;
    let l2 = dot(ba, ba);
    let rr = r1 - r2;
    let a2 = l2 - rr * rr;
    let il2 = 1. / l2;

    let pa = p - a;
    let y = dot(pa, ba);
    let z = y - l2;
    let w = pa * l2 - ba * y;
    let x2 = dot(w, w);
    let y2 = y * y * l2;
    let z2 = z * z * l2;

    let k = sign(rr) * rr * rr * x2;
    if sign(z) * a2 * z2 > k { return sqrt(x2 + z2) * il2 - r2; }
    if sign(y) * a2 * y2 < k { return sqrt(x2 + y2) * il2 - r1; }
    return (sqrt(x2 * a2 * il2) + y * rr) * il2 - r1;
}

// Solid Angle - exact
fn sdSolidAngle(p: vec3f, sincos: vec2f, r: f32) -> f32 {
    let q = vec2f(length(p.xz), p.y);
    let l = length(q) - r;
    let m = length(q - sincos * clamp(dot(q, sincos), 0., r));
    return max(l, m * sign(sincos.y * q.x - sincos.x * q.y));
}

// Plane - exact
fn sdPlane(p: vec3f, n: vec3f, h: f32) -> f32 {
  // n must be normalized
    return dot(p, n) + h;
}

// Octahedron - exact
fn sdOctahedron(p: vec3f, s: f32) -> f32 {
    var q: vec3f = abs(p);
    let m = q.x + q.y + q.z - s;
    if 3. * q.x < m {q = q.xyz;} else {if 3. * q.y < m {q = q.yzx;} else {if 3. * q.z < m {q = q.zxy;} else {return m * 0.57735027;}}}
    let k = clamp(0.5 * (q.z - q.y + s), 0., s);
    return length(vec3f(q.x, q.y - s + k, q.z - k));
}

// Octahedron - bound (not exact)
fn sdOctahedronBound(p: vec3f, s: f32) -> f32 {
    let q = abs(p);
    return (q.x + q.y + q.z - s) * 0.57735027;
}

// Pyramid - exact
fn sdPyramid(p: vec3f, h: f32) -> f32 {
    let m2 = h * h + 0.25;
    var xz: vec2f = abs(p.xz);
    xz = select(xz, xz.yx, xz[1] > xz[0]);
    xz = xz - vec2f(0.5);

    let q = vec3f(xz[1], h * p.y - 0.5 * xz[0], h * xz[0] + 0.5 * p.y);
    let s = max(-q.x, 0.);
    let t = clamp((q.y - 0.5 * xz[1]) / (m2 + 0.25), 0., 1.);

    let a = m2 * (q.x + s) * (q.x + s) + q.y * q.y;
    let b = m2 * (q.x + 0.5 * t) * (q.x + 0.5 * t) + (q.y - m2 * t) * (q.y - m2 * t);

    let d2 = min(a, b) * step(min(q.y, -q.x * m2 - q.y * 0.5), 0.);
    return sqrt((d2 + q.z * q.z) / m2) * sign(max(q.z, -p.y));
}

// Hexagonal Prism - exact
fn sdHexPrism(p: vec3f, h: vec2f) -> f32 {
    let k = vec3f(-0.8660254, 0.5, 0.57735);
    let a = abs(p);
    let v = a.xy - 2. * min(dot(k.xy, a.xy), 0.) * k.xy;
    let d1 = length(v - vec2f(clamp(v.x, -k.z * h.x, k.z * h.x), h.x)) * sign(v.y - h.x);
    let d2 = a.z - h.y;
    return min(max(d1, d2), 0.) + length(max(vec2f(d1, d2), vec2f(0.)));
}

// Triangular Prism - bound
fn sdTriPrism(p: vec3f, h: vec2f) -> f32 {
    let q = abs(p);
    return max(q.z - h.y, max(q.x * 0.866025 + p.y * 0.5, -p.y) - h.x * 0.5);
}

// Quadratic Bezier - exact
fn sdBezier(p: vec3f, A: vec3f, B: vec3f, C: vec3f) -> vec2f {
    let a = B - A;
    let b = A - 2. * B + C;
    let c = a * 2.;
    let d = A - p;
    let kk = 1. / dot(b, b);
    let kx = kk * dot(a, b);
    let ky = kk * (2. * dot(a, a) + dot(d, b)) / 3.;
    let kz = kk * dot(d, a);

    let p1 = ky - kx * kx;
    let p3 = p1 * p1 * p1;
    let q = kx * (2.0 * kx * kx - 3.0 * ky) + kz;
    var h: f32 = q * q + 4. * p3;

    var res: vec2f;
    if h >= 0. {
        h = sqrt(h);
        let x = (vec2f(h, -h) - q) / 2.;
        let uv = sign(x) * pow(abs(x), vec2f(1. / 3.));
        let t = clamp(uv.x + uv.y - kx, 0., 1.);
        let f = d + (c + b * t) * t;
        res = vec2f(dot(f, f), t);
    } else {
        let z = sqrt(-p1);
        let v = acos(q / (p1 * z * 2.)) / 3.;
        let m = cos(v);
        let n = sin(v) * 1.732050808;
        let t = clamp(vec2f(m + m, -n - m) * z - kx, vec2f(0.0), vec2f(1.0));
        let f = d + (c + b * t.x) * t.x;
        var dis: f32 = dot(f, f);
        res = vec2f(dis, t.x);

        let g = d + (c + b * t.y) * t.y;
        dis = dot(g, g);
        res = select(res, vec2f(dis, t.y), dis < res.x);
    }
    res.x = sqrt(res.x);
    return res;
}

// Triangle - exact
fn udTriangle(p: vec3f, a: vec3f, b: vec3f, c: vec3f) -> f32 {
    let ba = b - a; let pa = p - a;
    let cb = c - b; let pb = p - b;
    let ac = a - c; let pc = p - c;
    let nor = cross(ba, ac);
    let d1 = ba * clamp(dot(ba, pa) / dot(ba, ba), 0., 1.) - pa;
    let d2 = cb * clamp(dot(cb, pb) / dot(cb, cb), 0., 1.) - pb;
    let d3 = ac * clamp(dot(ac, pc) / dot(ac, ac), 0., 1.) - pc;
    let k0 = min(min(dot(d1, d1), dot(d2, d2)), dot(d3, d3));
    let k1 = dot(nor, pa) * dot(nor, pa) / dot(nor, nor);
    let t = sign(dot(cross(ba, nor), pa)) + sign(dot(cross(cb, nor), pb)) + sign(dot(cross(ac, nor), pc));
    return sqrt(select(k0, k1, t < 2.));
}

// Quad - exact
fn udQuad(p: vec3f, a: vec3f, b: vec3f, c: vec3f, d: vec3f) -> f32 {
    let ba = b - a; let pa = p - a;
    let cb = c - b; let pb = p - b;
    let dc = d - c; let pc = p - c;
    let ad = a - d; let pd = p - d;
    let nor = cross(ba, ad);
    let d1 = ba * clamp(dot(ba, pa) / dot(ba, ba), 0., 1.) - pa;
    let d2 = cb * clamp(dot(cb, pb) / dot(cb, cb), 0., 1.) - pb;
    let d3 = dc * clamp(dot(dc, pc) / dot(dc, dc), 0., 1.) - pc;
    let d4 = ad * clamp(dot(ad, pd) / dot(ad, ad), 0., 1.) - pd;
    let k0 = min(min(dot(d1, d1), dot(d2, d2)), min(dot(d3, d3), dot(d4, d4)));
    let k1 = dot(nor, pa) * dot(nor, pa) / dot(nor, nor);
    let t = sign(dot(cross(ba, nor), pa)) + sign(dot(cross(cb, nor), pb)) + sign(dot(cross(dc, nor), pc)) + sign(dot(cross(ad, nor), pd));
    return sqrt(select(k0, k1, t < 3.));
}

fn sdBunny(p: vec3f) -> f32 {
    if dot(p, p) > 1. { return length(p) - .8; }
    let q = vec4f(p, 1.);
    let f00 = sin(mat4x4f(-3.02, 1.95, -3.42, -.6, 3.08, .85, -2.25, -.24, -.29, 1.16, -3.74, 2.89, -.71, 4.5, -3.24, -3.5) * q);
    let f01 = sin(mat4x4f(-.4, -3.61, 3.23, -.14, -.36, 3.64, -3.91, 2.66, 2.9, -.54, -2.75, 2.71, 7.02, -5.41, -1.12, -7.41) * q);
    let f02 = sin(mat4x4f(-1.77, -1.28, -4.29, -3.2, -3.49, -2.81, -.64, 2.79, 3.15, 2.14, -3.85, 1.83, -2.07, 4.49, 5.33, -2.17) * q);
    let f03 = sin(mat4x4f(-.49, .68, 3.05, .42, -2.87, .78, 3.78, -3.41, -2.65, .33, .07, -.64, -3.24, -5.9, 1.14, -4.71) * q);
    let f10 = sin(mat4x4f(-.34, .06, -.59, -.76, .1, -.19, -.12, .44, .64, -.02, -.26, .15, -.16, .21, .91, .15) * f00 + mat4x4f(.01, .54, -.77, .11, .06, -.14, .43, .51, -.18, .08, .39, .2, .33, -.49, -.1, .19) * f01 + mat4x4f(.27, .22, .43, .53, .18, -.17, .23, -.64, -.14, .02, -.1, .16, -.13, -.06, -.04, -.36) * f02 + mat4x4f(-.13, .29, -.29, .08, 1.13, .02, -.83, .32, -.32, .04, -.31, -.16, .14, -.03, -.2, .39) * f03 + vec4f(.73, -4.28, -1.56, -1.8)) + f00;
    let f11 = sin(mat4x4f(-1.11, .55, -.12, -1.00, .16, .15, -.3, .31, -.01, .01, .31, -.42, -.29, .38, -.04, .71) * f00 + mat4x4f(.96, -.02, .86, .52, -.14, .6, .44, .43, .02, -.15, -.49, -.05, -.06, -.25, -.03, -.22) * f01 + mat4x4f(.52, .44, -.05, -.11, -.56, -.1, -.61, -.4, -.04, .55, .32, -.07, -.02, .28, .26, -.49) * f02 + mat4x4f(.02, -.32, .06, -.17, -.59, .00, -.24, .6, -.06, .13, -.21, -.27, -.12, -.14, .58, -.55) * f03 + vec4f(-2.24, -3.48, -.8, 1.41)) + f01;
    let f12 = sin(mat4x4f(.44, -.06, -.79, -.46, .05, -.6, .3, .36, .35, .12, .02, .12, .4, -.26, .63, -.21) * f00 + mat4x4f(-.48, .43, -.73, -.4, .11, -.01, .71, .05, -.25, .25, -.28, -.2, .32, -.02, -.84, .16) * f01 + mat4x4f(.39, -.07, .9, .36, -.38, -.27, -1.86, -.39, .48, -.2, -.05, .1, -.00, -.21, .29, .63) * f02 + mat4x4f(.46, -.32, .06, .09, .72, -.47, .81, .78, .9, .02, -.21, .08, -.16, .22, .32, -.13) * f03 + vec4f(3.38, 1.2, .84, 1.41)) + f02;
    let f13 = sin(mat4x4f(-.41, -.24, -.71, -.25, -.24, -.75, -.09, .02, -.27, -.42, .02, .03, -.01, .51, -.12, -1.24) * f00 + mat4x4f(.64, .31, -1.36, .61, -.34, .11, .14, .79, .22, -.16, -.29, -.70, .02, -.37, .49, .39) * f01 + mat4x4f(.79, .47, .54, -.47, -1.13, -.35, -1.03, -.22, -.67, -.26, .1, .21, -.07, -.73, -.11, .72) * f02 + mat4x4f(.43, -.23, .13, .09, 1.38, -.63, 1.57, -.2, .39, -.14, .42, .13, -.57, -.08, -.21, .21) * f03 + vec4f(-.34, -3.28, .43, -.52)) + f03;
    let f20 = sin(mat4x4f(-.72, .23, -.89, .52, .38, .19, -.16, -.88, .26, -.37, .09, .63, .29, -.72, .3, -.95) * f10 + mat4x4f(-.22, -.51, -.42, -.73, -.32, .00, -1.03, 1.17, -.2, -.03, -.13, -.16, -.41, .09, .36, -.84) * f11 + mat4x4f(-.21, .01, .33, .47, .05, .2, -.44, -1.04, .13, .12, -.13, .31, .01, -.34, .41, -.34) * f12 + mat4x4f(-.13, -.06, -.39, -.22, .48, .25, .24, -.97, -.34, .14, .42, -.00, -.44, .05, .09, -.95) * f13 + vec4f(.48, .87, -.87, -2.06)) / 1.4 + f10;
    let f21 = sin(mat4x4f(-.27, .29, -.21, .15, .34, -.23, .85, -.09, -1.15, -.24, -.05, -.25, -.12, -.73, -.17, -.37) * f10 + mat4x4f(-1.11, .35, -.93, -.06, -.79, -.03, -.46, -.37, .6, -.37, -.14, .45, -.03, -.21, .02, .59) * f11 + mat4x4f(-.92, -.17, -.58, -.18, .58, .6, .83, -1.04, -.8, -.16, .23, -.11, .08, .16, .76, .61) * f12 + mat4x4f(.29, .45, .3, .39, -.91, .66, -.35, -.35, .21, .16, -.54, -.63, 1.1, -.38, .2, .15) * f13 + vec4f(-1.72, -.14, 1.92, 2.08)) / 1.4 + f11;
    let f22 = sin(mat4x4f(1.00, .66, 1.3, -.51, .88, .25, -.67, .03, -.68, -.08, -.12, -.14, .46, 1.15, .38, -.1) * f10 + mat4x4f(.51, -.57, .41, -.09, .68, -.5, -.04, -1.01, .2, .44, -.6, .46, -.09, -.37, -1.3, .04) * f11 + mat4x4f(.14, .29, -.45, -.06, -.65, .33, -.37, -.95, .71, -.07, 1.00, -.6, -1.68, -.2, -.00, -.7) * f12 + mat4x4f(-.31, .69, .56, .13, .95, .36, .56, .59, -.63, .52, -.3, .17, 1.23, .72, .95, .75) * f13 + vec4f(-.9, -3.26, -.44, -3.11)) / 1.4 + f12;
    let f23 = sin(mat4x4f(.51, -.98, -.28, .16, -.22, -.17, -1.03, .22, .7, -.15, .12, .43, .78, .67, -.85, -.25) * f10 + mat4x4f(.81, .6, -.89, .61, -1.03, -.33, .6, -.11, -.06, .01, -.02, -.44, .73, .69, 1.02, .62) * f11 + mat4x4f(-.1, .52, .8, -.65, .4, -.75, .47, 1.56, .03, .05, .08, .31, -.03, .22, -1.63, .07) * f12 + mat4x4f(-.18, -.07, -1.22, .48, -.01, .56, .07, .15, .24, .25, -.09, -.54, .23, -.08, .2, .36) * f13 + vec4f(-1.11, -4.28, 1.02, -.23)) / 1.4 + f13;
    return dot(f20, vec4f(.09, .12, -.07, -.03)) + dot(f21, vec4f(-.04, .07, -.08, .05)) + dot(f22, vec4f(-.01, .06, -.02, .07)) + dot(f23, vec4f(-.05, .07, .03, .04)) - 0.16;
}

// === Fractals ===

fn sdMandelbulb(p: vec3f, iter: f32, expo: f32, b_offset: f32) -> f32 {
    let pw = (expo - 1.0) / 2.0;

    var z = p;
    var m = dot(z, z);
    var dz = 1.0;

    for (var i = 0u; i < u32(iter); i++) {
        dz = dz * expo * pow(m, pw) + 1.0;

        let r = length(z);
        if (r == 0.0) {
            break;
        }

        var b = expo * acos(z.y / r);
        b += b_offset;

        let a = expo * atan2(z.x, z.z);

        z = p + pow(r, expo) * vec3f(sin(b) * sin(a), cos(b), sin(b) * cos(a));
        m = dot(z, z);

        if (m > 256.0) {
            break;
        }
    }

    return 0.25 * log(m) * sqrt(m) / dz;
}

fn sdJuliaQuaternion(p: vec3f, iter: f32) -> f32 {
    let c = vec4f(-0.5, 0.5, -0.4, -0.2);

    var z = vec4f(p, 0.0);
    var z2: vec4f;

    var md2 = 1.0;
    var mz2 = dot(z, z);

    for (var i: u32 = 0u; i < u32(iter); i++) {
        md2 *= 4.0 * mz2;

        z2.x = z.x * z.x - dot(z.yzw, z.yzw);
        let yz = vec3f(z.y, z.z, z.w);
        let res = 2.0 * z.x * yz;
        z2.y = res.x;
        z2.z = res.y;
        z2.w = res.z;
        z = z2 + c;

        mz2 = dot(z, z);
        if (mz2 > 4.0) {
            break;
        }
    }

    return 0.25 * sqrt(mz2 / md2) * log(mz2);
}

fn sdMengerSponge(p: vec3f, iter: f32) -> f32 {
    var d = sdBox(p, vec3f(1.0));
    var s = 1.0;

    for (var m = 0u; m < u32(iter); m++) {
        let a = fract(p * s * 0.5) * 2.0 - 1.0;
        s *= 3.0;
        let r = abs(1.0 - 3.0 * abs(a));

        let da = max(r.x, r.y);
        let db = max(r.y, r.z);
        let dc = max(r.z, r.x);
        let c = (min(da, min(db, dc)) - 1.0) / s;

        if (c > d) {
            d = c;
        }
    }

    return d;
}

// === Boolean operations with primitives ===

// Union, Subtraction, Intersection - exact (outside), bound, bound
fn opUnion(d1: f32, d2: f32) -> vec2f {
    if d1 < d2 {
        return vec2f(d1, 1.0); // d1 wins
    } else {
        return vec2f(d2, 0.0); // d2 wins
    }
}

fn opSubtract(d1: f32, d2: f32) -> vec2f {
    if d1 > -d2 {
        return vec2f(d1, 0.0); // d1 remains
    } else {
        return vec2f(-d2, 0.0); // inverted d2 wins
    }
}

fn opIntersect(d1: f32, d2: f32) -> vec2f {
    if d1 > d2 {
        return vec2f(d1, 1.0); // d1 wins (greater value)
    } else {
        return vec2f(d2, 0.0); // d2 wins
    }
}

// Chamfer Union, Chamfer Subtraction, Chamfer Intersection - bound, bound, bound
fn opChamferUnion(d1: f32, d2: f32, r: f32) -> vec2f {
    let raw = (d1 - r + d2) * 0.5;
    let d = min(min(d1, d2), raw);

    var blend: f32;
    if d == d1 {
        blend = 1.0;
    } else if d == d2 {
        blend = 0.0;
    } else {
        blend = 0.5; // blended region
    }

    return vec2f(d, blend);
}

fn opChamferSubtract(d1: f32, d2: f32, r: f32) -> vec2f {
    let raw = (d1 + r - d2) * 0.5;
    let d = max(max(d1, -d2), raw);

    return vec2f(d, 0.0);
}

fn opChamferIntersect(d1: f32, d2: f32, r: f32) -> vec2f {
    let raw = (d1 + r + d2) * 0.5;
    let d = max(max(d1, d2), raw);

    var blend: f32;
    if d == d1 {
        blend = 1.0;
    } else if d == d2 {
        blend = 0.0;
    } else {
        blend = 0.0;
    }

    return vec2f(d, blend);
}

// Blend Union, Blend Subtraction, Blend Intersection - bound, bound, bound
fn opSmoothUnion(d1: f32, d2: f32, k: f32) -> vec2f {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    let dist = mix(d2, d1, h) - k * h * (1.0 - h);
    return vec2f(dist, h);
}

fn opSmoothSubtract(d1: f32, d2: f32, k: f32) -> vec2f {
    let h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    let dist = mix(d1, -d2, h) + k * h * (1.0 - h);
    return vec2f(dist, 0.0);
}

fn opSmoothIntersect(d1: f32, d2: f32, k: f32) -> vec2f {
    let h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    let dist = mix(d2, d1, h) + k * h * (1.0 - h);
    return vec2f(dist, h);
}

// === Displacement ===

// Displacement - bound (not exact)
fn opDisplace(d1: f32, d2: f32, strength: f32) -> vec2f {
    let d2_scaled = d2 * strength;
    let displaced = d1 + d2_scaled;
    let influence = clamp(abs(d2_scaled) / (abs(d1) + abs(d2_scaled) + 0.0001), 0.0, 1.0);
    return vec2f(displaced, influence);
}
//let d = opDisplace(sdfPrimitive3d(p), displacement3d(p));

// Twist - bound
fn opTwist(p: vec3f, k: f32) -> vec3f {
  let s = sin(k * p.z);
  let c = cos(k * p.z);
  let m = mat2x2<f32>(vec2f(c, s), vec2f(-s, c));
  return vec3f(m * p.xy, p.z);
}
//let d = sdfPrimitive3d(opTwist(p, k));

// Bend - bound
fn opCheapBend(p: vec3f, k: f32) -> vec3f {
  let s = sin(k * p.x);
  let c = cos(k * p.x);
  let m = mat2x2<f32>(vec2f(c, s), vec2f(-s, c));
  return vec3f(m * p.xy, p.z);
}
//let d = sdfPrimitive3d(opCheapBend(p, k));

// === Positioning ===

// Translate - exact
fn opTranslate(p: vec3f, t: vec3f) -> vec3f {
    return p - t;
}
//let d = sdfPrimitive3d(opTranslate(p, t));

// 90 degree rotation - exact
fn op90RotateX(p: vec3f) -> vec3f {
    return vec3f(p.x, p.z, -p.y);
}
fn op90RotateY(p: vec3f) -> vec3f {
    return vec3f(-p.z, p.y, p.x);
}
fn op90RotateZ(p: vec3f) -> vec3f {
    return vec3f(p.y, -p.x, p.z);
}
//let d = sdfPrimitive3d(op90RotateZ(p));

// Rotation around axis - exact
fn opRotateX(p: vec3f, a: f32) -> vec3f {
    let s = sin(a); let c = cos(a);
    return vec3f(p.x, c * p.y + s * p.z, -s * p.y + c * p.z);
}
fn opRotateY(p: vec3f, a: f32) -> vec3f {
    let s = sin(a); let c = cos(a);
    return vec3f(c * p.x - s * p.z, p.y, s * p.x + c * p.z);
}
fn opRotateZ(p: vec3f, a: f32) -> vec3f {
    let s = sin(a); let c = cos(a);
    return vec3f(c * p.x + s * p.y, -s * p.x + c * p.y, p.z);
}

// Rotation around free axis - exact
fn opRotateE(p: vec3f, e: vec3f, a: f32) -> vec3f {
    let c = cos(a);
    return dot(e, p) * (1. - c) * e - cross(e, p) * sin(a) + c * p;
}

fn OpRotateEuler(p: vec3f, e: vec3f) -> vec3f {
    var p1 = opRotateX(p, e.x);
    var p2 = opRotateY(p1, e.y);
    return opRotateZ(p2, e.z);
}

// Scale - exact
fn opScale(p: vec3f, s: f32) -> vec3f {
    return p / s;
}
//let d = sdfPrimitive3d(opScale(p, s)) * s;

// Free transformation - exact
//fn opTransform(p: vec3f, transform: mat4x4<f32>) -> vec3f {
  //let q = inverse(transform) * vec4f(p, 1.);
//}
//let d = sdfPrimitive3d(opTransform(p, transform)) * determinant(transform);
//let d = sdfPrimitive3d(opScale(opRotateE(opTranslate(p, t), e, a), s)) * s;

// Symmetry - exact
fn opSymmetryX(p: vec3f) -> vec3f { return vec3f(abs(p.x), p.y, p.z); }
fn opSymmetryY(p: vec3f) -> vec3f { return vec3f(p.x, abs(p.y), p.z); }
fn opSymmetryZ(p: vec3f) -> vec3f { return vec3f(p.x, p.y, abs(p.z)); }
//let d = sdfPrimitive3d(opSymmetryX(p));

// Infinite Repetition - exact
fn opInfArray(p: vec3f, c: vec3f) -> vec3f {
    return p - c * round(p / c);
}
//let d = sdfPrimitive3d(opInfArray(p, c));

// Finite Repetition - exact
fn opLimArray(p: vec3f, c: f32, lim: vec3f) -> vec3f {
    return p - c * clamp(round(p / c), -lim, lim);
}
//let d = sdfPrimitive3d(opLimArray(p, c));

// === Primitive alterations ===

// Elongation - exact
fn opElongate(p: vec3f, h: vec3f) -> vec3f {
    return p - clamp(p, -h, h);
}
//let d = sdfPrimitive3d(opElongateFast(p, h));

fn opElongateCorrect(p: vec3f, h: vec3f) -> vec4f {
    let q = abs(p) - h;
    let sgn = 2. * step(vec3f(0.), p) - vec3f(1.);
    return vec4f(sgn * max(q, vec3f(0.)), min(max(q.x, max(q.y, q.z)), 0.));
}
//let p2 = opElongateCorrect(p, h);
//let d = p2.w + sdfPrimitive3d(p2.xyz);

// Rounding - exact
//fn opRound(p: vec3f, r: f32) -> f32 {
//  return sdfPrimitive3d(p) - r;
//}

// Onion - exact
fn opOnion(d: f32, thickness: f32) -> f32 {
    return abs(d) - thickness;
}
//let d = opOnion(sdfPrimitive3d(p), thickness);

// Extrusion from 2D SDF - exact
fn opExtrusion(d: f32, z: f32, h: f32) -> f32 {
    let w = vec2f(d, abs(z) - h);
    return min(max(w.x, w.y), 0.) + length(max(w, vec2f(0.)));
}
//let d = opExtrusion(sdfPrimitive2d(p.xy), p.z, h));

// Revolution from 2D SDF - exact
fn opRevolution(p: vec3f, o: f32) -> vec2f {
    return vec2f(length(p.xz) - o, p.y);
}
//let d = sdfPrimitive2d(opRevolution(p, h));

// Change metric - bound
fn length4(p: vec3f) -> f32 {
    var q: vec3f = p * p;
    q = q * q;
    return sqrt(sqrt(q.x + q.y + q.z));
}
fn length6(p: vec3f) -> f32 {
    var q: vec3f = p * p * p;
    q = q * q;
    return pow(q.x + q.y + q.z, 1. / 6.);
}
fn length8(p: vec3f) -> f32 {
    var q: vec3f = p * p;
    q = q * q; q = q * q;
    return pow(q.x + q.y + q.z, 1. / 8.);
}
