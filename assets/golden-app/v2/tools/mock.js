// Fake data shaped like the v2 contract (docs/metrics.md v2 + docs/contracts.md ChatRequest/ChatReply),
// so the page can be judged before the server speaks v2. Everything here is replaced by server replies:
//   mockChat(req)      -> POST /api/chat        {understood, ask?{question, choices}, brief?, sheet?, model}
//   LIBRARY            -> the agent's list_cost_functions (library/*.json + cache/library)
//   sheet.candidates   -> PROPOSED: the funnel's candidates with their features, so goal edits re-rank locally
// Downtown LA around the hero spot; positions are metres east (x) / north (y) of CENTER.

export const CENTER = { lat: 34.0493, lon: -118.2508 };
const M_LAT = 111320, M_LON = 111320 * Math.cos(CENTER.lat * Math.PI / 180);
export const toLatLon = (x, y) => ({ lat: CENTER.lat + y / M_LAT, lon: CENTER.lon + x / M_LON });
export const SUN_AZ = 246; // sunset azimuth, mid-November

export const OBJECTS = [
  { id: 'costar:52227', kind: 'building', name: 'Grand Tower', floors: 48, x: -265, y: -112, top_m: 220, radius_m: 22 },
  { id: 'costar:10504968', kind: 'building', name: 'Hope Place', floors: 31, x: -330, y: 120, top_m: 140, radius_m: 18 },
  { id: 'vexcel:det-118', kind: 'fountain', name: 'Plaza fountain', x: -118, y: -58, top_m: 4, radius_m: 9 },
];

// ---- value noise, seeded, so the fake scene looks like a city and not like static
const hash = (x, y, s) => { const h = Math.sin(x * 127.1 + y * 311.7 + s * 74.7) * 43758.5453; return h - Math.floor(h); };
function noise(x, y, s, cell) {
  const gx = x / cell, gy = y / cell, x0 = Math.floor(gx), y0 = Math.floor(gy), fx = gx - x0, fy = gy - y0;
  const sm = t => t * t * (3 - 2 * t), a = hash(x0, y0, s), b = hash(x0 + 1, y0, s), c = hash(x0, y0 + 1, s), d = hash(x0 + 1, y0 + 1, s);
  return a + (b - a) * sm(fx) + (c - a) * sm(fy) + (a - b - c + d) * sm(fx) * sm(fy);
}
const deg = r => r * 180 / Math.PI;
const bearing = (c, o) => (deg(Math.atan2(o.x - c.x, o.y - c.y)) + 360) % 360;
const adiff = (a, b) => Math.abs(((a - b + 540) % 360) - 180);
const bumpf = (x, at, w) => Math.max(0, 1 - Math.abs(x - at) / w);

// ---- candidates: a 32 x 32 grid, 1,024 points, 40 m apart (today's HeatLayout is the same idea)
export const GRID = 32, STEP_M = 40;
export const CANDIDATES = [];
for (let r = 0; r < GRID; r++) for (let k = 0; k < GRID; k++) {
  const x = (k - GRID / 2 + 0.5) * STEP_M + 60, y = (GRID / 2 - r - 0.5) * STEP_M - 20;
  const roof = noise(x, y, 3, 55) > 0.56 ? 1 : 0;
  CANDIDATES.push({ id: `p${r}-${k}`, r, k, x, y, ...toLatLon(x, y), roof,
    eye_m: roof ? Math.round(14 + noise(x, y, 4, 90) * 70) : 1.7, ground_m: Math.round(88 + y / 40), occl: noise(x, y, 5, 70) });
}

/** The features of one candidate (metrics.md "Features"), faked from geometry + noise. */
export function featFor(c) {
  const find = ref => {
    if (!ref) return OBJECTS[0];
    if (ref.id) return OBJECTS.find(o => o.id === ref.id);
    return OBJECTS.filter(o => o.kind === ref.kind).sort((a, b) => Math.hypot(a.x - c.x, a.y - c.y) - Math.hypot(b.x - c.x, b.y - c.y))[0];
  };
  const dist = o => Math.hypot(o.x - c.x, o.y - c.y);
  const visible = o => Math.max(0, Math.min(1, 1.25 - dist(o) / 1100 - 0.75 * c.occl + (c.roof ? 0.35 : 0) + o.top_m / 600));
  const tower = OBJECTS[0], sbd = adiff(SUN_AZ, bearing(c, tower));
  const sunNoise = noise(c.x, c.y, 7, 80);
  return (name, refs) => {
    const [a, b] = refs.map(find);
    switch (name) {
      case 'ground_m': return c.ground_m;
      case 'eye_m': return c.eye_m;
      case 'roof': return c.roof;
      case 'dist': return dist(a);
      case 'bearing': return bearing(c, a);
      case 'bearing_diff': return adiff(bearing(c, a), bearing(c, b));
      case 'size_deg': return deg(Math.atan2(a.top_m, dist(a)));
      case 'visible': return visible(a);
      case 'sun_between': return Math.min(0.94, bumpf(sbd, 13, 10) * (0.5 + 0.5 * sunNoise) + (c.roof ? 0.1 : 0));
      case 'sun_behind': return bumpf(sbd, 0, 7);
      case 'sun_clear': return Math.max(0, 1 - 0.9 * c.occl - (sbd < 8 ? 0.5 : 0));
      case 'sun_visible_min': return Math.round(10 + 30 * sunNoise);
      case 'gap_width_deg': return +(2 + 6 * sunNoise).toFixed(1);
      case 'sun_bearing_diff': return adiff(SUN_AZ, bearing(c, a));
    }
    throw Error('unknown feature ' + name);
  };
}

// ---- the library: seed entries + ones the agent saved (same JSON as library/*.json would hold)
const F = (f, obj) => (obj ? { f, obj } : { f });
export const LIBRARY = [
  { name: 'sun_between', about: 'The sun sets in a gap between buildings', params: [], score: F('sun_between'), seed: true },
  { name: 'sun_behind', about: 'The sun sets right behind a building, for a rim of light', params: [], score: F('sun_behind'), seed: true },
  { name: 'sun_clear', about: 'A clear view of the sun all the way down', params: [], score: F('sun_clear'), seed: true },
  { name: 'in_front', about: 'One thing in front of another, both in one frame', params: ['a', 'b'], seed: true,
    must: { op: 'and', of: [{ op: 'lt', x: { f: 'bearing_diff', a: 'a', b: 'b' }, than: { num: 30 } },
                            { op: 'lt', x: F('dist', 'a'), than: F('dist', 'b') },
                            { op: 'gt', x: F('visible', 'b'), than: { num: 0.4 } }] },
    score: { op: 'avg', of: [{ op: 'bump', x: { f: 'bearing_diff', a: 'a', b: 'b' }, at: 0, width: 30 }, F('visible', 'a'), F('visible', 'b')] } },
  { name: 'lined_up', about: 'Two things stacked in one line, one right behind the other', params: ['a', 'b'], seed: true,
    score: { op: 'bump', x: { f: 'bearing_diff', a: 'a', b: 'b' }, at: 0, width: 4 } },
  { name: 'fills', about: 'The subject fills the frame: not a speck, not cropped', params: ['a'], seed: true,
    score: { op: 'bump', x: F('size_deg', 'a'), at: 18, width: 12 } },
  { name: 'high_vantage', about: 'Higher up: a roof beats the street', params: [], seed: true,
    score: { op: 'max', of: [F('roof'), { op: 'ramp', x: F('eye_m'), lo: 2, hi: 40 }] } },
  { name: 'street_level', about: 'From the street, no roof access needed', params: [], seed: true,
    must: { op: 'not', x: F('roof') }, score: { op: 'not', x: F('roof') } },
  { name: 'long_sun_window', about: 'The sun stays in view for at least 20 minutes of golden hour', params: [], saved: '11/14, from a Princeton brief',
    score: { op: 'ramp', x: F('sun_visible_min'), lo: 10, hi: 30 } },
  { name: 'tower_and_sun_aligned', about: 'The setting sun sits just beside the tower, not hidden by it', params: ['a'], saved: '11/14, from a hero brief',
    score: { op: 'bump', x: F('sun_bearing_diff', 'a'), at: 9, width: 8 } },
];

// ---- the goals the agent settles on for the hero brief (metrics.md "Goals in the brief")
const TOWER = { id: 'costar:52227' }, FOUNTAIN = { kind: 'fountain', near: { id: 'costar:52227' }, within_m: 300 };
export const HERO_GOALS = [
  { key: 'near', def: { name: 'close_to_tower', about: 'Within 650 m of Grand Tower, so it reads as a tower', params: ['b'],
      must: { op: 'lt', x: F('dist', 'b'), than: { num: 650 } }, score: { op: 'ramp', x: F('dist', 'b'), lo: 650, hi: 150 } },
    args: { b: TOWER }, must: true },
  { key: 'front', use: 'in_front', args: { a: FOUNTAIN, b: TOWER }, must: true },
  { key: 'sun', use: 'sun_between', weight: 1.0 },
  { key: 'high', use: 'high_vantage', weight: 0.5 },
];

/** A plain-English reason for one goal's score at one spot ("why" in Spot.scores). */
export function whyFor(R, c) {
  const f = featFor(c), t = OBJECTS[0], fo = OBJECTS[2];
  switch (R.fn.name) {
    case 'close_to_tower': return `${Math.round(f('dist', [{ id: t.id }]))} m from ${t.name}`;
    case 'in_front': return `Fountain ${Math.round(f('dist', [{ id: fo.id }]))} m away, tower ${Math.round(f('dist', [{ id: t.id }]))} m, ${Math.round(f('bearing_diff', [{ id: fo.id }, { id: t.id }]))}° apart in the frame`;
    case 'sun_between': return `Sets into a ${f('gap_width_deg', [])}° gap, ${f('sun_visible_min', [])} min of golden hour in view`;
    case 'high_vantage': return c.roof ? `Roof, eye ${c.eye_m} m above the street` : 'Street level';
    case 'sun_behind': return `Sun ${Math.round(f('sun_bearing_diff', [{ id: t.id }]))}° from the tower's bearing`;
    case 'street_level': return c.roof ? 'Needs a roof' : 'On the street';
    default: return R.fn.about;
  }
}

// What the AI wrote after looking at each top spot's oblique photo (the `investigate` tool), best first.
export const INVESTIGATIONS = [
  { verdict: 'confirmed', note: 'The oblique photo shows the plaza clear of trees on this line. The fountain basin sits in the lower third with the tower straight behind it. The sun drops into the gap to its left at 4:38.' },
  { verdict: 'confirmed', note: 'Good line, but a construction crane shows in the 2025 photo just right of the tower. Frame tighter or check that it has gone.' },
  { verdict: 'moved up', note: 'The numbers rated this lower, but the photo shows a low parapet, not a wall, so the fountain is in full view. I moved it up one place.' },
  { verdict: 'caution', note: 'The fountain is half hidden by the bus shelter from standing height. Fine if you can raise the camera about 2 m.' },
  { verdict: 'confirmed', note: 'Wide and calm, with the whole plaza in frame. The sun clips the tower edge for about 6 minutes before it sets.' },
];
// hero spots golden's cache has oblique photos and mesh for (the mock's spots borrow these)
export const CACHED_VIEWS = [
  { lat: 34.04934, lon: -118.25076, az: 246 }, { lat: 34.04947, lon: -118.25027, az: 246 },
  { lat: 34.04705, lon: -118.25027, az: 247 }, { lat: 34.04934, lon: -118.25076, az: 240 },
];
export const CONTACTS = [
  { who: 'Building manager', org: 'Bunker Hill Plaza LLC', phone: '(213) 555-0142' },
  { who: 'Leasing office', org: 'Hope & 5th Partners', phone: '(213) 555-0187' },
  { who: 'Plaza security desk', org: 'City of LA, Pershing Sq. district', phone: '(213) 555-0110' },
];

// ---- the chat: POST /api/chat, stateless, the whole conversation each turn (docs/contracts.md)
const ASKS = [
  { understood: ['Sunset', 'A fountain in front of a tower', 'Downtown LA (my pick)'],
    ask: { question: 'Which tower? There are two tall ones within 300 m of a fountain downtown.',
           choices: ['Grand Tower, 48 floors', 'Hope Place, 31 floors', 'Either one'] } },
  { understood: ['Sunset', 'Plaza fountain in front of Grand Tower', 'Downtown LA (my pick)'],
    ask: { question: 'Rooftop or street level?', choices: ['Either is fine', 'Rooftop', 'Street level'] } },
];
/** A mock of POST /api/chat. With `go`, or after two answers, it settles the brief and searches. */
export async function mockChat(req) {
  await new Promise(r => setTimeout(r, req.instant ? 0 : 700));
  const users = req.messages.filter(m => m.role === 'user').length;
  if (!req.go && users <= ASKS.length) return { ...ASKS[users - 1], brief: null, sheet: null, model: 'mock' };
  const street = /street/i.test(req.messages.at(-1)?.text || '');
  const goals = street ? HERO_GOALS.filter(g => g.key !== 'high').concat({ key: 'street', use: 'street_level', must: true }) : HERO_GOALS;
  return {
    understood: ['Sunset, Sat 15 Nov', 'Plaza fountain in front of Grand Tower', 'Downtown LA (my pick)', street ? 'Street level' : 'Roof or street', '60° lens (my pick)'],
    ask: null, model: 'mock',
    brief: { event: 'sunset', date: '2026-11-15', tz: 'America/Los_Angeles', area: 'Downtown LA', lens_deg: 60, goals },
    sheet: {
      steps: [
        { kind: 'chat', text: 'Settled the brief after 2 questions' },
        { kind: 'find_objects', text: 'Found Plaza fountain (Vexcel detection) 160 m from Grand Tower (CoStar)' },
        { kind: 'define_cost_function', text: 'Wrote “close_to_tower”: tried on 50 points, 62% pass, so it splits them well' },
        { kind: 'search', text: 'Ran 4 goals over 1,024 points' },
        { kind: 'investigate', text: 'Looked at the top 5 in oblique photos: moved one up, flagged one' },
      ],
      candidates: CANDIDATES, // PROPOSED field, see the note at the top
    },
  };
}
