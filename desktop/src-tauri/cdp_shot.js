// Capture a screenshot of the WebView page via CDP and print base64 PNG.
// usage: node cdp_shot.js [outfile]
const http = require('http');

function getJSON(url) {
  return new Promise((resolve, reject) => {
    http.get(url, res => {
      let d = '';
      res.on('data', c => d += c);
      res.on('end', () => { try { resolve(JSON.parse(d)); } catch (e) { reject(e); } });
    }).on('error', reject);
  });
}

(async () => {
  const targets = await getJSON('http://127.0.0.1:9222/json/list');
  const page = targets.find(t => t.type === 'page' && /tauri\.localhost|localhost|http/.test(t.url)) || targets.find(t => t.type === 'page');
  if (!page) { console.error('no page target'); process.exit(1); }
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  let id = 0;
  const pending = new Map();
  ws.onmessage = ev => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); }
  };
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  const send = (method, params = {}) => new Promise(res => {
    const myId = ++id;
    pending.set(myId, res);
    ws.send(JSON.stringify({ id: myId, method, params }));
  });
  await send('Page.enable');
  const shot = await send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
  const b64 = shot.result && shot.result.data;
  if (!b64) { console.error('no data'); process.exit(1); }
  const fs = require('fs');
  const out = process.argv[2];
  if (out) { fs.writeFileSync(out, Buffer.from(b64, 'base64')); console.log('SAVED ' + out); }
  else { process.stdout.write(b64); }
  ws.close();
})().catch(e => { console.error(e); process.exit(1); });
