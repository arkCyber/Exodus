#!/usr/bin/env node
/**
 * Exodus encrypted sync vault — minimal reference HTTP server.
 *
 * Endpoints (base URL e.g. http://127.0.0.1:8787/api):
 *   GET  /health
 *   PUT  /vault/:deviceId   body: { deviceId, payload, updatedAt }
 *   GET  /vault/:deviceId   → { deviceId, payload, updatedAt }
 *
 * Env:
 *   PORT        default 8787
 *   SYNC_TOKEN  optional Bearer token (reject if missing/wrong)
 *   DATA_DIR    default ./sync-vault-data
 */

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PORT = Number(process.env.PORT || 8787);
const SYNC_TOKEN = (process.env.SYNC_TOKEN || '').trim();
const DATA_DIR = process.env.DATA_DIR || path.join(__dirname, '..', 'sync-vault-data');

fs.mkdirSync(DATA_DIR, { recursive: true });

function log(msg) {
  const ts = new Date().toISOString();
  console.log(`[${ts}] ${msg}`);
}

function send(res, status, body) {
  const data = typeof body === 'string' ? body : JSON.stringify(body);
  res.writeHead(status, {
    'Content-Type': 'application/json',
    'Access-Control-Allow-Origin': '*',
  });
  res.end(data);
}

function unauthorized(res) {
  send(res, 401, { error: 'Unauthorized' });
}

function checkAuth(req, res) {
  if (!SYNC_TOKEN) return true;
  const h = req.headers.authorization || '';
  if (h === `Bearer ${SYNC_TOKEN}`) return true;
  unauthorized(res);
  return false;
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    req.on('error', reject);
  });
}

function vaultPath(deviceId) {
  const safe = deviceId.replace(/[^a-zA-Z0-9_-]/g, '_');
  return path.join(DATA_DIR, `${safe}.json`);
}

const server = http.createServer(async (req, res) => {
  if (req.method === 'OPTIONS') {
    res.writeHead(204, {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, PUT, OPTIONS',
      'Access-Control-Allow-Headers': 'Authorization, Content-Type',
    });
    res.end();
    return;
  }

  const url = new URL(req.url || '/', `http://127.0.0.1:${PORT}`);
  const pathname = url.pathname.replace(/\/+$/, '') || '/';

  if (req.method === 'GET' && pathname === '/health') {
    send(res, 200, { ok: true, service: 'exodus-sync-vault', dataDir: DATA_DIR });
    return;
  }

  const vaultMatch = pathname.match(/^\/(?:api\/)?vault\/([^/]+)$/);
  if (!vaultMatch) {
    send(res, 404, { error: 'Not found' });
    return;
  }

  if (!checkAuth(req, res)) return;

  const deviceId = decodeURIComponent(vaultMatch[1]);
  const file = vaultPath(deviceId);

  if (req.method === 'PUT') {
    try {
      const raw = await readBody(req);
      const body = JSON.parse(raw || '{}');
      if (!body.payload || typeof body.payload !== 'string') {
        send(res, 400, { error: 'payload (base64) required' });
        return;
      }
      const record = {
        deviceId,
        payload: body.payload,
        updatedAt: body.updatedAt || Math.floor(Date.now() / 1000),
      };
      fs.writeFileSync(file, JSON.stringify(record, null, 2), 'utf8');
      log(`PUT vault ${deviceId} (${body.payload.length} b64 chars)`);
      send(res, 200, { ok: true, deviceId, updatedAt: record.updatedAt });
    } catch (e) {
      log(`PUT error: ${e.message}`);
      send(res, 400, { error: String(e.message) });
    }
    return;
  }

  if (req.method === 'GET') {
    if (!fs.existsSync(file)) {
      send(res, 404, { error: 'Vault not found' });
      return;
    }
    try {
      const record = JSON.parse(fs.readFileSync(file, 'utf8'));
      log(`GET vault ${deviceId}`);
      send(res, 200, record);
    } catch (e) {
      send(res, 500, { error: String(e.message) });
    }
    return;
  }

  send(res, 405, { error: 'Method not allowed' });
});

server.listen(PORT, '127.0.0.1', () => {
  log(`Exodus sync vault server http://127.0.0.1:${PORT}/api`);
  log(`  Health: GET http://127.0.0.1:${PORT}/health`);
  log(`  Vault:  PUT/GET http://127.0.0.1:${PORT}/vault/{deviceId}`);
  log(`  Configure Exodus: http://127.0.0.1:${PORT}/api`);
  if (SYNC_TOKEN) log('  Bearer auth enabled (SYNC_TOKEN)');
});
