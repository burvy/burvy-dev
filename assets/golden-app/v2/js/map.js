// The map: Vexcel ortho tiles from golden's server, every searched point as a soft heat, points a must
// dropped under a veil, the objects the goals measure from, and the top spots as numbered pins.
// Pans and zooms freely (drag / wheel / two fingers / buttons). When the view has moved off the searched
// area it offers "Search this area": the map centre, and a radius that fits the view (100-800 m).
import { cellLatLon } from './layers.js';

const TILE = 256, MIN_Z = 13, MAX_Z = 20, TILE_Z = [14, 19]; // golden's cache holds ortho z14-19
const MAX_RADIUS = 800, MIN_RADIUS = 100;
const rad = d => d * Math.PI / 180;
const world = (lat, lon, z) => {
  const s = Math.sin(rad(lat)), n = TILE * 2 ** z;
  return { x: (lon + 180) / 360 * n, y: (0.5 - Math.log((1 + s) / (1 - s)) / (4 * Math.PI)) * n };
};
const unworld = (x, y, z) => {
  const n = TILE * 2 ** z, lon = x / n * 360 - 180, m = Math.PI * (1 - 2 * y / n);
  return { lat: Math.atan(Math.sinh(m)) * 180 / Math.PI, lon };
};
const mPerPx = (lat, z) => 156543.03392 * Math.cos(rad(lat)) / 2 ** z;
const tiles = new Map();

export function createMap(canvas, { base, onPick, onSearchArea, onMove }) {
  const ctx = canvas.getContext('2d'), off = document.createElement('canvas');
  const view = { lat: 34.0493, lon: -118.2508, z: 16 };
  let data = null, tilesOk = !!base, searched = null, raf = 0;

  const size = () => ({ W: canvas.clientWidth, H: canvas.clientHeight });
  function toPx(lat, lon) {
    const { W, H } = size(), c = world(view.lat, view.lon, view.z), p = world(lat, lon, view.z);
    return { x: W / 2 + p.x - c.x, y: H / 2 + p.y - c.y };
  }
  function fromPx(x, y) {
    const { W, H } = size(), c = world(view.lat, view.lon, view.z);
    return unworld(c.x + x - W / 2, c.y + y - H / 2, view.z);
  }
  const redraw = () => { cancelAnimationFrame(raf); raf = requestAnimationFrame(draw); };

  function tile(z, x, y) {
    const k = `${z}/${x}/${y}`;
    let im = tiles.get(k);
    if (!im) { im = new Image(); im.onload = redraw; im.onerror = () => { im.bad = true; }; im.src = `${base}/api/tiles/vexcel/${k}`; tiles.set(k, im); }
    return im;
  }
  function drawTiles(W, H) {
    const tz = Math.max(TILE_Z[0], Math.min(TILE_Z[1], Math.round(view.z))), scale = 2 ** (view.z - tz);
    const c = world(view.lat, view.lon, tz), x0 = c.x - W / 2 / scale, y0 = c.y - H / 2 / scale;
    let any = false;
    for (let tx = Math.floor(x0 / TILE); tx <= Math.floor((x0 + W / scale) / TILE); tx++)
      for (let ty = Math.floor(y0 / TILE); ty <= Math.floor((y0 + H / scale) / TILE); ty++) {
        const im = tile(tz, tx, ty);
        if (im.complete && im.naturalWidth) { any = true; ctx.drawImage(im, (tx * TILE - x0) * scale, (ty * TILE - y0) * scale, TILE * scale + 0.6, TILE * scale + 0.6); }
      }
    return any;
  }

  function cellRect(h, i, px) {
    const p = cellLatLon(h, i), q = toPx(p.lat, p.lon);
    return [q.x - px / 2, q.y - px / 2, px + 0.6, px + 0.6];
  }

  function draw() {
    const dpr = devicePixelRatio || 1, { W, H } = size();
    if (!W || !H) return;
    if (canvas.width !== Math.round(W * dpr) || canvas.height !== Math.round(H * dpr)) { canvas.width = Math.round(W * dpr); canvas.height = Math.round(H * dpr); }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = '#dce9f4'; ctx.fillRect(0, 0, W, H);
    if (tilesOk) drawTiles(W, H);
    if (!data) return;
    const { result, run, layer, colors, picks, sel, fresh } = data, h = result.heat;
    const px = h.cell_m / mPerPx(view.lat, view.z);

    // what a must dropped: a cool veil (all of them, or just the selected goal's)
    ctx.fillStyle = 'rgba(12,42,67,.30)';
    run.droppedBy.forEach((k, i) => { if (k >= 0 && (layer === 'all' || layer === k)) ctx.fillRect(...cellRect(h, i, px)); });

    // the heat, blurred offscreen: combined fit (sunset ramp), or one goal's raw score in its colour
    off.width = canvas.width; off.height = canvas.height;
    const o = off.getContext('2d'); o.setTransform(dpr, 0, 0, dpr, 0, 0);
    const vals = layer === 'all' ? run.heat : result.layers[layer].heat.values;
    vals.forEach((v, i) => {
      if (v == null || v < 0.06 || (layer === 'all' && run.droppedBy[i] >= 0)) return;
      o.fillStyle = layer === 'all' ? ramp(v) : hexA(colors[layer], 0.12 + 0.78 * v);
      o.fillRect(...cellRect(h, i, px));
    });
    ctx.save(); ctx.setTransform(1, 0, 0, 1, 0, 0); ctx.filter = `blur(${Math.max(1.5, px * dpr * 0.35)}px)`; ctx.globalAlpha = 0.85; ctx.drawImage(off, 0, 0); ctx.restore();

    // the searched area, and the one "Search this area" would use
    const ring = (a, style, dash) => {
      const c = toPx(a.lat, a.lon), r = a.radius_m / mPerPx(view.lat, view.z);
      ctx.save(); ctx.setLineDash(dash); ctx.lineWidth = 2; ctx.strokeStyle = style; ctx.beginPath(); ctx.arc(c.x, c.y, r, 0, 7); ctx.stroke(); ctx.restore();
    };
    ring(result.brief.area, 'rgba(255,255,255,.9)', [2, 5]);
    const prop = proposal();
    if (prop) ring(prop, 'rgba(242,107,29,.95)', [8, 6]);

    // the selected spot: the direction the sun goes down, from the server's numbers
    const sp = sel != null ? picks[sel] : null;
    if (sp?.spot?.sun?.hidden_az_deg != null) {
      const p = toPx(sp.lat, sp.lon), a = rad(sp.spot.sun.hidden_az_deg), L = Math.max(W, H);
      ctx.save(); ctx.setLineDash([6, 6]); ctx.lineWidth = 2; ctx.strokeStyle = '#f9a03f';
      ctx.beginPath(); ctx.moveTo(p.x, p.y); ctx.lineTo(p.x + Math.sin(a) * L, p.y - Math.cos(a) * L); ctx.stroke(); ctx.restore();
    }
    // the objects the goals talk about
    for (const ob of result.objects || []) {
      const p = toPx(ob.lat, ob.lon);
      ctx.save(); ctx.fillStyle = '#fff'; ctx.strokeStyle = '#0c2a43'; ctx.lineWidth = 2;
      ctx.beginPath(); ob.kind === 'building' ? ctx.rect(p.x - 6, p.y - 6, 12, 12) : ctx.arc(p.x, p.y, 6, 0, 7); ctx.fill(); ctx.stroke();
      const name = ob.name || ob.kind;
      ctx.font = '600 12.5px "Instrument Sans",sans-serif'; ctx.lineWidth = 3.5; ctx.strokeStyle = 'rgba(12,42,67,.85)'; ctx.fillStyle = '#fff';
      ctx.strokeText(name, p.x + 10, p.y + 4); ctx.fillText(name, p.x + 10, p.y + 4); ctx.restore();
    }
    // the top spots, numbered; ones the server hasn't seen yet get a dashed ring
    picks.forEach((s, n) => {
      const p = toPx(s.lat, s.lon), on = n === sel, r = on ? 15 : 12;
      ctx.save(); ctx.shadowColor = 'rgba(12,42,67,.45)'; ctx.shadowBlur = 6;
      ctx.fillStyle = on ? '#f26b1d' : '#fff'; ctx.beginPath(); ctx.arc(p.x, p.y, r, 0, 7); ctx.fill(); ctx.shadowBlur = 0;
      ctx.lineWidth = 2.5; ctx.strokeStyle = on ? '#fff' : '#f26b1d'; if (fresh?.has(n)) ctx.setLineDash([4, 3]); ctx.stroke(); ctx.setLineDash([]);
      ctx.fillStyle = '#0c2a43'; ctx.font = `800 ${on ? 15 : 13}px "Bricolage Grotesque",sans-serif`; ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
      ctx.fillText(n + 1, p.x, p.y + 1); ctx.restore();
    });
  }

  // ---- "Search this area": offered once the view is well off the searched circle
  function proposal() {
    if (!data || !searched) return null;
    const { W, H } = size(), m = mPerPx(view.lat, view.z);
    const radius_m = Math.round(Math.max(MIN_RADIUS, Math.min(MAX_RADIUS, Math.min(W, H) / 2 * m * 0.85)) / 10) * 10;
    const a = searched, moved = Math.hypot(rad(view.lat - a.lat) * 6378137, rad(view.lon - a.lon) * 6378137 * Math.cos(rad(a.lat)));
    const resized = Math.abs(Math.log2(radius_m / a.radius_m));
    return moved > a.radius_m * 0.3 || resized > 0.8 ? { lat: +view.lat.toFixed(5), lon: +view.lon.toFixed(5), radius_m } : null;
  }
  const moved = () => { redraw(); onMove?.(proposal()); };

  // ---- gestures: mouse drags, wheel zooms; on touch one finger scrolls the page, two move the map
  const pts = new Map();
  let drag = null, tap = null;
  canvas.addEventListener('pointerdown', e => {
    pts.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (e.pointerType === 'mouse' || pts.size === 2) { canvas.setPointerCapture(e.pointerId); drag = gesture(); }
    tap = pts.size === 1 ? { x: e.clientX, y: e.clientY, t: performance.now() } : null;
  });
  canvas.addEventListener('pointermove', e => {
    if (!pts.has(e.pointerId)) return;
    pts.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (!drag || (e.pointerType !== 'mouse' && pts.size < 2)) return;
    const g = gesture(), r = canvas.getBoundingClientRect();
    if (drag.d && g.d) zoomAt(Math.log2(g.d / drag.d), g.x - r.left, g.y - r.top, false);
    panBy(g.x - drag.x, g.y - drag.y);
    drag = g; canvas.style.cursor = 'grabbing';
    if (tap && Math.hypot(e.clientX - tap.x, e.clientY - tap.y) > 6) tap = null;
  });
  const up = e => {
    pts.delete(e.pointerId);
    if (tap && e.type === 'pointerup' && performance.now() - tap.t < 500) click(e);
    tap = null; drag = pts.size === 2 || (e.pointerType === 'mouse' && pts.size) ? gesture() : null; canvas.style.cursor = '';
    if (!drag) moved();
  };
  canvas.addEventListener('pointerup', up);
  canvas.addEventListener('pointercancel', up);
  function gesture() {
    const a = [...pts.values()], x = a.reduce((s, p) => s + p.x, 0) / a.length, y = a.reduce((s, p) => s + p.y, 0) / a.length;
    return { x, y, d: a.length === 2 ? Math.hypot(a[0].x - a[1].x, a[0].y - a[1].y) : 0 };
  }
  function panBy(dx, dy) { const { W, H } = size(), c = fromPx(W / 2 - dx, H / 2 - dy); view.lat = c.lat; view.lon = c.lon; redraw(); }
  function zoomAt(dz, x, y, settle = true) {
    const nz = Math.max(MIN_Z, Math.min(MAX_Z, view.z + dz)); if (nz === view.z) return;
    const before = fromPx(x, y); view.z = nz; const after = fromPx(x, y);
    view.lat += before.lat - after.lat; view.lon += before.lon - after.lon;
    settle ? moved() : redraw();
  }
  canvas.addEventListener('wheel', e => {
    e.preventDefault(); const r = canvas.getBoundingClientRect();
    zoomAt(-e.deltaY * (e.deltaMode ? 0.05 : 0.0022), e.clientX - r.left, e.clientY - r.top);
  }, { passive: false });
  function click(e) {
    if (!data) return;
    const r = canvas.getBoundingClientRect(), mx = e.clientX - r.left, my = e.clientY - r.top;
    let best = null, bd = 26;
    data.picks.forEach((s, n) => { const p = toPx(s.lat, s.lon), d = Math.hypot(p.x - mx, p.y - my); if (d < bd) { bd = d; best = n; } });
    if (best != null) onPick(best);
  }
  new ResizeObserver(redraw).observe(canvas);

  function fit(a) {
    const { W, H } = size(); if (!W) return;
    view.lat = a.lat; view.lon = a.lon;
    const m = (a.radius_m * 2.2) / Math.min(W, H); // metres per css px so the circle fits
    view.z = Math.max(MIN_Z, Math.min(MAX_Z, Math.log2(156543.03392 * Math.cos(rad(a.lat)) / m)));
    moved();
  }

  return {
    set(d) { const first = !data || d.result !== data.result; data = d; if (first) { searched = d.result.brief.area; } redraw(); onMove?.(proposal()); },
    fit, zoom: dz => { const { W, H } = size(); zoomAt(dz, W / 2, H / 2); },
    fitArea: () => data && fit(data.result.brief.area),
    centreOn(lat, lon) { view.lat = lat; view.lon = lon; moved(); },
    proposal,
    attribution: () => (tilesOk ? 'Imagery © Vexcel Imaging' : 'No imagery: golden’s server isn’t reachable'),
    noTiles() { tilesOk = false; redraw(); },
    draw: redraw,
  };
}

// combined fit: pale sky -> golden -> sunset orange -> deep sunset
function ramp(v) {
  const stops = [[0, [61, 155, 224, 0.0]], [0.3, [249, 200, 120, 0.55]], [0.65, [249, 160, 63, 0.8]], [1, [214, 72, 12, 0.95]]];
  let i = 1; while (i < stops.length - 1 && v > stops[i][0]) i++;
  const [a0, c0] = stops[i - 1], [a1, c1] = stops[i], t = (v - a0) / (a1 - a0);
  const c = c0.map((x, k) => x + (c1[k] - x) * t);
  return `rgba(${c[0] | 0},${c[1] | 0},${c[2] | 0},${c[3].toFixed(2)})`;
}
export const RAMP_CSS = 'linear-gradient(90deg,rgba(61,155,224,.15),rgba(249,200,120,.8),#f9a03f,#d6480c)';
function hexA(hex, a) { const n = parseInt(hex.slice(1), 16); return `rgba(${n >> 16},${(n >> 8) & 255},${n & 255},${a})`; }
