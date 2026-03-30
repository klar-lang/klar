// Klar Standard Library — JavaScript Runtime
// This file is embedded in every compiled Klar program.

// === Core ===
function println(...args) { console.log(...args); }
function print(...args) { process.stdout.write(args.map(String).join(' ')); }
function assert(cond, msg) { if (!cond) throw new Error(msg || 'assertion failed'); }
function assert_eq(a, b) { if (a !== b) throw new Error(`assert_eq failed: ${JSON.stringify(a)} !== ${JSON.stringify(b)}`); }
function assert_err(fn) { try { fn(); throw new Error('expected error'); } catch(e) { if (e.message === 'expected error') throw e; } }

// === std.string ===
const string = {
  split: (s, sep) => s.split(sep),
  join: (arr, sep) => arr.join(sep),
  trim: (s) => s.trim(),
  contains: (s, sub) => s.includes(sub),
  replace: (s, from, to) => s.replaceAll(from, to),
  starts_with: (s, prefix) => s.startsWith(prefix),
  ends_with: (s, suffix) => s.endsWith(suffix),
  to_upper: (s) => s.toUpperCase(),
  to_lower: (s) => s.toLowerCase(),
  pad_left: (s, len, ch) => s.padStart(len, ch || ' '),
  pad_right: (s, len, ch) => s.padEnd(len, ch || ' '),
  length: (s) => s.length,
  chars: (s) => [...s],
  repeat: (s, n) => s.repeat(n),
  slice: (s, start, end) => s.slice(start, end),
};

// === std.list ===
const list = {
  map: (arr, fn) => arr.map(fn),
  filter: (arr, fn) => arr.filter(fn),
  reduce: (arr, fn, init) => arr.reduce(fn, init),
  find: (arr, fn) => { const r = arr.find(fn); return r === undefined ? null : r; },
  any: (arr, fn) => arr.some(fn),
  all: (arr, fn) => arr.every(fn),
  sort: (arr, fn) => [...arr].sort(fn),
  reverse: (arr) => [...arr].reverse(),
  take: (arr, n) => arr.slice(0, n),
  drop: (arr, n) => arr.slice(n),
  chunk: (arr, size) => { const r = []; for (let i = 0; i < arr.length; i += size) r.push(arr.slice(i, i + size)); return r; },
  zip: (a, b) => a.map((v, i) => [v, b[i]]),
  flatten: (arr) => arr.flat(),
  unique: (arr) => [...new Set(arr)],
  length: (arr) => arr.length,
  push: (arr, item) => [...arr, item],
  concat: (a, b) => [...a, ...b],
  includes: (arr, item) => arr.includes(item),
  index_of: (arr, item) => arr.indexOf(item),
};

// === std.map ===
const map = {
  get: (m, k) => { const v = m.get(k); return v === undefined ? null : v; },
  set: (m, k, v) => { const n = new Map(m); n.set(k, v); return n; },
  remove: (m, k) => { const n = new Map(m); n.delete(k); return n; },
  keys: (m) => [...m.keys()],
  values: (m) => [...m.values()],
  entries: (m) => [...m.entries()],
  merge: (a, b) => new Map([...a, ...b]),
  filter: (m, fn) => new Map([...m].filter(([k, v]) => fn(k, v))),
  map_values: (m, fn) => new Map([...m].map(([k, v]) => [k, fn(v)])),
  size: (m) => m.size,
  contains: (m, k) => m.has(k),
};

// === std.set ===
const set = {
  from: (arr) => new Set(arr),
  add: (s, v) => new Set([...s, v]),
  remove: (s, v) => { const n = new Set(s); n.delete(v); return n; },
  contains: (s, v) => s.has(v),
  union: (a, b) => new Set([...a, ...b]),
  intersection: (a, b) => new Set([...a].filter(x => b.has(x))),
  difference: (a, b) => new Set([...a].filter(x => !b.has(x))),
  is_subset: (a, b) => [...a].every(x => b.has(x)),
  size: (s) => s.size,
  to_list: (s) => [...s],
};

// === std.json ===
const json = {
  encode: (v) => JSON.stringify(v),
  decode: (s) => JSON.parse(s),
  pretty_print: (v) => JSON.stringify(v, null, 2),
};

// === std.math ===
const math = {
  abs: Math.abs,
  min: Math.min,
  max: Math.max,
  floor: Math.floor,
  ceil: Math.ceil,
  round: Math.round,
  pow: Math.pow,
  sqrt: Math.sqrt,
  pi: Math.PI,
  e: Math.E,
  log: Math.log,
  sin: Math.sin,
  cos: Math.cos,
  random: Math.random,
  clamp: (v, lo, hi) => Math.min(Math.max(v, lo), hi),
};

// === std.io ===
const io = (() => {
  try {
    const fs = require('fs');
    return {
      read_file: (path) => fs.readFileSync(path, 'utf-8'),
      write_file: (path, content) => fs.writeFileSync(path, content, 'utf-8'),
      file_exists: (path) => fs.existsSync(path),
      read_lines: (path) => fs.readFileSync(path, 'utf-8').split('\n'),
    };
  } catch {
    return {
      read_file: () => { throw new Error('io.read_file not available in browser'); },
      write_file: () => { throw new Error('io.write_file not available in browser'); },
      file_exists: () => false,
      read_lines: () => { throw new Error('io.read_lines not available in browser'); },
    };
  }
})();

// === std.env ===
const env = {
  get: (key) => process.env[key] || null,
  require: (key) => { const v = process.env[key]; if (!v) throw new Error(`env var '${key}' is required`); return v; },
  args: () => process.argv.slice(2),
};

// === std.time ===
const time = {
  now: () => new Date().toISOString(),
  timestamp: () => Date.now(),
  format: (iso, fmt) => iso, // simplified
  parse: (s) => new Date(s).toISOString(),
};

// === std.crypto ===
const crypto = (() => {
  try {
    const c = require('crypto');
    return {
      uuid: () => c.randomUUID(),
      random_bytes: (n) => c.randomBytes(n).toString('hex'),
      hash_sha256: (s) => c.createHash('sha256').update(s).digest('hex'),
    };
  } catch {
    return {
      uuid: () => 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, c => { const r = Math.random()*16|0; return (c === 'x' ? r : (r&0x3|0x8)).toString(16); }),
      random_bytes: () => Math.random().toString(36).slice(2),
      hash_sha256: () => { throw new Error('crypto not available'); },
    };
  }
})();

// === std.log ===
const log = {
  debug: (...args) => console.debug('[DEBUG]', ...args),
  info: (...args) => console.info('[INFO]', ...args),
  warn: (...args) => console.warn('[WARN]', ...args),
  error: (...args) => console.error('[ERROR]', ...args),
};

// === Std namespace ===
const std = { string, list, map, set, json, math, io, env, time, crypto, log };


function fib(n) {
  return ((n < 2) ? (() => { return n; })() : (() => { return (fib((n - 1)) + fib((n - 2))); })());
}

function main() {
  const result = fib(40);
  println(result);
}

// Entry point
main();
