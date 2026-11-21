// 通过 CDP 在 Tauri WebView 中执行 JS，避免 adb input text 丢字符。
// 用法: node cdp_eval.js "<JS表达式>"   —— 输出 JSON.stringify(结果)
const http = require('http');

function getJson(path) {
  return new Promise((resolve, reject) => {
    http.get({ host: '127.0.0.1', port: 9222, path }, res => {
      let d = ''; res.on('data', c => d += c); res.on('end', () => resolve(JSON.parse(d)));
    }).on('error', reject);
  });
}

(async () => {
  const expr = process.argv[2];
  if (!expr) { console.error('usage: node cdp_eval.js "<js>"'); process.exit(1); }
  const targets = await getJson('/json/list');
  const page = targets.find(t => t.type === 'page' && t.url.includes('tauri.localhost')) || targets.find(t => t.type === 'page');
  if (!page) { console.error('no page target'); process.exit(2); }
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  let id = 0;
  const pending = new Map();
  ws.onmessage = (ev) => {
    const m = JSON.parse(ev.data);
    if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
  };
  const send = (method, params) => new Promise(resolve => {
    const myId = ++id; pending.set(myId, resolve);
    ws.send(JSON.stringify({ id: myId, method, params }));
  });
  await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
  const r = await send('Runtime.evaluate', {
    expression: expr,
    awaitPromise: true,
    returnByValue: true,
  });
  if (r.result && r.result.exceptionDetails) {
    console.error('EXCEPTION:', JSON.stringify(r.result.exceptionDetails));
  } else if (r.result && r.result.result) {
    const v = r.result.result.value;
    console.log(typeof v === 'string' ? v : JSON.stringify(v));
  } else {
    console.log(JSON.stringify(r));
  }
  ws.close();
})().catch(e => { console.error('ERR', e); process.exit(3); });
