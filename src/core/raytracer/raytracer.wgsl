const EPSILON = 0.001f;

const PI = 3.1415927f;
const FRAC_1_PI = 0.31830987f;
const FRAC_PI_2 = 1.5707964f;

const MIN_T = 0.001f;
const MAX_T = 1000f;

const CHANNEL_R = 0u;
const CHANNEL_G = 1u;
const CHANNEL_B = 2u;

@group(0) @binding(0) var<uniform> vertexUniforms: VertexUniforms;

@vertex
fn vsMain(model: VertexInput) -> VertexOutput {
    return VertexOutput(
        vertexUniforms.viewProjectionMat * vertexUniforms.modelMat * vec4<f32>(model.position, 0.0, 1.0),
        model.texCoords
    );
}

struct VertexUniforms {
    viewProjectionMat: mat4x4<f32>,
    modelMat: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texCoords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clipPosition: vec4<f32>,
    @location(0) texCoords: vec2<f32>,
}

@group(1) @binding(0) var<uniform> frameData: vec4<u32>;
@group(1) @binding(1) var<storage, read_write> imageBuffer: array<array<f32, 3>>;

@group(2) @binding(0) var<uniform> camera: Camera;
@group(2) @binding(1) var<uniform> samplingParams: SamplingParams;
@group(2) @binding(2) var<storage, read> skyState: SkyState;

@group(3) @binding(0) var<storage, read> spheres: array<Sphere>;
@group(3) @binding(1) var<storage, read> materials: array<Material>;
@group(3) @binding(2) var<storage, read> textures: array<array<f32, 3>>;
@group(3) @binding(3) var<storage, read> lights: array<u32>;

@fragment
fn fsMain(in: VertexOutput) -> @location(0) vec4<f32> {
    let u = in.texCoords.x;
    let v = in.texCoords.y;

    let imageWidth = frameData.x;
    let imageHeight = frameData.y;
    let frameNumber = frameData.z;

    let x = u32(u * f32(imageWidth));
    let y = u32(v * f32(imageHeight));
    let idx = imageWidth * y + x;

    var rngState = initRng(vec2(x, y), vec2(imageWidth, imageHeight), frameNumber);
    var pixel = vec3(imageBuffer[idx][0u], imageBuffer[idx][1u], imageBuffer[idx][2u]);
    {
        if samplingParams.clearAccumulatedSamples == 1u {
            pixel = vec3(0f);
        }

        let rgb = samplePixel(x, y, &rngState);
        pixel += rgb;
    }
    imageBuffer[idx] = array<f32, 3>(pixel.r, pixel.g, pixel.b);

    let invN = 1f / f32(samplingParams.accumulatedSamplesPerPixel);

    return vec4(
        uncharted2(invN * pixel),
        1f
    );
}

fn uncharted2(x: vec3<f32>) -> vec3<f32> {
    // Based on uncharted2 tonemapping function
    // https://dmnsgn.github.io/glsl-tone-map/
    let exposureBias = 0.246;   // determined experimentally for the scene
    let curr = uncharted2Tonemap(exposureBias * x);

    let w = 11.2;
    let whiteScale = 1f / uncharted2Tonemap(vec3(w));
    return whiteScale * curr;
}

fn uncharted2Tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 0.15;
    let b = 0.50;
    let c = 0.10;
    let d = 0.20;
    let e = 0.02;
    let f = 0.30;
    let w = 11.2;
    return ((x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f)) - e / f;
}

fn samplePixel(x: u32, y: u32, rngState: ptr<function, u32>) -> vec3<f32> {
    let imageWidth = frameData.x;
    let imageHeight = frameData.y;
    let invWidth = 1f / f32(imageWidth);
    let invHeight = 1f / f32(imageHeight);

    let numSamples = samplingParams.numSamplesPerPixel;
    var color = vec3(0f);
    for (var i = 0u; i < numSamples; i += 1u) {
        let u = (f32(x) + rngNextFloat(rngState)) * invWidth;
        let v = (f32(y) + rngNextFloat(rngState)) * invHeight;

        let primaryRay = cameraMakeRay(camera, rngState, u, 1f - v);
        color += rayColor(primaryRay, rngState);
    }

    return color;
}

fn rayColor(primaryRay: Ray, rngState: ptr<function, u32>) -> vec3<f32> {
    var ray = primaryRay;

    var color = vec3(0f);
    var throughput = vec3(1f);

    for (var bounce = 0u; bounce < samplingParams.numBounces; bounce += 1u) {
        var intersection = Intersection();

        if intersection(ray, &intersection) {
            let material = materials[intersection.materialIdx];

            if material.id == 4u {
                let emissionTexture = material.desc1;
                let emissionColor = textureLookup(emissionTexture, intersection.u, intersection.v);
                color += throughput * emissionColor;
                break;
            }

            var scatter = scatterRay(ray, intersection, material, rngState);
            ray = scatter.ray;
            throughput *= scatter.throughput;
        } else {
            // The ray missed. Output background color.
            let v = normalize(ray.direction);
            let s = skyState.sunDirection;

            let theta = acos(v.y);
            let gamma = acos(clamp(dot(v, s), -1f, 1f));

            color += throughput * vec3(
                radiance(theta, gamma, CHANNEL_R),
                radiance(theta, gamma, CHANNEL_G),
                radiance(theta, gamma, CHANNEL_B)
            );

            break;
        }
    }

    return color;
}

fn intersection(ray: Ray, intersection: ptr<function, Intersection>) -> bool {
    var closestT = MAX_T;
    var closestIntersection = Intersection();

    for (var idx = 0u; idx < arrayLength(&spheres); idx = idx + 1u) {
        var testIntersect = Intersection();
        if rayIntersectSphere(ray, idx, MIN_T, closestT, &testIntersect) {
            closestT = testIntersect.t;
            closestIntersection = testIntersect;
        }
    }

    if closestT < MAX_T {
        *intersection = closestIntersection;
        return true;
    }

    return false;
}

fn scatterRay(wo: Ray, hit: Intersection, material: Material, rngState: ptr<function, u32>) -> Scatter {
    switch material.id {
        case 0u: {
            let texture = material.desc1;
            return scatterMixtureDensity(hit, texture, rngState);
        }

        case 1u: {
            let texture = material.desc1;
            let fuzz = material.x;
            return scatterMetal(wo, hit, texture, fuzz, rngState);
        }

        case 2u: {
            let refractionIndex = material.x;
            return scatterDielectric(wo, hit, refractionIndex, rngState);
        }

        case 3u: {
            let texture1 = material.desc1;
            let texture2 = material.desc2;
            return scatterCheckerboard(hit, texture1, texture2, rngState);
        }

        default: {
            return scatterMissingMaterial(hit, rngState);
        }
    }
}

fn scatterMixtureDensity(hit: Intersection, albedo: TextureDescriptor, rngState: ptr<function, u32>) -> Scatter {
    let scatterDirection = sampleMixtureDensity(hit, rngState);
    let materialValue = evalLambertian(hit, albedo, scatterDirection);
    let materialPdf = pdfLambertian(hit, scatterDirection);
    let lightPdf = pdfLight(hit, scatterDirection);
    let throughput = materialValue / max(EPSILON, (0.5f * materialPdf + 0.5f * lightPdf));
    return Scatter(Ray(hit.p, scatterDirection), throughput);
}

fn sampleMixtureDensity(hit: Intersection, rngState: ptr<function, u32>) -> vec3<f32> {
    if rngNextFloat(rngState) < 0.5f {
        return sampleLambertian(hit, rngState);
    } else {
        return sampleLight(hit, rngState);
    }
}

fn evalLambertian(hit: Intersection, texture: TextureDescriptor, wi: vec3<f32>) -> vec3<f32> {
    return textureLookup(texture, hit.u, hit.v) * FRAC_1_PI * max(EPSILON, dot(hit.n, wi));
}

fn sampleLambertian(hit: Intersection, rngState: ptr<function, u32>) -> vec3<f32> {
    let v = rngNextInCosineWeightedHemisphere(rngState);
    let onb = pixarOnb(hit.n);
    return onb * v;
}

fn pdfLambertian(hit: Intersection, wi: vec3<f32>) -> f32 {
    return max(EPSILON, dot(hit.n, wi) * FRAC_1_PI);
}

fn sampleLight(hit: Intersection, rngState: ptr<function, u32>) -> vec3<f32> {
    // Select a random light using a uniform distribution.
    let numLights = arrayLength(&lights);   // TODO: what about when there are no lights?
    let lightIdx = rngNextUintInRange(rngState, 0u, numLights - 1u);
    let sphereIdx = lights[lightIdx];
    let sphere = spheres[sphereIdx];

    return sampleHemisphere(hit, sphere, rngState);
}

fn sampleHemisphere(hit: Intersection, sphere: Sphere, rngState: ptr<function, u32>) -> vec3<f32> {
    let v = rngNextInUnitHemisphere(rngState);

    // Sample the hemisphere facing the intersection point.
    let dir = normalize(hit.p - sphere.centerAndPad.xyz);
    let onb = pixarOnb(dir);

    let pointOnSphere = sphere.centerAndPad.xyz + onb * sphere.radius * v;
    let toPointOnSphere = pointOnSphere - hit.p;

    return normalize(toPointOnSphere);
}

fn pdfLight(hit: Intersection, wi: vec3<f32>) -> f32 {
    let ray = Ray(hit.p, wi);
    var lightHit = Intersection();
    var pdf = 0f;

    if intersection(ray, &lightHit) {
        let sphereIdx = lightHit.sphereIdx;
        let sphere = spheres[sphereIdx];
        let numSpheres = arrayLength(&spheres);
        let toLight = lightHit.p - hit.p;
        let lengthSqr = dot(toLight, toLight);
        let cosine = abs(dot(wi, lightHit.n));
        let areaHalfSphere = 2f * PI * sphere.radius * sphere.radius;

        // lengthSqr / cosine is the inverse of the geometric factor, as defined in
        // "MULTIPLE IMPORTANCE SAMPLING 101".
        pdf = lengthSqr / max(EPSILON, cosine * areaHalfSphere * f32(numSpheres));
    }

    return pdf;
}

fn pixarOnb(n: vec3<f32>) -> mat3x3<f32> {
    // https://www.jcgt.org/published/0006/01/01/paper-lowres.pdf
    let s = select(-1f, 1f, n.z >= 0f);
    let a = -1f / (s + n.z);
    let b = n.x * n.y * a;
    let u = vec3<f32>(1f + s * n.x * n.x * a, s * b, -s * n.x);
    let v = vec3<f32>(b, s + n.y * n.y * a, -n.y);

    return mat3x3<f32>(u, v, n);
}

fn scatterMetal(wo: Ray, hit: Intersection, texture: TextureDescriptor, fuzz: f32, rngState: ptr<function, u32>) -> Scatter {
    let scatterDirection = reflect(wo.direction, hit.n) + fuzz * rngNextVec3InUnitSphere(rngState);
    let albedo = textureLookup(texture, hit.u, hit.v);
    return Scatter(Ray(hit.p, scatterDirection), albedo);
}

fn scatterDielectric(rayIn: Ray, hit: Intersection, refractionIndex: f32, rngState: ptr<function, u32>) -> Scatter {
    let wo = rayIn.direction;
    var outwardNormal = vec3(0f);
    var niOverNt = 0f;
    var cosine = 0f;
    if dot(wo, hit.n) > 0f {
        outwardNormal = -hit.n;
        niOverNt = refractionIndex;
        cosine = refractionIndex * dot(normalize(wo), hit.n);
    } else {
        outwardNormal = hit.n;
        niOverNt = 1f / refractionIndex;
        cosine = dot(normalize(-wo), hit.n);
    };

    var refractedDirection = vec3(0f);
    if refract(wo, outwardNormal, niOverNt, &refractedDirection) {
        let reflectionProb = schlick(cosine, refractionIndex);
        var wi = refractedDirection;
        if rngNextFloat(rngState) < reflectionProb {
            reflect(wo, hit.n);
        }

        return Scatter(Ray(hit.p, wi), vec3(1f));
    }

    let wi = reflect(wo, hit.n);
    return Scatter(Ray(hit.p, wi), vec3(1f));
}

fn refract(v: vec3<f32>, n: vec3<f32>, niOverNt: f32, refractDirection: ptr<function, vec3<f32>>) -> bool {
    // ni * sin(i) = nt * sin(t)
    // sin(t) = sin(i) * (ni / nt)
    let uv = normalize(v);
    let dt = dot(uv, n);
    let discriminant = 1f - niOverNt * niOverNt * (1f - dt * dt);
    if discriminant > 0f {
        *refractDirection = normalize(niOverNt * (uv - dt * n) - sqrt(discriminant) * n);
        return true;
    }

    return false;
}

fn schlick(cosine: f32, refractionIndex: f32) -> f32 {
    var r0 = (1f - refractionIndex) / (1f + refractionIndex);
    r0 = r0 * r0;
    return r0 + pow((1f - r0) * (1f - cosine), 5f);
}

fn scatterCheckerboard(hit: Intersection, texture1: TextureDescriptor, texture2: TextureDescriptor, rngState: ptr<function, u32>) -> Scatter {
    let sines = sin(5f * hit.p.x) * sin(5f * hit.p.y) * sin(5f * hit.p.z);
    if sines < 0f {
        return scatterMixtureDensity(hit, texture1, rngState);
    } else {
        return scatterMixtureDensity(hit, texture2, rngState);
    }
}

fn scatterMissingMaterial(hit: Intersection, rngState: ptr<function, u32>) -> Scatter {
    let scatterDirection = hit.n + rngNextVec3InUnitSphere(rngState);
    // An aggressive pink color to indicate an error
    let albedo = vec3(0.9921f, 0.24705f, 0.57254f);
    return Scatter(Ray(hit.p, scatterDirection), albedo);
}

fn radiance(theta: f32, gamma: f32, channel: u32) -> f32 {
    let r = skyState.radiances[channel];
    let idx = 9u * channel;
    let p0 = skyState.params[idx + 0u];
    let p1 = skyState.params[idx + 1u];
    let p2 = skyState.params[idx + 2u];
    let p3 = skyState.params[idx + 3u];
    let p4 = skyState.params[idx + 4u];
    let p5 = skyState.params[idx + 5u];
    let p6 = skyState.params[idx + 6u];
    let p7 = skyState.params[idx + 7u];
    let p8 = skyState.params[idx + 8u];

    let cosGamma = cos(gamma);
    let cosGamma2 = cosGamma * cosGamma;
    let cosTheta = abs(cos(theta));

    let expM = exp(p4 * gamma);
    let rayM = cosGamma2;
    let mieMLhs = 1.0 + cosGamma2;
    let mieMRhs = pow(1.0 + p8 * p8 - 2.0 * p8 * cosGamma, 1.5f);
    let mieM = mieMLhs / mieMRhs;
    let zenith = sqrt(cosTheta);
    let radianceLhs = 1.0 + p0 * exp(p1 / (cosTheta + 0.01));
    let radianceRhs = p2 + p3 * expM + p5 * rayM + p6 * mieM + p7 * zenith;
    let radianceDist = radianceLhs * radianceRhs;
    return r * radianceDist;
}

struct SkyState {
    params: array<f32, 27>,
    radiances: array<f32, 3>,
    sunDirection: vec3<f32>,
};

struct SamplingParams {
    numSamplesPerPixel: u32,
    numBounces: u32,
    accumulatedSamplesPerPixel: u32,
    clearAccumulatedSamples: u32,
}

struct Sphere {
    centerAndPad: vec4<f32>,
    radius: f32,
    materialIdx: u32,
}

struct Material {
    id: u32,
    desc1: TextureDescriptor,
    desc2: TextureDescriptor,
    x: f32,
}

struct TextureDescriptor {
    width: u32,
    height: u32,
    offset: u32,
}

fn textureLookup(desc: TextureDescriptor, u: f32, v: f32) -> vec3<f32> {
    let u = clamp(u, 0f, 1f);
    let v = 1f - clamp(v, 0f, 1f);

    let j = u32(u * f32(desc.width));
    let i = u32(v * f32(desc.height));
    let idx = i * desc.width + j;

    let elem = textures[desc.offset + idx];
    return vec3(elem[0u], elem[1u], elem[2u]);
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

struct Scatter {
    ray: Ray,
    throughput: vec3<f32>,
}

struct Intersection {
    p: vec3<f32>,
    n: vec3<f32>,
    u: f32,
    v: f32,
    t: f32,
    materialIdx: u32,
    sphereIdx: u32,
}

fn rayIntersectSphere(ray: Ray, sphereIdx: u32, tmin: f32, tmax: f32, hit: ptr<function, Intersection>) -> bool {
    let sphere = spheres[sphereIdx];
    let oc = ray.origin - sphere.centerAndPad.xyz;
    let a = dot(ray.direction, ray.direction);
    let b = dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - a * c;

    if discriminant > 0f {
        var t = (-b - sqrt(b * b - a * c)) / a;
        if t < tmax && t > tmin {
            *hit = sphereIntersection(ray, sphere, sphereIdx, t);
            return true;
        }

        t = (-b + sqrt(b * b - a * c)) / a;
        if t < tmax && t > tmin {
            *hit = sphereIntersection(ray, sphere, sphereIdx, t);
            return true;
        }
    }

    return false;
}

fn sphereIntersection(ray: Ray, sphere: Sphere, sphereIdx: u32, t: f32) -> Intersection {
    let p = rayPointAtParameter(ray, t);
    let n = (1f / sphere.radius) * (p - sphere.centerAndPad.xyz);
    let theta = acos(-n.y);
    let phi = atan2(-n.z, n.x) + PI;
    let u = 0.5 * FRAC_1_PI * phi;
    let v = FRAC_1_PI * theta;

    // TODO: passing sphereIdx in here just to pass it to Intersection
    return Intersection(p, n, u, v, t, sphere.materialIdx, sphereIdx);
}

fn rayPointAtParameter(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + t * ray.direction;
}

struct Camera {
    eye: vec3<f32>,
    horizontal: vec3<f32>,
    vertical: vec3<f32>,
    u: vec3<f32>,
    v: vec3<f32>,
    lensRadius: f32,
    lowerLeftCorner: vec3<f32>,
}

fn cameraMakeRay(camera: Camera, rngState: ptr<function, u32>, u: f32, v: f32) -> Ray {
    let randomPointInLens = camera.lensRadius * rngNextVec3InUnitDisk(rngState);
    let lensOffset = randomPointInLens.x * camera.u + randomPointInLens.y * camera.v;

    let origin = camera.eye + lensOffset;
    let direction = camera.lowerLeftCorner + u * camera.horizontal + v * camera.vertical - origin;

    return Ray(origin, direction);
}

fn rngNextInCosineWeightedHemisphere(state: ptr<function, u32>) -> vec3<f32> {
    let r1 = rngNextFloat(state);
    let r2 = rngNextFloat(state);
    let sqrt_r2 = sqrt(r2);

    let z = sqrt(1f - r2);
    let phi = 2f * PI * r1;
    let x = cos(phi) * sqrt_r2;
    let y = sin(phi) * sqrt_r2;

    return vec3<f32>(x, y, z);
}

fn rngNextInUnitHemisphere(state: ptr<function, u32>) -> vec3<f32> {
    let r1 = rngNextFloat(state);
    let r2 = rngNextFloat(state);

    let phi = 2f * PI * r1;
    let sinTheta = sqrt(1f - r2 * r2);

    let x = cos(phi) * sinTheta;
    let y = sin(phi) * sinTheta;
    let z = r2;

    return vec3(x, y, z);
}

fn rngNextVec3InUnitDisk(state: ptr<function, u32>) -> vec3<f32> {
    // Generate numbers uniformly in a disk:
    // https://stats.stackexchange.com/a/481559

    // r^2 is distributed as U(0, 1).
    let r = sqrt(rngNextFloat(state));
    let alpha = 2f * PI * rngNextFloat(state);

    let x = r * cos(alpha);
    let y = r * sin(alpha);

    return vec3(x, y, 0f);
}

fn rngNextVec3InUnitSphere(state: ptr<function, u32>) -> vec3<f32> {
    let r = pow(rngNextFloat(state), 0.33333f);
    let cosTheta = 1f - 2f * rngNextFloat(state);
    let sinTheta = sqrt(1f - cosTheta * cosTheta);
    let phi = 2f * PI * rngNextFloat(state);

    let x = r * sinTheta * cos(phi);
    let y = r * sinTheta * sin(phi);
    let z = cosTheta;

    return vec3(x, y, z);
}

fn rngNextUintInRange(state: ptr<function, u32>, min: u32, max: u32) -> u32 {
    let x = rngNextInt(state);
    return min + (x) % (max - min);
}

fn rngNextFloat(state: ptr<function, u32>) -> f32 {
    let x = rngNextInt(state);
    return f32(x) / f32(0xffffffffu);
}

fn initRng(pixel: vec2<u32>, resolution: vec2<u32>, frame: u32) -> u32 {
    // Adapted from https://github.com/boksajak/referencePT
    let seed = dot(pixel, vec2<u32>(1u, resolution.x)) ^ jenkinsHash(frame);
    return jenkinsHash(seed);
}

fn rngNextInt(state: ptr<function, u32>) -> u32 {
    // PCG random number generator
    // Based on https://www.shadertoy.com/view/XlGcRh
    let newState = *state * 747796405u + 2891336453u;
    *state = newState;
    let word = ((newState >> ((newState >> 28u) + 4u)) ^ newState) * 277803737u;
    return (word >> 22u) ^ word;
}

fn jenkinsHash(input: u32) -> u32 {
    var x = input;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
}
