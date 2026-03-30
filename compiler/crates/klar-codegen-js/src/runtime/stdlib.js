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

// === std.http ===
const http = (() => {
  try {
    const httpMod = require('http');
    const url = require('url');

    class Request {
      constructor(method, path, headers, body, params) {
        this.method = method;
        this.path = path;
        this.headers = headers || {};
        this.body = body || '';
        this.params = params || {};
        this.query = {};
      }
      param(name) { return this.params[name] || null; }
      header(name) { return this.headers[name.toLowerCase()] || null; }
      json() { return JSON.parse(this.body); }
      static get(path) { return new Request('GET', path, {}, '', {}); }
      static post(path, body) { return new Request('POST', path, {}, JSON.stringify(body), {}); }
    }

    class Response {
      constructor(status, headers, body) {
        this.status = status || 200;
        this._headers = headers || { 'Content-Type': 'application/json' };
        this._body = body || '';
      }
      static json(data, opts) {
        const status = opts && opts.status ? opts.status : 200;
        return new Response(status, { 'Content-Type': 'application/json' }, JSON.stringify(data));
      }
      static text(text, opts) {
        const status = opts && opts.status ? opts.status : 200;
        return new Response(status, { 'Content-Type': 'text/plain' }, text);
      }
      static redirect(url, status) {
        return new Response(status || 302, { 'Location': url }, '');
      }
    }

    class Router {
      constructor() {
        this.routes = [];
        this.middleware = [];
      }
      static new() { return new Router(); }
      get(path, handler) { this.routes.push({ method: 'GET', path, handler }); return this; }
      post(path, handler) { this.routes.push({ method: 'POST', path, handler }); return this; }
      put(path, handler) { this.routes.push({ method: 'PUT', path, handler }); return this; }
      delete(path, handler) { this.routes.push({ method: 'DELETE', path, handler }); return this; }
      use(mw) { this.middleware.push(mw); return this; }

      _match(method, pathname) {
        for (const route of this.routes) {
          if (route.method !== method) continue;
          const params = this._extractParams(route.path, pathname);
          if (params !== null) return { handler: route.handler, params };
        }
        return null;
      }

      _extractParams(pattern, pathname) {
        const patternParts = pattern.split('/');
        const pathParts = pathname.split('/');
        if (patternParts.length !== pathParts.length) return null;
        const params = {};
        for (let i = 0; i < patternParts.length; i++) {
          if (patternParts[i].startsWith('{') && patternParts[i].endsWith('}')) {
            params[patternParts[i].slice(1, -1)] = pathParts[i];
          } else if (patternParts[i].startsWith(':')) {
            params[patternParts[i].slice(1)] = pathParts[i];
          } else if (patternParts[i] !== pathParts[i]) {
            return null;
          }
        }
        return params;
      }
    }

    function serve(router, opts) {
      const port = typeof opts === 'number' ? opts : ((opts && opts.port) || 3000);
      const server = httpMod.createServer(async (req, res) => {
        const parsed = new url.URL(req.url, `http://localhost:${port}`);
        let body = '';
        for await (const chunk of req) body += chunk;

        const request = new Request(
          req.method, parsed.pathname, req.headers, body, {}
        );

        // Run middleware
        for (const mw of router.middleware) {
          const result = mw(request);
          if (result instanceof Response) {
            res.writeHead(result.status, result._headers);
            res.end(result._body);
            return;
          }
        }

        const match = router._match(req.method, parsed.pathname);
        if (!match) {
          res.writeHead(404, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ error: 'not found' }));
          return;
        }

        request.params = match.params;
        try {
          const response = await match.handler(request);
          res.writeHead(response.status, response._headers);
          res.end(response._body);
        } catch (e) {
          res.writeHead(500, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ error: e.message }));
        }
      });

      server.listen(port, () => {
        console.log(`[INFO] Klar HTTP server listening on port ${port}`);
      });
      return server;
    }

    // Middleware helpers
    function cors() {
      return (req) => {
        if (req.method === 'OPTIONS') {
          return new Response(204, {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
            'Access-Control-Allow-Headers': 'Content-Type, Authorization',
          }, '');
        }
        return null; // continue to next handler
      };
    }

    function logger() {
      return (req) => {
        console.log(`[${new Date().toISOString()}] ${req.method} ${req.path}`);
        return null;
      };
    }

    return { Request, Response, Router, serve, cors, logger };
  } catch {
    return {
      Request: class { constructor() { throw new Error('http not available'); } },
      Response: class { constructor() { throw new Error('http not available'); } },
      Router: class { constructor() { throw new Error('http not available'); } },
      serve: () => { throw new Error('http not available'); },
    };
  }
})();

// === std.sql ===
const sql = (() => {
  let pg;
  try { pg = require('pg'); } catch { pg = null; }

  class Pool {
    constructor(url) {
      this.url = url;
      if (pg) {
        this._pool = new pg.Pool({ connectionString: url });
      } else {
        this._pool = null;
        this._memory = new Map(); // in-memory fallback for testing
      }
    }
    static connect(url) { return new Pool(url || process.env.DATABASE_URL); }
    async query(q, params) {
      if (this._pool) {
        const result = await this._pool.query(q, params || []);
        return result.rows;
      }
      console.warn('[sql] query (no pg):', q, params || []);
      return [];
    }
    async execute(q, params) {
      if (this._pool) {
        const result = await this._pool.query(q, params || []);
        return { rows_affected: result.rowCount || 0 };
      }
      console.warn('[sql] execute (no pg):', q, params || []);
      return { rows_affected: 0 };
    }
    async close() { if (this._pool) await this._pool.end(); }
  }

  class Transaction {
    constructor(pool) { this.pool = pool; this._client = null; }
    async begin() {
      if (this.pool._pool) {
        this._client = await this.pool._pool.connect();
        await this._client.query('BEGIN');
      }
    }
    async query(q, params) {
      if (this._client) return (await this._client.query(q, params || [])).rows;
      return this.pool.query(q, params);
    }
    async execute(q, params) {
      if (this._client) {
        const r = await this._client.query(q, params || []);
        return { rows_affected: r.rowCount || 0 };
      }
      return this.pool.execute(q, params);
    }
    async commit() { if (this._client) { await this._client.query('COMMIT'); this._client.release(); } }
    async rollback() { if (this._client) { await this._client.query('ROLLBACK'); this._client.release(); } }
  }

  // Type-safe query builder
  class Query {
    constructor(table) { this.table = table; this._where = []; this._params = []; this._limit = null; this._order = null; }
    static from(table) { return new Query(table); }
    where(field, op, value) { this._params.push(value); this._where.push(`${field} ${op} $${this._params.length}`); return this; }
    limit(n) { this._limit = n; return this; }
    order_by(field, dir) { this._order = `${field} ${dir || 'ASC'}`; return this; }
    select() {
      let q = `SELECT * FROM ${this.table}`;
      if (this._where.length) q += ` WHERE ${this._where.join(' AND ')}`;
      if (this._order) q += ` ORDER BY ${this._order}`;
      if (this._limit) q += ` LIMIT ${this._limit}`;
      return { sql: q, params: this._params };
    }
    insert(data) {
      const keys = Object.keys(data);
      const vals = keys.map((_, i) => `$${i + 1}`);
      return { sql: `INSERT INTO ${this.table} (${keys.join(', ')}) VALUES (${vals.join(', ')}) RETURNING *`, params: keys.map(k => data[k]) };
    }
    update(data) {
      const keys = Object.keys(data);
      const sets = keys.map((k, i) => `${k} = $${i + 1}`);
      const params = keys.map(k => data[k]);
      let q = `UPDATE ${this.table} SET ${sets.join(', ')}`;
      if (this._where.length) q += ` WHERE ${this._where.join(' AND ')}`;
      return { sql: q + ' RETURNING *', params: [...params, ...this._params] };
    }
    delete() {
      let q = `DELETE FROM ${this.table}`;
      if (this._where.length) q += ` WHERE ${this._where.join(' AND ')}`;
      return { sql: q, params: this._params };
    }
  }

  return {
    connect: (url) => Pool.connect(url),
    Pool, Transaction, Query,
    query: (pool, q, params) => pool.query(q, params),
    execute: (pool, q, params) => pool.execute(q, params),
    from: (table) => Query.from(table),
    transaction: async (pool, fn) => {
      const tx = new Transaction(pool);
      try { await tx.begin(); const result = await fn(tx); await tx.commit(); return result; }
      catch (e) { await tx.rollback(); throw e; }
    },
  };
})();

// === Async/Concurrency primitives ===
const channel = {
  new: (buffer) => {
    const ch = { buffer: buffer || 0, queue: [], waiters: [], closed: false };
    return {
      send: async (value) => {
        if (ch.closed) throw new Error('send on closed channel');
        if (ch.waiters.length > 0) {
          const resolve = ch.waiters.shift();
          resolve(value);
        } else {
          ch.queue.push(value);
        }
      },
      recv: async () => {
        if (ch.queue.length > 0) return ch.queue.shift();
        if (ch.closed) return null;
        return new Promise((resolve) => ch.waiters.push(resolve));
      },
      close: () => { ch.closed = true; ch.waiters.forEach(r => r(null)); },
    };
  },
};

async function parallel(...fns) {
  return Promise.all(fns.map(fn => fn()));
}

async function spawn(fn) {
  return fn();
}

// === std.ws (WebSocket) ===
const ws = (() => {
  try {
    const httpMod = require('http');
    const cryptoMod = require('crypto');

    class WebSocketServer {
      constructor(server) {
        this.clients = new Set();
        this._handlers = {};
        server.on('upgrade', (req, socket, head) => {
          if (req.headers['upgrade'] !== 'websocket') { socket.destroy(); return; }
          const key = req.headers['sec-websocket-key'];
          const accept = cryptoMod.createHash('sha1')
            .update(key + '258EAFA5-E914-47DA-95CA-5AB5DC11CE56')
            .digest('base64');
          socket.write(
            'HTTP/1.1 101 Switching Protocols\r\n' +
            'Upgrade: websocket\r\n' +
            'Connection: Upgrade\r\n' +
            `Sec-WebSocket-Accept: ${accept}\r\n\r\n`
          );
          const client = { socket, send: (msg) => {
            const buf = Buffer.from(msg);
            const frame = Buffer.alloc(2 + buf.length);
            frame[0] = 0x81; frame[1] = buf.length;
            buf.copy(frame, 2);
            socket.write(frame);
          }};
          this.clients.add(client);
          if (this._handlers.connect) this._handlers.connect(client);
          socket.on('data', (data) => {
            const len = data[1] & 0x7f;
            const mask = data.slice(2, 6);
            const payload = data.slice(6, 6 + len);
            for (let i = 0; i < payload.length; i++) payload[i] ^= mask[i % 4];
            const msg = payload.toString('utf8');
            if (this._handlers.message) this._handlers.message(client, msg);
          });
          socket.on('close', () => {
            this.clients.delete(client);
            if (this._handlers.disconnect) this._handlers.disconnect(client);
          });
        });
      }
      on(event, handler) { this._handlers[event] = handler; return this; }
      broadcast(msg) { for (const c of this.clients) c.send(msg); }
    }

    return {
      upgrade: (server) => new WebSocketServer(server),
      WebSocketServer,
    };
  } catch {
    return { upgrade: () => { throw new Error('ws not available'); } };
  }
})();

// === std.redis ===
const redis = (() => {
  return {
    connect: (url) => {
      console.warn('[redis] connect:', url || 'redis://localhost:6379');
      const store = new Map();
      return {
        get: async (key) => store.get(key) || null,
        set: async (key, value, opts) => { store.set(key, value); },
        del: async (key) => store.delete(key),
        incr: async (key) => { const v = (store.get(key) || 0) + 1; store.set(key, v); return v; },
        keys: async (pattern) => [...store.keys()],
        expire: async (key, seconds) => { /* noop in memory */ },
      };
    },
  };
})();

// === std.auth ===
const auth = (() => {
  return {
    hash_password: (password) => {
      const c = require('crypto');
      const salt = c.randomBytes(16).toString('hex');
      const hash = c.pbkdf2Sync(password, salt, 100000, 64, 'sha512').toString('hex');
      return `${salt}:${hash}`;
    },
    verify_password: (password, stored) => {
      const c = require('crypto');
      const [salt, hash] = stored.split(':');
      const verify = c.pbkdf2Sync(password, salt, 100000, 64, 'sha512').toString('hex');
      return hash === verify;
    },
    generate_token: (payload) => {
      const c = require('crypto');
      const header = Buffer.from(JSON.stringify({ alg: 'HS256', typ: 'JWT' })).toString('base64url');
      const body = Buffer.from(JSON.stringify({ ...payload, iat: Date.now() })).toString('base64url');
      const sig = c.createHmac('sha256', process.env.JWT_SECRET || 'klar-dev-secret')
        .update(`${header}.${body}`).digest('base64url');
      return `${header}.${body}.${sig}`;
    },
    verify_token: (token) => {
      const c = require('crypto');
      const [header, body, sig] = token.split('.');
      const expected = c.createHmac('sha256', process.env.JWT_SECRET || 'klar-dev-secret')
        .update(`${header}.${body}`).digest('base64url');
      if (sig !== expected) return null;
      return JSON.parse(Buffer.from(body, 'base64url').toString());
    },
  };
})();

// === Std namespace ===
const std = { string, list, map, set, json, math, io, env, time, crypto, log, http, sql, channel, ws, redis, auth };
