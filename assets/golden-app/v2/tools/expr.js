// The v2 expression language (docs/metrics.md, "Expression language"), evaluated in the browser.
// The server's core does this for real; this copy lets the page re-rank instantly while you drag a weight,
// and it is the reference the mock data is written against. Pure functions, no DOM.

const TIER = { Point: 0, Ray: 1, Skyline: 2, AI: 3 };
// which tier each feature needs (metrics.md, "Features")
export const FEATURE_TIER = {
  ground_m: 'Point', eye_m: 'Point', roof: 'Point', dist: 'Point', bearing: 'Point', bearing_diff: 'Point', size_deg: 'Point',
  visible: 'Ray',
  sun_clear: 'Skyline', sun_between: 'Skyline', sun_behind: 'Skyline', sun_visible_min: 'Skyline',
  sun_hidden_elev: 'Skyline', gap_width_deg: 'Skyline', sun_bearing_diff: 'Skyline',
};

const clamp = v => Math.max(0, Math.min(1, v));

/** Evaluate expression `e` for one candidate. `feat(name, objRefs)` returns the feature's number;
 *  `args` maps a cost function's params ("a", "b") to object refs. */
export function evaluate(e, feat, args = {}) {
  const ev = x => evaluate(x, feat, args);
  const ref = r => (typeof r === 'string' ? args[r] : r);
  if ('num' in e) return e.num;
  if ('f' in e) return feat(e.f, [e.obj, e.a, e.b].filter(Boolean).map(ref));
  const xs = e.of ? e.of.map(ev) : null;
  switch (e.op) {
    case 'ramp': { const x = ev(e.x); return clamp((x - e.lo) / (e.hi - e.lo)); }
    case 'bump': { const x = ev(e.x); return clamp(1 - Math.abs(x - e.at) / e.width); }
    case 'and': case 'min': return Math.min(...xs);
    case 'or': case 'max': return Math.max(...xs);
    case 'mul': return xs.reduce((p, v) => p * v, 1);
    case 'add': return xs.reduce((p, v) => p + v, 0);
    case 'avg': return xs.reduce((p, v) => p + v, 0) / xs.length;
    case 'not': return 1 - clamp(ev(e.x));
    case 'lt': return ev(e.x) < ev(e.than) ? 1 : 0;
    case 'gt': return ev(e.x) > ev(e.than) ? 1 : 0;
  }
  throw Error('unknown expression ' + JSON.stringify(e).slice(0, 60));
}

/** The highest tier among an expression's features: the funnel runs cheap goals first. */
export function tierOf(e) {
  let t = 'Point';
  (function walk(x) {
    if (!x || typeof x !== 'object') return;
    if (x.f && TIER[FEATURE_TIER[x.f]] > TIER[t]) t = FEATURE_TIER[x.f];
    Object.values(x).forEach(v => (Array.isArray(v) ? v.forEach(walk) : walk(v)));
  })(e);
  return t;
}
export const tierRank = t => TIER[t];
