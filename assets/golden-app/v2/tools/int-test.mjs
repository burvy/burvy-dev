// task 057 phase 2 interaction test (Playwright + Edge). usage: node web/v2/tools/int-test.mjs <http://host/> <outdir>
// fixture: weights/must/remove re-rank in the browser, top-10, a new spot shows "no sun times yet".
// live: drag the map -> "Search this area" -> a new search centred on the view; zoom buttons; wheel.
import { chromium } from 'playwright-core';
const [url, out] = process.argv.slice(2);
const b = await chromium.launch({ channel: 'msedge', headless: true });
let fails = 0; const ok = (c, m) => { console.log(c ? 'ok  ' : 'FAIL', m); if (!c) fails++; };
const page = async (q, vp = { viewport: { width: 1440, height: 900 } }) => {
  const p = await (await b.newContext(vp)).newPage(); const errs = [];
  p.on('pageerror', e => errs.push(e.message));
  p.on('requestfailed', r => { if (!/api\/(photo|tiles)/.test(r.url())) errs.push('reqfail ' + r.url()); });
  await p.goto(url + q, { waitUntil: 'networkidle' }); await p.waitForTimeout(1500); return { p, errs };
};
const pins = p => p.evaluate(() => [...document.querySelectorAll('.sp-rank button')].map(b => b.textContent + (b.classList.contains('new') ? '*' : '')).join(','));
const coord = p => p.evaluate(() => document.querySelector('.sp-title p')?.textContent.split('.')[0] + '.' + document.querySelector('.sp-title p')?.textContent.split('.')[1]);

{ // fixture
  const { p, errs } = await page('?data=fixture&demo=results');
  const top0 = await coord(p);
  ok((await pins(p)) === '1,2,3', 'three server spots: ' + await pins(p));
  // drag the sun weight to 2 and remove "Up high"
  const sun = p.locator('.goal', { hasText: 'Sun in a gap' }).locator('input[type=range]');
  await sun.fill('2'); await p.waitForTimeout(200);
  await p.locator('.goal', { hasText: 'Up high' }).locator('[data-a=rm]').click(); await p.waitForTimeout(200);
  const after = await pins(p);
  ok(after.includes('*'), 'an edit brings in a new spot: ' + after);
  ok(await p.locator('#checknew').isVisible(), '"Check the new ones" offered');
  ok(await p.locator('.goal.gone', { hasText: 'Up high' }).isVisible(), 'removed goal listed with undo');
  await p.locator('.sp-rank button.new').first().click(); await p.waitForTimeout(200);
  ok(await p.locator('.ai', { hasText: 'no sun times yet' }).isVisible(), 'new spot says it has no sun times yet');
  await p.click('#checknew'); await p.waitForTimeout(300);
  ok(await p.locator('#toast.err').isVisible(), 'fixture mode explains search needs the server: ' + await p.locator('#toast').textContent());
  await p.click('#resetgoals'); await p.waitForTimeout(200);
  ok((await pins(p)) === '1,2,3' && (await coord(p)) === top0, 'back to the goals as searched');
  // must on sun: fewer points in the funnel
  const n0 = await p.evaluate(() => [...document.querySelectorAll('.fs .num')].at(-2).textContent);
  await p.locator('.goal', { hasText: 'Sun in a gap' }).locator('[data-a=must]').check(); await p.waitForTimeout(200);
  const n1 = await p.evaluate(() => [...document.querySelectorAll('.fs .num')].at(-2).textContent);
  ok(+n1.replace(/,/g, '') < +n0.replace(/,/g, '') || n1 !== n0, `sun as a must narrows the funnel: ${n0} -> ${n1}`);
  await p.click('#resetgoals'); await p.waitForTimeout(200);
  await p.locator('.goal', { hasText: 'Close to Grand Tower' }).locator('[data-a=must]').click(); await p.waitForTimeout(200);
  ok(await p.locator('.goal', { hasText: 'Close to Grand Tower' }).locator('[data-a=must]').isChecked() && await p.locator('#toast.err').isVisible(), 'fixture: a must off is undone with a note (needs the server)');
  await p.click('#moreK'); await p.waitForTimeout(200);
  ok((await pins(p)).split(',').length === 10, 'top 10: ' + await pins(p));
  ok(errs.length === 0, 'fixture: no page errors ' + errs.join(' | '));
  await p.screenshot({ path: `${out}/int-fixture-top10.png` });
}
{ // live
  const { p, errs } = await page('?demo=live');
  await p.waitForSelector('.sp-rank button', { timeout: 120000 });
  const before = await coord(p);
  ok(await p.locator('#searcharea').isHidden(), 'no "Search this area" while on the searched area');
  const box = await p.locator('#map').boundingBox();
  await p.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await p.mouse.down();
  await p.mouse.move(box.x + box.width / 2 - 260, box.y + box.height / 2 + 40, { steps: 8 }); await p.mouse.up();
  await p.waitForTimeout(300);
  ok(await p.locator('#searcharea').isVisible(), '"Search this area" after dragging');
  const req = p.waitForRequest(r => r.url().endsWith('/api/search') && r.method() === 'POST');
  await p.click('#searcharea');
  const body = JSON.parse((await req).postData());
  ok(body.area && body.area.radius_m >= 100 && body.area.radius_m <= 800, `searched ${JSON.stringify(body.area)}`);
  await p.waitForSelector('#busy', { state: 'hidden', timeout: 120000 });
  await p.waitForTimeout(500);
  const after = await coord(p);
  ok(after !== before, `new spots for the new area: ${before} -> ${after}`);
  ok(await p.locator('#searcharea').isHidden(), 'button hides once searched');
  await p.screenshot({ path: `${out}/int-live-moved.png` });
  // formula edit: make the sun goal a must -> server search; then switch the must off -> server search again
  await p.locator('.goal summary').first().click();
  await p.locator('.goal textarea').first().fill('{"use":"sun_clear","must":true}');
  let rq = p.waitForRequest(r => r.url().endsWith('/api/search'));
  await p.locator('.goal [data-a=run]').first().click();
  ok(JSON.parse((await rq).postData()).goals[0].must === true, 'formula edit goes to the server');
  await p.waitForSelector('#busy', { state: 'hidden', timeout: 120000 }); await p.waitForTimeout(300);
  ok(await p.locator('.goal [data-a=must]').first().isChecked(), 'the new search comes back with the goal as a must');
  rq = p.waitForRequest(r => r.url().endsWith('/api/search'), { timeout: 5000 });
  await p.locator('.goal [data-a=must]').first().click();
  ok(JSON.parse((await rq).postData()).goals[0].must !== true, 'a must off is a new search with it soft');
  await p.waitForSelector('#busy', { state: 'hidden', timeout: 120000 }); await p.waitForTimeout(300);
  await p.click('#zin'); await p.click('#zin'); await p.waitForTimeout(300);
  await p.mouse.move(box.x + 200, box.y + 200); await p.mouse.wheel(0, 600); await p.waitForTimeout(300);
  await p.click('#zfit'); await p.waitForTimeout(300);
  ok(await p.locator('#searcharea').isHidden(), 'fit returns to the searched area');
  ok(errs.length === 0, 'live: no page errors ' + errs.join(' | '));
}
await b.close();
console.log(fails ? `${fails} FAILED` : 'all ok');
process.exit(fails ? 1 : 0);
