import crypto from 'node:crypto';
console.log('node', process.version);
console.log('openssl', crypto.constants.OPENSSL_VERSION_NUMBER, process.versions.openssl);
const ciphers = crypto.getCiphers();
console.log('chacha ciphers:', ciphers.filter(x => x.includes('chacha')));
console.log('has xchacha20-poly1305:', ciphers.some(x => x.includes('xchacha')));
// argon2id availability
try {
  const k = crypto.scryptSync('p','s',32,{N:16,r:8,p:1,maxmem:64*1024*1024});
  console.log('scryptSync OK', k.length);
} catch(e){ console.log('scrypt err', e.message); }
console.log('argon2 in hashes:', (crypto.getHashes()||[]).filter(x=>x.includes('argon')));
