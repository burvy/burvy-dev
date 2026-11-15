// The browser side of docs/contracts.md#goal-layers-and-stages. No formulas here: the server scores
// every goal at every candidate (one per heat cell), and this file only re-weights, masks and re-picks.
// `rebuild` is the same rule as core and as `rebuild` in crates/contracts/tests/fixtures.rs. Pure, no DOM.

const EARTH_R = 6378137;
const rad = d => d * Math.PI / 180;

/** Centre of heat cell `i` (row 0 north, row-major), with the flat formula DsmGrid uses. */
export function cellLatLon(h, i) {
  const col = i % h.width, row = Math.floor(i / h.width);
  const east = (col + 0.5 - h.width / 2) * h.cell_m, north = (h.height / 2 - row - 0.5) * h.cell_m;
  return { lat: h.center_lat + north / EARTH_R * 180 / Math.PI, lon: h.center_lon + east / (EARTH_R * Math.cos(rad(h.center_lat))) * 180 / Math.PI };
}

/** The cell a point falls in, or -1 off the grid. */
export function cellOf(h, lat, lon) {
  const north = rad(lat - h.center_lat) * EARTH_R, east = rad(lon - h.center_lon) * EARTH_R * Math.cos(rad(h.center_lat));
  const col = Math.floor(east / h.cell_m + h.width / 2), row = Math.floor(h.height / 2 - north / h.cell_m);
  return col < 0 || row < 0 || col >= h.width || row >= h.height ? -1 : row * h.width + col;
}

export function metres(a, b) {
  const north = rad(b.lat - a.lat) * EARTH_R, east = rad(b.lon - a.lon) * EARTH_R * Math.cos(rad(a.lat));
  return Math.hypot(east, north);
}

/** The goals as the user has them now: `[{layer, weight, must, removed}]`, same order as `result.layers`. */
export function initialGoals(result) {
  return result.layers.map(L => ({ weight: L.weight, must: L.must, removed: false }));
}

/** Is cell `i` dropped by goal `k` as it stands? Its formula's own gate always counts; `must` only if on. */
function dropsAt(L, g, i) {
  if (g.removed) return false;
  return L.gateSet.has(i) || (g.must && !(L.heat.values[i] > 0));
}

/** Combined heat, the picked cells and per-cell drop reasons, from the layers and the goals as edited. */
export function rebuild(result, goals, n) {
  const layers = result.layers, h = result.heat, cells = h.values.length;
  layers.forEach(L => (L.gateSet ??= new Set(L.gate || [])));
  const live = layers.map((L, k) => [L, goals[k]]).filter(([, g]) => !g.removed);
  // contract rule: goals with weight > 0 rank (a must normally has weight 0; the page sets that when you tick it)
  let ranked = live.filter(([, g]) => g.weight > 0).map(([L, g]) => [L, g.weight]);
  if (!ranked.length) ranked = live.map(([L]) => [L, 1 / Math.max(1, live.length)]);

  const raw = new Array(cells).fill(null), droppedBy = new Array(cells).fill(-1);
  for (let i = 0; i < cells; i++) {
    if (h.values[i] == null) continue;
    const k = layers.findIndex((L, j) => dropsAt(L, goals[j], i));
    if (k >= 0) { droppedBy[i] = k; raw[i] = 0; continue; }
    raw[i] = ranked.reduce((s, [L, w]) => s + w * (L.heat.values[i] ?? 0), 0);
  }
  const best = raw.reduce((m, v) => (v != null && v > m ? v : m), 0);
  const heat = raw.map(v => (v == null ? null : best > 0 ? v / best : 0));

  const order = [];
  for (let i = 0; i < cells; i++) if (raw[i] > 0) order.push(i);
  order.sort((a, b) => raw[b] - raw[a] || a - b);
  const spacing = result.spacing_m ?? Math.max(40, 2 * h.cell_m), picks = [], at = [];
  for (const i of order) {
    if (picks.length === n) break;
    const p = cellLatLon(h, i);
    if (at.every(q => metres(q, p) >= spacing)) { picks.push(i); at.push(p); }
  }
  return { raw, heat, picks, droppedBy, best };
}

/** The funnel as the goals stand: the server's fixed stages, then one per active gate (server order first). */
export function stagesFor(result, goals, run) {
  const fixed = result.stages.filter(s => s.goal == null);
  const order = result.stages.filter(s => s.goal != null).map(s => result.layers.findIndex(L => L.goal === s.goal));
  result.layers.forEach((_, k) => { if (!order.includes(k)) order.push(k); });
  const h = result.heat, out = fixed.map(s => ({ ...s }));
  let alive = [];
  for (let i = 0; i < h.values.length; i++) if (h.values[i] != null) alive.push(i);
  for (const k of order) {
    const L = result.layers[k], g = goals[k];
    if (k < 0 || g.removed || !(g.must || L.gateSet.size)) continue;
    alive = alive.filter(i => !dropsAt(L, g, i));
    const server = result.stages.find(s => s.goal === L.goal);
    out.push({ label: server?.label, n: alive.length, goal: L.goal, k });
  }
  return out;
}

/** Edits the layers can't answer: a goal that dropped points (a must, or a formula with its own gate) was
 *  switched off or removed. Points it dropped were never scored on the later goals (core stops at the first
 *  gate), so letting them back in needs a new POST /api/search. Weights and must-on are exact here. */
export function needsServer(result, goals) {
  return result.layers.some((L, k) => (L.must || (L.gate || []).length) && (goals[k].removed || (L.must && !goals[k].must)));
}

/** The server's spot at a picked cell, if it sent one there (spot lat/lon are rounded to 1e-5). */
export function serverSpotAt(result, i) {
  return result.spots.find(s => cellOf(result.heat, s.lat, s.lon) === i) || null;
}

/** The goals as edited, back into brief JSON for a new POST /api/search. */
export function editedGoals(brief, goals) {
  return brief.goals.map((g, k) => [g, goals[k]]).filter(([, e]) => !e.removed).map(([g, e]) => {
    const out = { ...g };
    delete out.weight; delete out.must;
    if (e.must) { out.must = true; if (e.weight > 0) out.weight = e.weight; } else out.weight = e.weight;
    return out;
  });
}

/** A result from a server that doesn't send layers yet (before task 059): one layer, the combined heat. */
export function withLayers(result) {
  if (result.layers?.length) return result;
  const searched = result.heat.values.filter(v => v != null).length, goals = result.brief.goals || [], one = goals.length === 1 ? goals[0] : null;
  const best = Math.max(0, ...result.heat.values.filter(v => v != null));
  return { ...result, legacy: true, brief: { ...result.brief, goals: one ? goals : [] }, spacing_m: result.spacing_m ?? Math.max(40, 2 * result.heat.cell_m),
    // one goal: the heat is that goal's score (scaled to the best); more: the combined fit, which can't be split
    layers: [{ goal: one ? (one.use || one.def?.name) : goals.length ? 'fit' : `sun_${result.brief.framing || 'clear'}`, weight: one ? (one.weight ?? (one.must ? 0 : 1)) : 1, must: !!one?.must, heat: result.heat, gate: [] }],
    stages: result.stages?.length ? result.stages : [{ label: 'Somewhere to stand', n: result.searched ?? searched, goal: null }],
    objects: result.objects || [], best };
}
