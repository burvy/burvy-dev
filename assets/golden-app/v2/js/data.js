// Where the page's data comes from. Live: golden's server (POST /api/chat, POST /api/search).
// Fixture (?data=fixture, or when the server isn't reachable): fixtures/search_layers.json behind a
// two-question scripted chat, so the page can be shown with no server at all.
// Until the server sends goal layers (task 059), a live result gets one layer made from its heat
// (layers.js withLayers) and a chat sheet without `search` costs one extra POST /api/search.
import { withLayers } from './layers.js';

export function createData(base, mode) {
  let fixture = null;
  const loadFixture = async () => (fixture ??= await fetch(new URL('../fixtures/search_layers.json', import.meta.url)).then(r => r.json()));

  async function post(path, body, ms = 300e3) {
    const r = await fetch(base + path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body), signal: AbortSignal.timeout(ms) });
    const text = await r.text();
    let j = null; try { j = JSON.parse(text); } catch (_) { /* not JSON */ }
    if (!r.ok) throw new Error(j?.error || `${path} answered ${r.status}`);
    return j;
  }

  /** POST /api/chat. Returns the ChatReply, with `result` (a SearchResult with layers) once it has searched. */
  async function chat(req) {
    if (mode.fixture) return fixtureChat(req);
    const { instant, ...body } = req; // instant is the page's own (demo) flag; the server rejects unknown fields
    const reply = await post('/api/chat', body);
    if (reply.sheet) {
      const res = reply.sheet.search || await post('/api/search', reply.sheet.brief);
      reply.result = withLayers(res);
    }
    return reply;
  }

  /** POST /api/search with a brief (edited goals, a new area). */
  async function search(brief) {
    if (mode.fixture) throw new Error('Searching again needs golden’s server; this page is showing saved example data.');
    return withLayers(await post('/api/search', brief));
  }

  // ---- the scripted chat for fixture mode (same questions as the approved mock)
  const ASKS = [
    { understood: ['Sunset', 'A fountain in front of a tower', 'Downtown LA (my pick)'],
      ask: { question: 'Which tower? There are two tall ones within 300 m of a fountain downtown.', choices: ['Grand Tower, 48 floors', 'Hope Place, 31 floors', 'Either one'] } },
    { understood: ['Sunset', 'Plaza fountain in front of Grand Tower', 'Downtown LA (my pick)'],
      ask: { question: 'Rooftop or street level?', choices: ['Either is fine', 'Rooftop', 'Street level'] } },
  ];
  async function fixtureChat(req) {
    await new Promise(r => setTimeout(r, req.instant ? 0 : 600));
    const users = req.messages.filter(m => m.role === 'user').length;
    if (!req.go && users <= ASKS.length) return { ...ASKS[users - 1], model: 'example' };
    const res = withLayers(structuredClone(await loadFixture()));
    return {
      understood: ['Sunset, Sat 14 Nov', 'Plaza fountain in front of Grand Tower', 'Downtown LA (my pick)', 'Roof or street', '60° lens (my pick)'],
      model: 'example', brief: res.brief, result: res,
      sheet: { title: 'Fountain and Grand Tower at sunset', brief: res.brief, steps: [
        { tool: 'chat', summary: 'Settled the brief after 2 questions' },
        { tool: 'find_objects', summary: 'Found the Plaza fountain (Vexcel detection) 160 m from Grand Tower (CoStar)' },
        { tool: 'define_cost_function', summary: 'Wrote “close_to_tower”: tried on 50 points, 62% pass, so it splits them well' },
        { tool: 'search', summary: `Ran ${res.layers.length} goals over ${res.stages[0].n.toLocaleString('en-US')} points` },
        { tool: 'investigate', summary: 'Looked at the top 5 in oblique photos: flagged two' },
      ], spots: [] },
    };
  }

  return { chat, search, get fixture() { return !!mode.fixture; } };
}
