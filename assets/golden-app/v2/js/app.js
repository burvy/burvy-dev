// Frontend v2 (task 057): the chat steers, goals are live filters, the funnel and the map show their effect.
// The server scores every goal at every point (docs/contracts.md#goal-layers-and-stages); this page only
// re-weights, masks and re-picks those numbers (layers.js). Anything that changes a formula, the area, or
// needs exact sun times for a new spot is a new POST /api/search (data.js).
import { createData } from './data.js';
import { rebuild, stagesFor, initialGoals, serverSpotAt, editedGoals, cellLatLon, needsServer } from './layers.js';
import { createMap, RAMP_CSS } from './map.js';

const $ = s => document.querySelector(s);
const esc = v => String(v ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const fmt = n => n.toLocaleString('en-US');
const params = new URLSearchParams(location.search);
// API base: ?api=/golden (burvy.dev), else this origin when golden's server serves the page
const BASE = (() => { const a = params.get('api'); if (a && /^\/(?!\/)/.test(a)) return location.origin + a.replace(/\/$/, '');
  return location.protocol.startsWith('http') ? location.origin : 'http://localhost:8787'; })();
const MODE = { fixture: params.get('data') === 'fixture' };
const data = createData(BASE, MODE);
// the old page lives on at classic/; it takes the same ?api= base (burvy.dev's iframe)
document.getElementById('classic').href = 'classic/' + (params.get('api') ? '?api=' + encodeURIComponent(params.get('api')) : '');
const COLORS = ['--g1', '--g2', '--g3', '--g4', '--g5', '--g6'].map(v => getComputedStyle(document.documentElement).getPropertyValue(v).trim());
const ICON = {
  eye: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/></svg>',
  x: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 6l12 12M18 6L6 18"/></svg>',
  undo: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 10h11a5 5 0 010 10h-3"/><path d="M8 6l-4 4 4 4"/></svg>',
};
const VERDICT = { confirmed: 'Looks right', caution: 'Check this', moved_up: 'Moved up', moved_down: 'Moved down', ruled_out: 'Ruled out' };

const S = { messages: [], busy: false, result: null, goals: [], run: null, stages: [], picks: [], K: 3, sel: 0, layer: 'all', searching: false, sheet: null };

// ---------------------------------------------------------------- words for goals
const objName = (ref, objs) => {
  if (!ref) return 'it';
  const o = ref.id ? objs.find(x => x.id === ref.id) : objs.find(x => x.kind === ref.kind);
  return o?.name || (ref.kind ? ref.kind.replace(/^./, c => c.toUpperCase()) : 'the building');
};
const SEED = { // titles and lines for library entries the server names; anything else is worded from its name
  sun_between: ['Sun in a gap', 'The sun sets in a gap between buildings'], sun_behind: ['Sun behind a building', 'The sun sets right behind a building, for a rim of light'],
  sun_clear: ['Clear view of the sun', 'A clear view of the sun all the way down'], high_vantage: ['Up high', 'Higher up: a roof beats the street'],
  street_level: ['Street level only', 'From the street, no roof access needed'], long_sun_window: ['Long sun window', 'The sun stays in view for at least 20 minutes of golden hour'],
  fit: ['How well it fits', 'The server’s combined score (it doesn’t send per-goal scores yet)'],
};
function goalWords(k) {
  const r = S.result, g = r.brief.goals?.[k] || {}, name = r.layers[k].goal, objs = r.objects || [], a = g.args || {};
  const fromSpot = r.spots.find(s => s.scores?.[k])?.scores[k].why.split(':')[0];
  const about = g.def?.about || SEED[name]?.[1] || fromSpot || '';
  let title;
  switch (name) {
    case 'in_front': title = `${objName(a.a, objs)} in front of ${objName(a.b, objs)}`; break;
    case 'lined_up': title = `${objName(a.a, objs)} lined up with ${objName(a.b, objs)}`; break;
    case 'fills': title = `${objName(a.a, objs)} fills the frame`; break;
    case 'close_to_tower': title = `Close to ${objName({ id: 'costar:52227' }, objs)}`; break;
    default: title = SEED[name]?.[0] || name.replace(/_/g, ' ').replace(/^./, c => c.toUpperCase());
  }
  return { title, about, saved: !g.def, json: g };
}

// ---------------------------------------------------------------- chat (POST /api/chat)
const GREET = 'What do you want to shoot? Tell me what should be in the frame, the light and the day if you know them. I’ll ask about anything that changes where you stand.';
function bubble(role, html) { const d = document.createElement('div'); d.className = `b ${role}`; d.innerHTML = html; $('#log').append(d); $('#log').scrollTop = 1e9; return d; }
const chipHTML = t => / \(my pick\)$/.test(t) ? `<span class="chip pick" title="My pick: say if you want something else">${esc(t.replace(/ \(my pick\)$/, ''))}, my pick</span>` : `<span class="chip">${esc(t)}</span>`;
function header(list) { $('#hbrief').innerHTML = list.map(t => `<span>${esc(t.replace(/ \(my pick\)$/, ''))}</span>`).join(''); }
function choices(list) {
  $('#choices').innerHTML = list.map((t, i) => `<button type="button" class="${i ? '' : 'def'}">${esc(t)}</button>`).join('');
  $('#choices').querySelectorAll('button').forEach((b, i) => (b.onclick = () => send(list[i])));
}
async function send(text, go = false, instant = false) {
  text = (text || '').trim(); if (S.busy || (!text && !go)) return;
  if (text) { S.messages.push({ role: 'user', text }); bubble('user', esc(text)); }
  $('#msg').value = ''; choices([]); S.busy = true;
  const wait = bubble('agent', '<span class="dots" aria-label="Thinking"><i></i><i></i><i></i></span>');
  try {
    const r = await data.chat({ messages: S.messages, go, tz: 'America/Los_Angeles', instant });
    wait.remove(); header(r.understood || []);
    if (r.ask) {
      S.messages.push({ role: 'agent', text: r.ask.question });
      bubble('agent', esc(r.ask.question)); choices(r.ask.choices || []);
      $('#known').innerHTML = (r.understood || []).map(chipHTML).join(''); $('#gonow').hidden = false;
    } else {
      $('#known').innerHTML = ''; $('#gonow').hidden = true;
      const log = document.createElement('ol'); log.className = 'steps';
      log.innerHTML = (r.sheet.steps || []).map(s => `<li><b>${esc(s.tool.replace(/_/g, ' '))}</b>: ${esc(s.summary)}</li>`).join(''); $('#log').append(log);
      S.sheet = r.sheet;
      applyResult(r.result, { fit: true });
      if (matchMedia('(max-width:980px)').matches && !instant) $('#funnel').scrollIntoView({ behavior: 'smooth', block: 'start' });
      const musts = S.goals.filter(g => g.must).length, n = S.goals.length;
      const said = `I set ${n} goal${n === 1 ? '' : 's'}${musts ? `: ${musts} that every spot must meet, the rest to rank by` : ''}. Change them ${matchMedia('(max-width:980px)').matches ? 'below the map' : 'on the left'} and the map updates at once; drag the map and press “Search this area” if the place isn’t quite right.`;
      bubble('agent', esc(r.note ? `${r.note} ${said}` : said));
      S.messages.push({ role: 'agent', text: said });
    }
  } catch (e) { wait.remove(); bubble('agent', `That didn’t work: ${esc(e.message)}. Try again, or press “Search now”.`); $('#gonow').hidden = false; }
  S.busy = false;
}
function newBrief() { S.messages = []; $('#log').innerHTML = ''; bubble('agent', esc(GREET)); choices([]); $('#known').innerHTML = ''; }

// ---------------------------------------------------------------- result -> goals -> run
function applyResult(result, { fit = false, keepEdits = null } = {}) {
  S.result = result; S.goals = initialGoals(result);
  if (keepEdits) S.goals.forEach((g, k) => Object.assign(g, keepEdits[k] || {}));
  S.layer = 'all'; S.sel = 0;
  rerun();
  if (fit) map.fit(result.brief.area);
  $('#mapempty').hidden = true; $('#addgoal').hidden = false;
}
let tmr = 0;
function rerun({ soon = false } = {}) {
  if (soon) { clearTimeout(tmr); tmr = setTimeout(() => rerun(), 60); return; }
  const prev = S.picks[S.sel]?.i, r = S.result;
  S.run = rebuild(r, S.goals, Math.max(10, S.K));
  S.stages = stagesFor(r, S.goals, S.run);
  S.picks = S.run.picks.slice(0, S.K).map(i => ({ i, ...cellLatLon(r.heat, i), spot: serverSpotAt(r, i) }));
  for (const p of S.picks) if (p.spot) { p.lat = p.spot.lat; p.lon = p.spot.lon; }
  const keep = S.picks.findIndex(p => p.i === prev); S.sel = keep >= 0 ? keep : 0;
  if (S.layer !== 'all' && S.goals[S.layer]?.removed) S.layer = 'all';
  renderGoals(); renderFunnel(); renderLayers(); renderSpot(); drawMap();
}
const freshPicks = () => S.picks.filter(p => !p.spot).length;
const edited = () => S.goals.some((g, k) => g.removed || g.must !== S.result.layers[k].must || Math.abs(g.weight - S.result.layers[k].weight) > 1e-9);

async function searchAgain(brief, why, opts = {}) {
  if (S.searching) return;
  S.searching = true; setBusy(why);
  try {
    const res = await data.search(brief);
    applyResult(res, opts); toast('');
  } catch (e) { toast(e.message, true); }
  S.searching = false; setBusy(null);
}
function setBusy(why) {
  $('#busy').hidden = !why; $('#busy').textContent = why || '';
  document.querySelectorAll('[data-needs-server]').forEach(b => (b.disabled = !!why));
}
function toast(msg, err = false) { const t = $('#toast'); t.hidden = !msg; t.textContent = msg; t.classList.toggle('err', err); }
const editedBrief = extra => ({ ...S.result.brief, goals: editedGoals(S.result.brief, S.goals), ...extra });
const checkNew = () => searchAgain(editedBrief(), 'Getting exact sun times for the new spots…');

// ---------------------------------------------------------------- goals panel
function renderGoals() {
  const r = S.result, total = r.stages.find(s => s.goal == null && /stand/i.test(s.label))?.n ?? r.searched;
  const row = k => {
    const g = S.goals[k], L = r.layers[k], w = goalWords(k), st = S.stages.findIndex(s => s.k === k);
    const before = st > 0 ? S.stages[st - 1].n : total, after = st > 0 ? S.stages[st].n : total;
    const own = L.gateSet?.size || (L.gate || []).length;
    const meta = st > 0 ? `<span>Drops ${fmt(before - after)} of ${fmt(before)} points</span>` : `<span>Ranks every point, weight ${g.weight.toFixed(1)}</span>`;
    return `<li class="goal ${g.must ? 'must' : ''}" style="--c:${COLORS[k % COLORS.length]}" data-k="${k}">
      <div class="ghead"><h4>${esc(w.title)}</h4>
        <button class="ib" data-a="layer" aria-pressed="${S.layer === k}" aria-label="Show ${esc(w.title)} on the map" title="Show on the map">${ICON.eye}</button>
        <button class="ib" data-a="rm" aria-label="Remove ${esc(w.title)}" title="Remove">${ICON.x}</button></div>
      <p>${esc(w.about)}</p>
      <div class="ctl">
        <label class="sw"><input type="checkbox" data-a="must" ${g.must ? 'checked' : ''}> Must</label>
        ${g.must ? '' : `<label class="wt"><span class="sr">Weight</span><input type="range" min="0" max="2" step="0.1" value="${g.weight}" data-a="w" aria-label="Weight for ${esc(w.title)}"><output>${g.weight.toFixed(1)}</output></label>`}
      </div>
      <div class="gmeta">${meta}${!g.must && own ? `<span>Its formula still needs its own condition (${fmt(own)} points out)</span>` : ''}
        ${w.saved ? '' : '<span class="new">Written for this brief</span>'}</div>
      ${!r.brief.goals?.[k] ? '' : `<details><summary>How it’s scored</summary>
        <label class="sr" for="f${k}">Formula</label><textarea id="f${k}" class="formula" spellcheck="false">${esc(JSON.stringify(w.json, null, 1))}</textarea>
        <div class="frow"><button type="button" data-a="run" data-needs-server>Search with this formula</button><span class="ferr" role="status"></span></div></details>`}
    </li>`;
  };
  const live = S.goals.map((g, k) => k).filter(k => !S.goals[k].removed);
  const musts = live.filter(k => S.goals[k].must), soft = live.filter(k => !S.goals[k].must), gone = S.goals.map((g, k) => k).filter(k => S.goals[k].removed);
  $('#glists').innerHTML = `
    <div class="group"><h3>Every spot must <small>${musts.length ? 'cheapest check first' : ''}</small></h3>
      ${musts.length ? `<ul class="glist">${musts.map(row).join('')}</ul>` : '<div class="gempty">No musts: every point stays in, ranked by the goals below.</div>'}</div>
    <div class="group"><h3>Then rank by <small>${soft.length ? 'heavier counts more' : ''}</small></h3>
      ${soft.length ? `<ul class="glist">${soft.map(row).join('')}</ul>` : '<div class="gempty">Nothing to rank by, so every goal counts the same.</div>'}</div>
    ${gone.length ? `<div class="group"><h3>Removed</h3><ul class="glist">${gone.map(k => `<li class="goal gone" data-k="${k}"><div class="ghead"><h4>${esc(goalWords(k).title)}</h4><button class="ib" data-a="undo" aria-label="Put back ${esc(goalWords(k).title)}" title="Put back">${ICON.undo}</button></div></li>`).join('')}</ul></div>` : ''}
    ${edited() ? '<button type="button" class="reset" id="resetgoals">Back to my goals as searched</button>' : ''}`;
  $('#glists').querySelectorAll('.goal').forEach(li => {
    const k = +li.dataset.k, g = S.goals[k], on = (a, f) => { const el = li.querySelector(`[data-a=${a}]`); if (el) f(el); };
    on('undo', b => (b.onclick = () => { g.removed = false; edit(); }));
    on('must', b => (b.onchange = e => { g.must = e.target.checked; if (g.must) { g.prev = g.weight; g.weight = 0; } else g.weight = g.prev || 1; edit(); }));
    on('w', b => (b.oninput = e => { g.weight = +e.target.value; li.querySelector('output').textContent = g.weight.toFixed(1); flash(); rerun({ soon: true }); }));
    on('rm', b => (b.onclick = () => { g.removed = true; edit(); }));
    on('layer', b => (b.onclick = () => { S.layer = S.layer === k ? 'all' : k; renderGoals(); renderLayers(); drawMap(); }));
    on('run', b => (b.onclick = () => {
      let goal; try { goal = JSON.parse(li.querySelector('textarea').value); } catch (e) { li.querySelector('.ferr').textContent = `That isn’t valid JSON: ${e.message}`; return; }
      const goals = S.result.brief.goals.map((x, j) => (j === k ? goal : x)).filter((_, j) => !S.goals[j].removed || j === k);
      searchAgain({ ...S.result.brief, goals }, 'Searching with the new formula…');
    }));
  });
  const rs = $('#resetgoals'); if (rs) rs.onclick = () => { S.goals = initialGoals(S.result); flash(); rerun(); };
  setBusy(S.searching ? $('#busy').textContent : null);
}

// A goal edit: instant from the layers, unless it lets dropped points back in (their scores were never
// computed, core stops at the first gate): then it's a new search with the goals as edited.
function edit() {
  flash();
  if (!needsServer(S.result, S.goals)) return rerun();
  if (data.fixture) { // no server to ask: undo just the edits that would let points back in
    const start = initialGoals(S.result);
    S.goals = S.goals.map((g, k) => (needsServer(S.result, start.map((x, j) => (j === k ? g : x))) ? start[k] : g));
    toast('Letting dropped points back in needs golden’s server (it never scored them); this page is showing saved example data.', true);
    return rerun();
  }
  searchAgain(editedBrief(), 'Letting those points back in: searching again…');
}

// ---------------------------------------------------------------- the funnel: a band of light, narrowing
function renderFunnel() {
  const st = [...S.stages, { label: `Top ${S.K}`, n: S.picks.length, top: true }], n0 = st[0].n || 1, N = st.length, el = $('#funnel');
  const W = 1000, H = 84, w = W / N, h = n => 22 + 58 * Math.sqrt(n / n0);
  const pts = st.map((s, i) => [i * w + w / 2, h(s.n)]);
  const top = [[0, (H - pts[0][1]) / 2], ...pts.map(([x, hh]) => [x, (H - hh) / 2]), [W, (H - pts[N - 1][1]) / 2]];
  let d = `M${top[0][0]},${top[0][1]}`;
  for (let i = 1; i < top.length; i++) { const [x0, y0] = top[i - 1], [x1, y1] = top[i], mx = (x0 + x1) / 2; d += ` C${mx},${y0} ${mx},${y1} ${x1},${y1}`; }
  const bot = top.map(([x, y]) => [x, H - y]).reverse();
  d += ` L${bot[0][0]},${bot[0][1]}`;
  for (let i = 1; i < bot.length; i++) { const [x0, y0] = bot[i - 1], [x1, y1] = bot[i], mx = (x0 + x1) / 2; d += ` C${mx},${y0} ${mx},${y1} ${x1},${y1}`; }
  const looked = S.picks.filter(p => p.spot?.investigation).length, fresh = freshPicks();
  const area = S.result.brief.area;
  el.style.setProperty('--n', N);
  el.innerHTML = `<svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="none" aria-hidden="true">
      <defs><linearGradient id="fg" x1="0" x2="1"><stop offset="0" stop-color="#d3e8f8"/><stop offset=".45" stop-color="#fbd7a6"/><stop offset=".8" stop-color="#f9a03f"/><stop offset="1" stop-color="#f26b1d"/></linearGradient></defs>
      <path d="${d} Z" fill="url(#fg)"/></svg>
    <div class="fsteps">${st.map((s, i) => `<div class="fs ${s.top ? 'last' : ''}"><span class="num">${s.top ? `Top ${s.n}` : fmt(s.n)}</span></div>`).join('')}</div>
    <div class="fcap">${st.map((s, i) => {
      if (s.top) return `<div>${fresh ? `${fresh} new since your edit` : looked ? `${looked} of ${s.n} checked by the AI in photos` : 'Best spots, spaced apart'}${fresh ? `<br><button class="check" id="checknew" data-needs-server>Check the new ones</button>` : ''}</div>`;
      if (s.goal == null) return `<div>${esc(s.label)}${i === 0 ? `<span class="wide">, ${esc(area.name || `${area.radius_m} m around the pin`)}</span>` : ''}</div>`;
      return `<div><b style="--c:${COLORS[s.k % COLORS.length]}"></b>${esc(s.label || goalWords(s.k).title)}</div>`;
    }).join('')}</div>`;
  const ck = $('#checknew'); if (ck) ck.onclick = checkNew;
}
function flash() { const f = $('#funnel'); f.classList.remove('flash'); void f.offsetWidth; f.classList.add('flash'); }

// ---------------------------------------------------------------- map, layers and legend
const map = createMap($('#map'), {
  base: BASE,
  onPick: n => { S.sel = n; renderSpot(); drawMap(); },
  onMove: prop => { $('#searcharea').hidden = !prop || !S.result; },
});
$('#searcharea').onclick = () => { const a = map.proposal(); if (a) searchAgain(editedBrief({ area: { lat: a.lat, lon: a.lon, radius_m: a.radius_m } }), `Searching ${a.radius_m} m around the middle of the map…`, { keepEdits: null }); };
$('#zin').onclick = () => map.zoom(0.5); $('#zout').onclick = () => map.zoom(-0.5); $('#zfit').onclick = () => map.fitArea();
function renderLayers() {
  const live = S.goals.map((g, k) => k).filter(k => !S.goals[k].removed);
  $('#layers').innerHTML = [`<button data-l="all" aria-pressed="${S.layer === 'all'}"><i style="--c:conic-gradient(#f9a03f,#f26b1d,#d6480c,#f9a03f)"></i>All goals</button>`,
    ...(live.length > 1 ? live.map(k => `<button data-l="${k}" aria-pressed="${S.layer === k}"><i style="--c:${COLORS[k % COLORS.length]}"></i>${esc(goalWords(k).title)}</button>`) : [])].join('');
  $('#layers').querySelectorAll('button').forEach(b => (b.onclick = () => { S.layer = b.dataset.l === 'all' ? 'all' : +b.dataset.l; renderLayers(); renderGoals(); drawMap(); }));
  const k = S.layer, spacing = Math.round(S.result.spacing_m ?? 40);
  const pins = `<div class="pins"><span class="pin">1</span><span class="pin">2</span><span class="pin">3</span> The top ${S.K} spots, at least ${spacing} m apart
    <button type="button" id="moreK">${S.K === 3 ? 'Show top 10' : 'Show top 3'}</button></div>${matchMedia('(pointer:coarse)').matches ? '<div class="mute">Two fingers move and zoom the map</div>' : ''}`;
  $('#legend').hidden = false;
  $('#legend').innerHTML = (k === 'all'
    ? `<div class="lrow"><span class="ramp" style="background:${RAMP_CSS}"></span><span>Colour: how well <b>every</b> point fits${S.run.droppedBy.some(x => x >= 0) ? '. Shaded: dropped by a must' : ''}</span></div>`
    : `<div class="lrow"><span class="ramp" style="background:linear-gradient(90deg,transparent,${COLORS[k % COLORS.length]})"></span><span>${esc(goalWords(k).title)} at every point${S.goals[k].must || S.result.layers[k].gate?.length ? '. Shaded: dropped by it' : ''}</span></div>`) + pins;
  $('#moreK').onclick = () => { S.K = S.K === 3 ? 10 : 3; rerun(); };
}
function drawMap() {
  if (!S.result) return map.draw();
  map.set({ result: S.result, run: S.run, layer: S.layer, colors: COLORS, picks: S.picks, sel: S.sel, fresh: new Set(S.picks.map((p, n) => (p.spot ? -1 : n))) });
  $('#attrib').textContent = map.attribution();
}

// ---------------------------------------------------------------- spot card
const clock = iso => { const m = /T(\d\d):(\d\d)/.exec(iso || ''); if (!m) return ''; const h = +m[1]; return `${h % 12 || 12}:${m[2]} ${h < 12 ? 'AM' : 'PM'}`; };
let mediaTab = 'photo';
function renderSpot() {
  const el = $('#spot'), p = S.picks[S.sel];
  el.hidden = false;
  if (!p) { el.innerHTML = '<div class="sp-head"><div class="sp-title"><h2>No spot passes every must</h2><p>Untick a must, or remove one, to let points back in.</p></div></div>'; return; }
  const sp = p.spot, r = S.result, inv = sp?.investigation;
  const roofUp = sp && sp.standing === 'roof' ? (sp.why.find(w => /^Roof \d+ m/.test(w)) || 'On a roof') : null;
  const sun = sp?.sun, when = sun ? [sun.hidden_at ? `sun behind the skyline at ${clock(sun.hidden_at)}` : 'sun clear to the horizon', sun.flat_sunset ? `sunset ${clock(sun.flat_sunset)}` : ''].filter(Boolean).join(', ') : '';
  const fit = Math.round((S.run.heat[p.i] ?? 0) * 100);
  const live = S.goals.map((g, k) => k).filter(k => !S.goals[k].removed);
  el.innerHTML = `
    <div class="sp-head">
      <div class="sp-rank" role="tablist" aria-label="Top spots">${S.picks.map((q, n) => `<button role="tab" aria-pressed="${n === S.sel}" aria-label="Spot ${n + 1}${q.spot ? '' : ', new'}" class="${q.spot ? '' : 'new'}">${n + 1}</button>`).join('')}</div>
      <div class="sp-title"><h2>${esc(sp ? (roofUp || 'At street level') : 'New after your edit')}</h2>
        <p>${p.lat.toFixed(5)}, ${p.lon.toFixed(5)}${when ? `. ${esc(when.replace(/^./, c => c.toUpperCase()))}, ${sun.golden_min_visible} min of golden hour in view.` : '.'}</p></div>
      <div class="sp-score"><span class="num">${fit}</span><br><small>fit, of 100</small></div>
    </div>
    <div class="sp-body">
      <div class="sp-why">
        <ul class="reasons">${live.map(k => { const g = S.goals[k], v = r.layers[k].heat.values[p.i] ?? 0, why = sp?.scores?.[k]?.why; const w = goalWords(k); return `
          <li style="--c:${COLORS[k % COLORS.length]}"><span class="gname"><i></i>${esc(w.title)}${g.must ? ' <em>must, passes</em>' : ` <em>weight ${g.weight.toFixed(1)}</em>`}</span>
          <span class="bar"><b style="width:${Math.round(v * 100)}%"></b></span><span class="v">${Math.round(v * 100)}</span>
          <span class="w">${esc(why ? why.replace(/^[^:]*:\s*/, '') : w.about)}</span></li>`; }).join('')}</ul>
        <div class="ai">${inv
          ? `<h3>What the AI saw in the photos <span class="verdict ${inv.verdict}">${esc(VERDICT[inv.verdict] || inv.verdict)}</span></h3><p>${esc(inv.note)}</p>`
          : sp ? `<h3>Not looked at by the AI <span class="verdict none">numbers only</span></h3><p>The scores come from the height map. The AI looks at the top few in aerial photos when it plans the shoot.</p>`
          : `<h3>New after your edit <span class="verdict none">no sun times yet</span></h3><p>This point moved into the top ${S.K} when you changed a goal. Its goal scores above are the server’s; search again to get its exact sun times and reasons.</p><button type="button" id="checkone" data-needs-server>Get its sun times</button>`}</div>
      </div>
      <div class="sp-media">
        <div class="sp-tabs" role="tablist"><button role="tab" data-t="photo" aria-selected="${mediaTab === 'photo'}">Photo</button><button role="tab" data-t="3d" aria-selected="${mediaTab === '3d'}">3D view</button></div>
        <div class="view" id="view"></div>
        <ul class="calls">${sp?.building_id ? `<li><span>On building <b>${esc(sp.building_id)}</b></span><span class="mute">roof access needed</span></li>` : ''}
          <li><span class="mute">${esc(r.brief.date)}, ${r.brief.event === 'sunrise' ? 'sunrise' : 'sunset'}. <a href="https://www.google.com/maps/dir/?api=1&amp;destination=${+p.lat},${+p.lon}" target="_blank" rel="noopener">Directions</a></span></li></ul>
      </div>
    </div>`;
  el.querySelectorAll('.sp-rank button').forEach((b, n) => (b.onclick = () => { S.sel = n; renderSpot(); drawMap(); }));
  el.querySelectorAll('.sp-tabs button').forEach(b => (b.onclick = () => { mediaTab = b.dataset.t; renderSpot(); }));
  const one = $('#checkone'); if (one) one.onclick = checkNew;
  setBusy(S.searching ? $('#busy').textContent : null);
  media(p);
}
let view3d = null;
async function media(p) {
  const v = $('#view'), az = p.spot?.sun?.hidden_az_deg ?? (S.result.brief.event === 'sunrise' ? 115 : 246);
  if (view3d) { try { view3d.dispose?.(); } catch (_) {} view3d = null; }
  if (mediaTab === 'photo') {
    v.innerHTML = '<div class="ph">Loading the aerial photo…</div>';
    try {
      const r = await fetch(`${BASE}/api/photo?lat=${p.lat}&lon=${p.lon}&look_az_deg=${Math.round(az)}&radius_m=150&max_px=1024`);
      if (!r.ok) throw 0; const m = await r.json();
      if (S.picks[S.sel] !== p || mediaTab !== 'photo') return;
      v.innerHTML = `<img alt="Aerial photo looking toward the ${S.result.brief.event} from spot ${S.sel + 1}" src="${esc(BASE + m.image_url)}">`;
    } catch (_) { if (S.picks[S.sel] === p) v.innerHTML = '<div class="ph">No aerial photo here: golden’s server isn’t reachable, or has none cached for this spot.</div>'; }
  } else {
    v.innerHTML = '<div class="ph">Loading the 3D view…</div>';
    try {
      const at = await fetch(`${BASE}/api/mesh/at?lat=${p.lat}&lon=${p.lon}`).then(r => (r.ok ? r.json() : Promise.reject()));
      const m = await import('../../js/meshview.js');
      if (S.picks[S.sel] !== p || mediaTab !== '3d') return;
      v.innerHTML = '';
      view3d = await m.create({ host: v, base: BASE, lat: p.lat, lon: p.lon, yaw: az, tilesetUrl: at.tileset_url, attribution: at.attribution, onStatus: () => {} });
    } catch (_) { if (S.picks[S.sel] === p) v.innerHTML = '<div class="ph">No 3D view here: golden’s server isn’t reachable, or has no mesh for this spot.</div>'; }
  }
}

// ---------------------------------------------------------------- start
$('#say').onsubmit = e => { e.preventDefault(); send($('#msg').value); };
$('#msg').addEventListener('keydown', e => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send($('#msg').value); } });
$('#gonow').onclick = () => send('', true);
$('#newbrief').onclick = newBrief;
$('#addgoal').onclick = () => { const m = $('#msg'); m.placeholder = 'e.g. “also rank by a long sun window”'; m.focus(); m.scrollIntoView({ block: 'center', behavior: 'smooth' }); };
newBrief();
$('#glists').innerHTML = '<div class="gempty">Once the brief is clear, the goals I chose show up here, and you can change any of them.</div>';
map.draw();
$('#datamode').textContent = MODE.fixture ? 'Example data' : '';
fetch(`${BASE}/api/health`, { signal: AbortSignal.timeout(2500) }).then(r => { if (!r.ok) throw 0; }).catch(() => {
  map.noTiles();
  if (!MODE.fixture) { MODE.fixture = true; $('#datamode').textContent = 'Example data (server not reachable)'; }
});
// ?demo=chat|results|edited|moved: jump to a state, for screenshots
const demo = params.get('demo');
(async () => {
  if (!demo) return;
  if (demo === 'live') return send(params.get('q') || 'fountain in front of a tower at sunset downtown LA', true, true);
  await send('fountain in front of a tower at sunset', false, true);
  if (demo === 'chat') return;
  await send('Grand Tower, 48 floors', false, true); await send('Either is fine', false, true);
  if (demo === 'edited') { // drop "Up high" and make the sun count double: new spots come in, without sun times yet
    const k = S.result.layers.findIndex(L => L.goal === 'high_vantage'), s = S.result.layers.findIndex(L => L.goal === 'sun_between');
    if (k >= 0) S.goals[k].removed = true; if (s >= 0) S.goals[s].weight = 2; flash(); rerun();
    const n = S.picks.findIndex(p => !p.spot); if (n >= 0) { S.sel = n; renderSpot(); drawMap(); }
  }
  if (demo === 'top10') { S.K = 10; rerun(); }
  if (demo === 'moved') { const a = S.result.brief.area; map.centreOn(a.lat + 0.004, a.lon - 0.003); }
})();
