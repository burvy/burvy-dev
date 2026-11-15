/* Realistic 3D view from a spot, built from what Vexcel gives us at the place:
 *  - buildings and trees: the height map (/api/scene rings: 1 m cells to 256 m, 4 m to 1 km) as a flat-shaded surface;
 *  - roofs and ground: Vexcel aerial photos draped on top (/api/tiles/vexcel), Esri World Imagery where Vexcel has none;
 *  - walls: facades with windows (trees get foliage instead), lit windows after sunset;
 *  - sky: physically based (three.js Sky, Preetham model) with the sun at the chosen time, haze, long low-sun shadows.
 * Usage: const v = await RealView.create({host, base, lat, lon, eyeM, yaw}); v.setSun(az, el); v.mount(host); v.dispose().
 */
const THREE = window.THREE;
const EARTH = 40075016.686; // metres around the equator
const SKY_JS = 'https://cdn.jsdelivr.net/npm/three@0.128.0/examples/js/objects/Sky.js';
const rad = Math.PI / 180;

const mx = lon => (lon + 180) / 360;
const my = lat => (1 - Math.log(Math.tan(Math.PI / 4 + lat * rad / 2)) / Math.PI) / 2;
const clamp = (x, a, b) => Math.max(a, Math.min(b, x));
const lerp = (a, b, t) => a + (b - a) * t;

function loadScript(src) {
  return new Promise((ok, bad) => { const s = document.createElement('script'); s.src = src; s.onload = ok; s.onerror = bad; document.head.appendChild(s); });
}

/* ---------- facade texture: one tile = 3 bays x 4 floors (12 m x 14 m); white wall so vertex colour tints it ---------- */
const FACADE_W = 12, FACADE_H = 14;
function facadeTextures() {
  const S = 256, c = document.createElement('canvas'), e = document.createElement('canvas');
  c.width = c.height = e.width = e.height = S;
  const g = c.getContext('2d'), ge = e.getContext('2d');
  g.fillStyle = '#fff'; g.fillRect(0, 0, S, S);
  ge.fillStyle = '#000'; ge.fillRect(0, 0, S, S);
  const px = m => m / FACADE_W * S, py = m => m / FACADE_H * S;
  let seed = 7; const rnd = () => (seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296;
  for (let f = 0; f < 4; f++) for (let b = 0; b < 3; b++) {
    // window 0.9..2.6 m up in each 3.5 m floor, 1.8 m wide in each 4 m bay (canvas y runs down = texture v up)
    const x = px(b * 4 + 1.1), w = px(1.8), y = S - py(f * 3.5 + 2.6), h = py(1.7);
    const k = 40 + rnd() * 30;
    g.fillStyle = `rgb(${k * .55},${k * .7},${k * .85})`; g.fillRect(x, y, w, h);
    g.fillStyle = 'rgba(255,255,255,.18)'; g.fillRect(x, y, w, h * .3); // sky reflected in the top of the glass
    g.fillStyle = '#d9d4cc'; g.fillRect(x - 2, y + h, w + 4, 3);       // sill
    if (rnd() < .35) { ge.fillStyle = `rgb(255,${190 + rnd() * 40 | 0},${110 + rnd() * 50 | 0})`; ge.fillRect(x, y, w, h); }
  }
  const t = new THREE.CanvasTexture(c), te = new THREE.CanvasTexture(e);
  for (const x of [t, te]) { x.wrapS = x.wrapT = THREE.RepeatWrapping; x.anisotropy = 4; }
  t.encoding = te.encoding = THREE.sRGBEncoding;
  return { map: t, emissiveMap: te };
}


/* ---------- aerial photo for one ring, composed from map tiles into a canvas ---------- */
function orthoTexture(base, lat, lon, L, onUpdate) {
  const k = EARTH * Math.cos(lat * rad); // metres per unit of mercator x at this latitude
  const x0 = mx(lon) - L / 2 / k, x1 = mx(lon) + L / 2 / k, y0 = my(lat) - L / 2 / k, y1 = my(lat) + L / 2 / k;
  // the deepest zoom that keeps it to ~6 tiles across (sharp but few requests), at most z20
  const z = clamp(Math.floor(Math.log2(6 / (x1 - x0))), 1, 20), N = 2 ** z;
  const W = clamp(Math.round((x1 - x0) * N * 256), 256, 2048);
  const c = document.createElement('canvas'); c.width = c.height = W;
  const g = c.getContext('2d'); g.fillStyle = '#6f7466'; g.fillRect(0, 0, W, W);
  const tex = new THREE.CanvasTexture(c);
  tex.encoding = THREE.sRGBEncoding; tex.anisotropy = 8;
  let pending = 0;
  for (let ty = Math.floor(y0 * N); ty <= Math.floor(y1 * N); ty++) for (let tx = Math.floor(x0 * N); tx <= Math.floor(x1 * N); tx++) {
    const im = new Image(); im.crossOrigin = 'anonymous'; pending++;
    im.onload = () => {
      g.drawImage(im, (tx / N - x0) / (x1 - x0) * W, (ty / N - y0) / (y1 - y0) * W, W / ((x1 - x0) * N) + 1, W / ((x1 - x0) * N) + 1);
      tex.needsUpdate = true; onUpdate(--pending);
    };
    im.onerror = () => {
      if (im.esri) return onUpdate(--pending);
      im.esri = 1; im.src = `https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/${z}/${ty}/${tx}`;
    };
    im.src = `${base}/api/tiles/vexcel/${z}/${tx}/${ty}`;
  }
  return { tex, canvas: c, g };
}

/* ---------- one ring of the height map as a continuous 1 m surface (vertex per cell centre) ---------- */
/* Trees vs buildings from shape alone: a canopy is bumpy everywhere (many cells curve by 0.5-6 m), a roof is flat or
   planar and only its edges curve. Fraction of bumpy cells in a 5x5 window, per cell. */
function vegetation(H, n, cell) {
  const lo = 0.5 * Math.sqrt(cell), hi = 6 * cell; // coarser cells curve more
  const bumpy = new Uint8Array(n * n), out = new Float32Array(n * n);
  for (let r = 1; r < n - 1; r++) for (let c = 1; c < n - 1; c++) {
    const i = r * n + c, v = H[i];
    const k = Math.abs(H[i - 1] + H[i + 1] - 2 * v) + Math.abs(H[i - n] + H[i + n] - 2 * v);
    bumpy[i] = k > lo && k < hi ? 1 : 0;
  }
  for (let r = 2; r < n - 2; r++) for (let c = 2; c < n - 2; c++) {
    let s = 0;
    for (let dr = -2; dr <= 2; dr++) for (let dc = -2; dc <= 2; dc++) s += bumpy[(r + dr) * n + c + dc];
    out[r * n + c] = s / 25;
  }
  return out;
}

/* 3x3 median: removes DSM speckle (cars, noise) but keeps building edges straight. */
function median3(H, n) {
  const out = new Float32Array(H), w = new Float32Array(9);
  for (let r = 1; r < n - 1; r++) for (let c = 1; c < n - 1; c++) {
    let k = 0, bad = false;
    for (let dr = -1; dr <= 1; dr++) for (let dc = -1; dc <= 1; dc++) { const v = H[(r + dr) * n + c + dc]; if (Number.isNaN(v)) bad = true; w[k++] = v; }
    if (!bad) { w.sort(); out[r * n + c] = w[4]; }
  }
  return out;
}

function ringGeometry(H, n, cell, cut, ground, drop, veg) {
  // H: heights (m above base, NaN = no data), row 0 north. cut: half-size (m) of the inner ring to leave out.
  const half = n / 2, L = n * cell;
  const pos = new Float32Array(n * n * 3), uv = new Float32Array(n * n * 2), gnd = new Float32Array(n * n);
  for (let r = 0; r < n; r++) for (let c = 0; c < n; c++) {
    const i = r * n + c, x = (c + .5 - half) * cell, z = (r + .5 - half) * cell;
    let y = H[i];
    if (Number.isNaN(y)) y = 0;
    pos.set([x, y - drop, z], i * 3);
    uv.set([x / L + .5, .5 - z / L], i * 2);
    gnd[i] = ground(r, c);
  }
  const idx = [];
  for (let r = 0; r < n - 1; r++) for (let c = 0; c < n - 1; c++) {
    const a = r * n + c, b = a + 1, d = a + n, e = d + 1;
    if (Number.isNaN(H[a]) || Number.isNaN(H[b]) || Number.isNaN(H[d]) || Number.isNaN(H[e])) continue;
    if (cut) { const x = (c + 1 - half) * cell, z = (r + 1 - half) * cell; if (Math.abs(x) < cut - cell && Math.abs(z) < cut - cell) continue; }
    // split each square along the diagonal that keeps building edges straight (the shorter height jump)
    if (Math.abs(H[a] - H[e]) < Math.abs(H[b] - H[d])) idx.push(a, d, e, a, e, b); else idx.push(a, d, b, b, d, e);
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute('position', new THREE.BufferAttribute(pos, 3));
  g.setAttribute('uv', new THREE.BufferAttribute(uv, 2));
  g.setAttribute('ground', new THREE.BufferAttribute(gnd, 1));
  g.setAttribute('veg', new THREE.BufferAttribute(veg, 1));
  g.setIndex(idx);
  return g;
}

/* The photo on top; on steep faces (walls) a facade with windows tinted by the photo, except where the photo is green
   (tree canopy), which stays foliage. Windows glow after sunset. */
function surfaceMaterial(photoTex, facade, shared) {
  const m = new THREE.MeshStandardMaterial({ map: photoTex, roughness: .92, metalness: 0, flatShading: true });
  m.onBeforeCompile = sh => {
    sh.uniforms.facadeMap = { value: facade.map };
    sh.uniforms.facadeGlow = { value: facade.emissiveMap };
    sh.uniforms.night = shared.night;
    sh.vertexShader = sh.vertexShader
      .replace('#include <common>', '#include <common>\nattribute float ground;\nattribute float veg;\nvarying vec3 vW;\nvarying float vGround;\nvarying float vVeg;')
      .replace('#include <begin_vertex>', '#include <begin_vertex>\nvW = (modelMatrix * vec4(transformed, 1.0)).xyz;\nvGround = ground;\nvVeg = veg;');
    sh.fragmentShader = sh.fragmentShader
      .replace('#include <common>', '#include <common>\nuniform sampler2D facadeMap;\nuniform sampler2D facadeGlow;\nuniform float night;\nvarying vec3 vW;\nvarying float vGround;\nvarying float vVeg;\nfloat gSteep; float gWin; vec2 gFuv;\nfloat gSteepPre(vec3 n) { return 1.0 - smoothstep(0.35, 0.7, abs(n.y)); }')
      .replace('#include <normal_fragment_maps>', `#include <normal_fragment_maps>
        vec3 wN = inverseTransformDirection(normal, viewMatrix);
        vec3 pc = diffuseColor.rgb;
        float green = max(smoothstep(0.3, 0.55, vVeg), smoothstep(0.0, 0.05, pc.g - max(pc.r, pc.b) * 0.97) * (1.0 - gSteepPre(wN)));
        gSteep = 1.0 - smoothstep(0.35, 0.7, abs(wN.y));
        float along = abs(wN.x) > abs(wN.z) ? vW.z : vW.x;
        gFuv = vec2(along / 12.0, (vW.y - vGround) / 14.0);
        vec3 fac = texture2D(facadeMap, gFuv).rgb;
        vec3 tint = mix(pc, vec3(0.72, 0.68, 0.62), 0.55);
        gWin = gSteep * (1.0 - green) * step(1.0, vW.y - vGround);
        diffuseColor.rgb = mix(pc, fac * tint * 1.15, gWin);
        float leaf = 0.75 + 0.25 * sin(vW.x * 1.7 + vW.y * 2.3) * sin(vW.z * 1.9 - vW.y * 1.3);
        diffuseColor.rgb = mix(diffuseColor.rgb, mix(vec3(0.13, 0.19, 0.08), pc, 0.25) * leaf, gSteep * green);`)
      .replace('#include <emissivemap_fragment>', '#include <emissivemap_fragment>\ntotalEmissiveRadiance += texture2D(facadeGlow, gFuv).rgb * gWin * night * 0.9;');
  };
  return m;
}


/* ---------- light and air by sun elevation ---------- */
function sunColor(el) { // white high up, orange then red near the horizon
  const t = clamp((el + 1) / 14, 0, 1);
  return new THREE.Color(lerp(1, 1, t), lerp(.45, .93, t), lerp(.2, .84, t));
}
function hazeColor(el) {
  const day = new THREE.Color('#b9c6d4'), dusk = new THREE.Color('#d99a72'), night = new THREE.Color('#1c2233');
  return el > 8 ? day : el > -1 ? dusk.clone().lerp(day, (el + 1) / 9) : night.clone().lerp(dusk, clamp((el + 7) / 6, 0, 1));
}

export async function create({ host, base = '', lat, lon, eyeM = null, yaw = 270, desktop = false, onStatus = () => {} }) {
  if (!THREE) throw new Error('three.js not loaded');
  onStatus('Loading the height map…');
  const metaRes = await fetch(`${base}/api/scene?lat=${lat}&lon=${lon}&radius_m=1000&detail=desktop`);
  if (!metaRes.ok) throw new Error('scene ' + metaRes.status);
  const meta = await metaRes.json();
  const binRes = await fetch(base + meta.data_url);
  if (!binRes.ok) throw new Error('scene bin ' + binRes.status);
  const dv = new DataView(await binRes.arrayBuffer());
  if (!THREE.Sky) await loadScript(SKY_JS).catch(() => {});
  onStatus('Building the city…');

  const renderer = new THREE.WebGLRenderer({ antialias: true, powerPreference: 'high-performance' });
  renderer.setPixelRatio(Math.min(devicePixelRatio || 1, 2));
  renderer.outputEncoding = THREE.sRGBEncoding;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  const scene = new THREE.Scene();
  const cam = new THREE.PerspectiveCamera(60, 1, 0.5, 60000);
  const disposables = [];
  let raf = 0; const render = () => { if (!raf) raf = requestAnimationFrame(() => { raf = 0; renderer.render(scene, cam); }); };

  const facade = facadeTextures(); disposables.push(facade.map, facade.emissiveMap);
  const shared = { night: { value: 0 } };

  // rings, innermost first; each outer ring leaves out the area the inner one already draws
  let cut = 0, centre = 0;
  meta.rings.forEach((ring, i) => {
    const n = ring.width, H = new Float32Array(n * n);
    for (let j = 0; j < n * n; j++) { const v = dv.getUint16(ring.byte_offset + j * 2, true); H[j] = v === meta.nodata ? NaN : v / 10; }
    if (i === 0) { // what the eye stands on: highest point within ~2 m of the spot
      for (let r = n / 2 - 2; r < n / 2 + 2; r++) for (let c = n / 2 - 2; c < n / 2 + 2; c++) if (!Number.isNaN(H[r * n + c])) centre = Math.max(centre, H[r * n + c]);
    }
    // street level under a cell: the lowest height within ~20 m, so facades start their floors at the street
    const w = Math.max(1, Math.round(20 / ring.cell_m)), st = Math.max(1, w >> 2);
    const ground = (r, c) => {
      let m = Infinity;
      for (let dr = -w; dr <= w; dr += st) for (let dc = -w; dc <= w; dc += st) {
        const rr = r + dr, cc = c + dc; if (rr < 0 || cc < 0 || rr >= n || cc >= n) continue;
        const v = H[rr * n + cc]; if (!Number.isNaN(v) && v < m) m = v;
      }
      return Number.isFinite(m) ? m : 0;
    };
    const Hs = ring.cell_m <= 1 ? median3(H, n) : H;
    const geo = ringGeometry(Hs, n, ring.cell_m, cut, ground, i === 0 ? 0 : 0.15 * i, vegetation(Hs, n, ring.cell_m));
    const photo = orthoTexture(base, lat, lon, n * ring.cell_m, () => render());
    const mat = surfaceMaterial(photo.tex, facade, shared);
    const mesh = new THREE.Mesh(geo, mat);
    mesh.receiveShadow = mesh.castShadow = ring.cell_m <= 4;
    scene.add(mesh);
    disposables.push(geo, photo.tex, mat);
    cut = n * ring.cell_m / 2;
  });

  // ground beyond the data, so the horizon is never empty
  const far = new THREE.Mesh(new THREE.PlaneGeometry(60000, 60000), new THREE.MeshStandardMaterial({ color: '#4b5040', roughness: 1 }));
  far.rotation.x = -Math.PI / 2; far.position.y = -0.5; scene.add(far); disposables.push(far.geometry, far.material);

  // sky
  let sky = null;
  if (THREE.Sky) {
    sky = new THREE.Sky(); sky.scale.setScalar(50000); scene.add(sky);
    const u = sky.material.uniforms;
    u.turbidity.value = 3.5; u.rayleigh.value = 2.2; u.mieCoefficient.value = .004; u.mieDirectionalG.value = .82;
  }
  scene.fog = new THREE.FogExp2('#b9c6d4', .00042);
  const sun = new THREE.DirectionalLight('#ffffff', 2.5);
  sun.castShadow = true;
  const sm = desktop ? 4096 : 2048;
  sun.shadow.mapSize.set(sm, sm);
  Object.assign(sun.shadow.camera, { left: -450, right: 450, top: 450, bottom: -450, near: 1, far: 4000 });
  sun.shadow.bias = -0.0004; sun.shadow.normalBias = 0.6;
  const hemi = new THREE.HemisphereLight('#a9c4e8', '#4a4535', .6);
  scene.add(sun, sun.target, hemi);

  // camera on the spot
  const eye = (eyeM != null ? Math.max(eyeM - meta.base_m, centre + 1.7) : centre + 1.7);
  cam.position.set(0, eye, 0);
  sun.target.position.set(0, 0, 0);
  let look = { yaw, pitch: 3, fov: 60 };
  const aim = () => {
    const a = look.yaw * rad, p = look.pitch * rad;
    cam.fov = look.fov; cam.updateProjectionMatrix();
    cam.lookAt(cam.position.x + Math.sin(a) * Math.cos(p), cam.position.y + Math.sin(p), cam.position.z - Math.cos(a) * Math.cos(p));
    render();
  };

  // look around: drag (yaw + pitch), pinch or wheel (zoom like a lens)
  const el = renderer.domElement;
  el.style.touchAction = 'none'; el.style.cursor = 'grab'; el.style.display = 'block';
  const pts = new Map(); let pinch = 0;
  el.onpointerdown = e => { pts.set(e.pointerId, [e.clientX, e.clientY]); el.setPointerCapture(e.pointerId); el.style.cursor = 'grabbing'; };
  el.onpointerup = el.onpointercancel = e => { pts.delete(e.pointerId); pinch = 0; el.style.cursor = 'grab'; };
  el.onpointermove = e => {
    const prev = pts.get(e.pointerId); if (!prev) return;
    pts.set(e.pointerId, [e.clientX, e.clientY]);
    if (pts.size === 2) {
      const [a, b] = [...pts.values()], d = Math.hypot(a[0] - b[0], a[1] - b[1]);
      if (pinch) look.fov = clamp(look.fov * pinch / d, 15, 80);
      pinch = d;
    } else {
      const k = look.fov / 60 * .22;
      look.yaw -= (e.clientX - prev[0]) * k; look.pitch = clamp(look.pitch + (e.clientY - prev[1]) * k, -35, 45);
    }
    aim();
  };
  el.onwheel = e => { e.preventDefault(); look.fov = clamp(look.fov * (e.deltaY > 0 ? 1.08 : 1 / 1.08), 15, 80); aim(); };

  let hostEl = null;
  const fit = () => {
    if (!hostEl) return;
    const w = hostEl.clientWidth || 600, h = hostEl.clientHeight || 320;
    renderer.setSize(w, h, false); el.style.width = '100%'; el.style.height = '100%';
    cam.aspect = w / Math.max(h, 1); cam.updateProjectionMatrix(); render();
  };
  const ro = new ResizeObserver(fit);

  const view = {
    attribution: meta.attribution + ' · Imagery © Vexcel Imaging, Esri',
    canvas: el,
    get yaw() { return look.yaw; },
    mount(h) {
      if (hostEl) ro.unobserve(hostEl);
      hostEl = h; h.innerHTML = ''; h.style.position = 'relative'; h.appendChild(el);
      const note = document.createElement('div');
      note.textContent = view.attribution;
      note.style.cssText = 'position:absolute;left:8px;bottom:6px;font:11px sans-serif;color:rgba(255,255,255,.85);text-shadow:0 1px 3px #000;pointer-events:none';
      h.appendChild(note); ro.observe(h); fit();
    },
    setYaw(y) { look.yaw = y; aim(); },
    setLook(o) { Object.assign(look, o); aim(); },
    _cam: cam, // for tests and debugging
    setSun(az, elev) {
      const dir = new THREE.Vector3(Math.sin(az * rad) * Math.cos(elev * rad), Math.sin(elev * rad), -Math.cos(az * rad) * Math.cos(elev * rad));
      if (sky) { const u = sky.material.uniforms, low = clamp(1 - elev / 12, 0, 1); u.sunPosition.value.copy(dir); u.turbidity.value = lerp(3.5, 6, low); u.mieCoefficient.value = lerp(.004, .006, low); u.rayleigh.value = lerp(2.2, 3, low); }
      const day = clamp((elev + 2) / 10, 0, 1);
      sun.color.copy(sunColor(elev));
      sun.intensity = 2.3 * clamp(elev / 5, 0, 1);
      sun.visible = elev > -1;
      sun.position.copy(cam.position).addScaledVector(dir, 2000);
      sun.target.position.set(cam.position.x, 0, cam.position.z); sun.target.updateMatrixWorld();
      hemi.intensity = lerp(.06, .5, day);
      hemi.color.copy(hazeColor(elev)).lerp(new THREE.Color('#a9c4e8'), day * .6);
      renderer.toneMappingExposure = lerp(.7, .42, day);
      scene.fog.color.copy(hazeColor(elev));
      shared.night.value = clamp((2 - elev) / 8, 0, 1); // windows light up from about sunset
      if (!sky) scene.background = hazeColor(elev);
      render();
    },
    dispose() {
      ro.disconnect(); cancelAnimationFrame(raf);
      for (const d of disposables) d.dispose?.();
      if (sky) { sky.geometry.dispose(); sky.material.dispose(); }
      renderer.dispose(); el.remove();
    },
  };
  view.mount(host);
  aim();
  onStatus('');
  return view;
}
