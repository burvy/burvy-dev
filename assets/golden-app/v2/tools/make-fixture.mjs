// Writes fixtures/search_layers.json: a SearchResult in the v2 shape (goal layers, funnel stages,
// investigations, area name), built from the mock's fake downtown so it tells the same story as the
// approved mock. The maths is core's (crates/core/src/search.rs) and the browser rule in
// docs/contracts.md#goal-layers-and-stages:
//   one candidate per heat cell inside the circle; a must goal drops a cell where its score is <= 0,
//   a cost function's own `must` drops it too; raw = sum(weight * score) over goals with weight > 0;
//   heat = raw / max raw; spots = best raw first, ties to the lower cell, at least spacing_m apart.
// Usage: node web/v2/tools/make-fixture.mjs <out.json>
import { writeFileSync } from 'node:fs';
import { evaluate } from './expr.js';
import { featFor, LIBRARY, OBJECTS, CENTER } from './mock.js';

const EARTH_R = 6378137, RADIUS = 400, CELL = 20, SIDE = 40, MIN_SPACING = 40, MAX_SPOTS = 10;
const rad = d => d * Math.PI / 180, r5 = x => Math.round(x * 1e5) / 1e5, r3 = x => Math.round(x * 1e3) / 1e3;
const offset = (east, north) => ({
  lat: CENTER.lat + (north / EARTH_R) * 180 / Math.PI,
  lon: CENTER.lon + (east / (EARTH_R * Math.cos(rad(CENTER.lat)))) * 180 / Math.PI,
});
const metres = (a, b) => Math.hypot((b.lon - a.lon) * Math.PI / 180 * EARTH_R * Math.cos(rad(a.lat)), (b.lat - a.lat) * Math.PI / 180 * EARTH_R);

// same seeded value noise as mock.js (not exported there)
const hash = (x, y, s) => { const h = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return h - Math.floor(h); };
function noise(x, y, s, cell) {
  const gx = x / cell, gy = y / cell, x0 = Math.floor(gx), y0 = Math.floor(gy), fx = gx - x0, fy = gy - y0;
  const sm = t => t * t * (3 - 2 * t), a = hash(x0, y0, s), b = hash(x0 + 1, y0, s), c = hash(x0, y0 + 1, s), d = hash(x0 + 1, y0 + 1, s);
  return a + (b - a) * sm(fx) + (c - a) * sm(fy) + (a - b - c + d) * sm(fx) * sm(fy);
}

// ---- the brief the chat settled on (contract JSON)
const TOWER = { id: 'costar:52227' }, FOUNTAIN = { kind: 'fountain', near: TOWER, within_m: 300 };
const lib = n => LIBRARY.find(f => f.name === n);
const close = { name: 'close_to_tower', about: 'Within 650 m of Grand Tower, so it reads as a tower',
  must: { op: 'lt', x: { f: 'dist', obj: TOWER }, than: { num: 650 } }, score: { op: 'ramp', x: { f: 'dist', obj: TOWER }, lo: 650, hi: 150 } };
const GOALS = [
  { json: { def: close, must: true }, fn: close, args: {}, label: 'Close enough to Grand Tower' },
  { json: { use: 'in_front', args: { a: FOUNTAIN, b: TOWER }, must: true }, fn: lib('in_front'), args: { a: FOUNTAIN, b: TOWER }, label: 'The fountain in front of the tower' },
  { json: { use: 'sun_between', weight: 1 }, fn: lib('sun_between'), args: {} },
  { json: { use: 'high_vantage', weight: 0.5 }, fn: lib('high_vantage'), args: {} },
];
for (const g of GOALS) { g.must = !!g.json.must; g.weight = g.json.weight ?? (g.must ? 0 : 1); g.name = g.fn.name; }

const brief = {
  text: 'The fountain in front of Grand Tower with the sun going down between buildings, tonight',
  area: { lat: CENTER.lat, lon: CENTER.lon, radius_m: RADIUS, name: 'Downtown LA, around Grand Tower' },
  date: '2026-11-14', tz: 'America/Los_Angeles', event: 'sunset', standing: 'any', framing: 'between',
  max_spots: MAX_SPOTS, goals: GOALS.map(g => g.json), lens_deg: 60,
};

// ---- candidates: one per cell inside the circle; a few have nowhere to stand (null)
const cells = [];
let inCircle = 0;
for (let i = 0; i < SIDE * SIDE; i++) {
  const col = i % SIDE, row = Math.floor(i / SIDE);
  const east = (col + 0.5 - SIDE / 2) * CELL, north = (SIDE / 2 - row - 0.5) * CELL;
  if (Math.hypot(east, north) > RADIUS) { cells.push(null); continue; }
  inCircle++;
  if (noise(east, north, 9, 30) > 0.86) { cells.push(null); continue; } // spires, tree tops
  const roof = noise(east, north, 3, 55) > 0.56 ? 1 : 0;
  const c = { i, x: east, y: north, ...offset(east, north), roof, occl: noise(east, north, 5, 70),
    eye_m: roof ? Math.round(14 + noise(east, north, 4, 90) * 70) : 1.7, ground_m: Math.round(88 + north / 40) };
  c.feat = featFor(c);
  cells.push(c);
}
const sited = cells.filter(Boolean);

// ---- the funnel, short-circuited like core: gates in order (here close_to_tower, then in_front), each
// its formula's own `must` first, then its score if the goal is a must. At the first failure the candidate
// is dropped and nothing after is computed, so later layers are null there. Survivors get every goal.
const scoreOf = (g, c) => Math.max(0, Math.min(1, evaluate(g.fn.score, c.feat, g.args)));
const ownGate = (g, c) => !g.fn.must || evaluate(g.fn.must, c.feat, g.args) > 0;
const gatesInOrder = GOALS.filter(g => g.must || g.fn.must);
const layers = GOALS.map(g => ({ goal: g.name, weight: g.weight, must: g.must, heat: grid(cells.map(() => null)), gate: [] }));
for (const c of sited) {
  let out = false;
  for (const g of gatesInOrder) {
    const L = layers[GOALS.indexOf(g)];
    if (!ownGate(g, c)) { L.gate.push(c.i); out = true; break; }
    L.heat.values[c.i] = r3(scoreOf(g, c));
    if (g.must && !(L.heat.values[c.i] > 0)) { out = true; break; }
  }
  if (!out) GOALS.forEach((g, k) => (layers[k].heat.values[c.i] = r3(scoreOf(g, c))));
}
function grid(values) { return { center_lat: CENTER.lat, center_lon: CENTER.lon, cell_m: CELL, width: SIDE, height: SIDE, values }; }

// ---- the browser rule, as the contract states it (the Rust test checks the same thing)
const dropped = i => layers.some(L => L.gate.includes(i) || (L.must && !(L.heat.values[i] > 0)));
let ranked = layers.filter(L => L.weight > 0).map(L => [L, L.weight]);
if (!ranked.length) ranked = layers.map(L => [L, 1 / layers.length]);
const raw = cells.map(c => (c ? (dropped(c.i) ? 0 : ranked.reduce((s, [L, w]) => s + w * L.heat.values[c.i], 0)) : null));
const best = Math.max(0, ...raw.filter(v => v != null));
const heat = grid(raw.map(v => (v == null ? null : r3(best > 0 ? v / best : 0))));

// stages: fixed, then the gates (cheapest first: close_to_tower is Point, in_front is Ray)
const stages = [{ label: 'Points in the area', n: inCircle, goal: null }, { label: 'Somewhere to stand', n: sited.length, goal: null }];
let alive = sited.map(c => c.i);
for (const [gi, L] of layers.entries()) {
  if (!L.must && !L.gate.length) continue;
  alive = alive.filter(i => !(L.gate.includes(i) || (L.must && !(L.heat.values[i] > 0))));
  stages.push({ label: GOALS[gi].label, n: alive.length, goal: L.goal });
}

const spacing = Math.max(MIN_SPACING, 2 * CELL);
const order = sited.filter(c => raw[c.i] > 0).sort((a, b) => raw[b.i] - raw[a.i] || a.i - b.i);
const picked = [];
for (const c of order) {
  if (picked.length === MAX_SPOTS) break;
  if (picked.every(p => metres(p, c) >= spacing)) picked.push(c);
}

// ---- spots: what make_spot writes, with fake sun times and the AI's notes on the top five
const tower = OBJECTS[0], fountain = OBJECTS[2];
const NOTES = [
  ['confirmed', 'The oblique photo shows the plaza clear of trees on this line. The fountain basin sits in the lower third with the tower straight behind it, and the sun drops into the gap to its left.'],
  ['caution', 'Good line, but a construction crane shows in the 2025 photo just right of the tower. Frame tighter or check that it has gone.'],
  ['confirmed', 'The photo shows a low parapet, not a wall, so the fountain is in full view from standing height.'],
  ['caution', 'The fountain is half hidden by the bus shelter from standing height. Fine if you can raise the camera about 2 m.'],
  ['confirmed', 'Wide and calm, with the whole plaza in frame. The sun clips the tower edge for a few minutes before it sets.'],
];
const why1 = (g, c) => {
  const f = c.feat, d = o => Math.round(f('dist', [{ id: o.id }]));
  switch (g.name) {
    case 'close_to_tower': return `${g.fn.about}: ${d(tower)} m from it`;
    case 'in_front': return `${g.fn.about}: fountain ${d(fountain)} m away, tower ${d(tower)} m, ${Math.round(f('bearing_diff', [{ id: fountain.id }, { id: tower.id }]))} deg apart in the frame`;
    case 'sun_between': return `${g.fn.about}: ${f('gap_width_deg', [])} deg gap, ${f('sun_visible_min', [])} min of golden hour in view`;
    case 'high_vantage': return c.roof ? `${g.fn.about}: roof, eye ${c.eye_m} m above the street` : `${g.fn.about}: street level`;
  }
};
const hhmm = m => `2026-11-14T${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}:00-08:00`;
const spots = picked.map((c, k) => {
  const lat = r5(c.lat), lon = r5(c.lon), standing = c.roof ? 'roof' : 'ground', vis = c.feat('sun_visible_min', []);
  const scores = GOALS.map(g => ({ goal: g.name, score: r3(scoreOf(g, c)), why: why1(g, c) }));
  const place = c.roof ? `Roof ${Math.round(c.eye_m - 1.7)} m above the street` : 'At street level';
  const spot = {
    id: `s-${lat.toFixed(5)}-${lon.toFixed(5)}-${standing}`, rank: k + 1, score: r3(raw[c.i] / best), lat, lon, standing,
    ground_m: c.ground_m, eye_m: r3(c.ground_m + c.eye_m), sun: {
      date: '2026-11-14', flat_sunset: hhmm(16 * 60 + 48), hidden_at: hhmm(16 * 60 + 48 - Math.round(4 + 8 * c.occl)),
      hidden_az_deg: 246.1, hidden_elev_deg: r3(1 + 2 * c.occl), golden_min_visible: vis },
    why: [...scores.map(s => s.why), `${vis} min of golden hour with the sun in view`, place],
    building_id: c.roof ? `costar:${1000000 + c.i}` : null, scores,
  };
  if (k < NOTES.length) spot.investigation = { verdict: NOTES[k][0], note: NOTES[k][1] };
  return spot;
});

const objects = OBJECTS.map(o => ({ id: o.id, kind: o.kind, ...((ll) => ({ lat: r5(ll.lat), lon: r5(ll.lon) }))(offset(o.x, o.y)),
  base_m: 88, top_m: 88 + o.top_m, radius_m: o.radius_m, source: o.id.split(':')[0], name: o.name }));
const out = { brief, spots, heat, searched: sited.length, took_ms: 2140, layers, stages, spacing_m: spacing, objects };
writeFileSync(process.argv[2], JSON.stringify(out) + '\n');
console.log(`cells ${inCircle}, sited ${sited.length}, stages ${stages.map(s => s.n).join(' > ')}, spots ${spots.length}, bytes ${JSON.stringify(out).length}`);
