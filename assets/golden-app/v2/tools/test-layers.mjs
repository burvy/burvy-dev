// node web/v2/tools/test-layers.mjs [result.json]: the page's rebuild gives back the server's heat and spots
// for the searched weights (docs/contracts.md#goal-layers-and-stages), and edits behave.
// (a saved result needs 4 goals: goal 1 a must that gates, goals 2 and 3 soft, like the fixture)
import { readFileSync } from 'node:fs';
import { rebuild, initialGoals, stagesFor, serverSpotAt, editedGoals, needsServer } from '../js/layers.js';
const file = process.argv[2] || new URL('../fixtures/search_layers.json', import.meta.url);
const r = JSON.parse(readFileSync(file, 'utf8'));
let bad = 0; const ok = (c, m) => { if (!c) { bad++; console.error('FAIL', m); } };
const g = initialGoals(r), run = rebuild(r, g, r.brief.max_spots);
ok(run.heat.every((v, i) => (v == null) === (r.heat.values[i] == null) && (v == null || Math.abs(v - r.heat.values[i]) < 1e-3)), 'heat matches');
ok(run.picks.length === r.spots.length, `picked ${run.picks.length} of ${r.spots.length}`);
run.picks.forEach((i, n) => ok(serverSpotAt(r, i)?.rank === n + 1, `pick ${n + 1} is server spot ${n + 1}`));
const st = stagesFor(r, g, run);
ok(JSON.stringify(st.map(s => s.n)) === JSON.stringify(r.stages.map(s => s.n)), `stages ${st.map(s => s.n)} vs ${r.stages.map(s => s.n)}`);
// weights and must-on stay in the browser; must-off or removing a gating goal goes back to the server
const g2 = initialGoals(r); g2[1].must = false; g2[1].weight = 1;
ok(needsServer(r, g2), 'a must off needs the server');
const g4 = initialGoals(r); g4[1].removed = true;
ok(needsServer(r, g4), 'removing a gating goal needs the server');
const g5 = initialGoals(r); g5[2].must = true; g5[2].weight = 0;
ok(!needsServer(r, g5) && stagesFor(r, g5, rebuild(r, g5, 10)).at(-1).n <= st.at(-1).n, 'a new must is local and only narrows');
const g6 = initialGoals(r); g6[2].weight = 2; g6[3].weight = 0;
ok(!needsServer(r, g6) && rebuild(r, g6, 3).picks.every(i => r.layers.every(L => L.heat.values[i] != null)), 're-weighting is local and only picks fully scored points');
const g3 = initialGoals(r); g3[3].removed = true;
ok(!needsServer(r, g3), 'removing a soft goal is local');
const p3 = rebuild(r, g3, 3).picks; ok(p3.join() !== run.picks.slice(0, 3).join(), 'removing a goal re-ranks');
ok(editedGoals(r.brief, g3).length === r.brief.goals.length - 1 && !!editedGoals(r.brief, g3)[0].must === !!r.layers[0].must, 'edited goals back to brief JSON');
console.log(bad ? `${bad} failed` : 'layers: all ok');
process.exit(bad ? 1 : 0);
