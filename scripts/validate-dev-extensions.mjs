#!/usr/bin/env node
/**
 * Exodus Browser — aerospace-grade validator for workspace `extensions/` dev samples.
 *
 * Checks: manifest JSON, referenced assets, JS syntax (node --check), HTML lang,
 * strict mode, header comments, and timestamped logging conventions.
 */

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const EXT_DIR = path.join(ROOT, 'extensions');

/** @typedef {{ file: string, reason: string }} Issue */

/**
 * Emit a validation issue.
 * @param {Issue[]} issues
 * @param {string} file
 * @param {string} reason
 */
function addIssue(issues, file, reason) {
  issues.push({ file, reason });
}

/**
 * List extension subdirectories.
 * @returns {string[]}
 */
function listExtensionDirs() {
  if (!fs.existsSync(EXT_DIR)) {
    return [];
  }
  return fs
    .readdirSync(EXT_DIR, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => path.join(EXT_DIR, d.name))
    .sort();
}

/**
 * Parse script src attributes from popup HTML.
 * @param {string} html
 * @returns {string[]}
 */
function scriptSrcsFromHtml(html) {
  const out = [];
  const re = /<script[^>]+src=["']([^"']+)["']/gi;
  let m;
  while ((m = re.exec(html)) !== null) {
    out.push(m[1]);
  }
  return out;
}

/**
 * Collect JS paths referenced by a manifest object.
 * @param {string} extRoot
 * @param {Record<string, unknown>} manifest
 * @returns {string[]}
 */
function referencedJs(extRoot, manifest) {
  /** @type {string[]} */
  const files = [];
  const bg = /** @type {{ service_worker?: string } | undefined} */ (manifest.background);
  if (bg?.service_worker) {
    files.push(path.join(extRoot, bg.service_worker));
  }
  const scripts = /** @type {Array<{ js?: string[] }>} */ (manifest.content_scripts || []);
  for (const cs of scripts) {
    for (const rel of cs.js || []) {
      files.push(path.join(extRoot, rel));
    }
  }
  const action = /** @type {{ default_popup?: string } | undefined} */ (manifest.action);
  if (action?.default_popup) {
    const popup = path.join(extRoot, action.default_popup);
    if (action.default_popup.endsWith('.html') && fs.existsSync(popup)) {
      const html = fs.readFileSync(popup, 'utf8');
      for (const src of scriptSrcsFromHtml(html)) {
        files.push(path.join(extRoot, src));
      }
    } else if (action.default_popup.endsWith('.js')) {
      files.push(popup);
    }
  }
  return files;
}

/**
 * Collect CSS paths from manifest content_scripts.
 * @param {string} extRoot
 * @param {Record<string, unknown>} manifest
 * @returns {string[]}
 */
function referencedCss(extRoot, manifest) {
  /** @type {string[]} */
  const files = [];
  const scripts = /** @type {Array<{ css?: string[] }>} */ (manifest.content_scripts || []);
  for (const cs of scripts) {
    for (const rel of cs.css || []) {
      files.push(path.join(extRoot, rel));
    }
  }
  return files;
}

/**
 * Audit a single .js file for Exodus extension conventions.
 * @param {string} filePath
 * @param {string} body
 * @returns {Issue[]}
 */
function auditJs(filePath, body) {
  /** @type {Issue[]} */
  const issues = [];
  if (!body.trimStart().startsWith('/**')) {
    addIssue(issues, filePath, "missing file header block comment (/** ... */)");
  }
  if (!body.includes("'use strict'") && !body.includes('"use strict"')) {
    addIssue(issues, filePath, "missing 'use strict'");
  }
  if (!body.includes('tsLog') && !body.includes('LOG_PREFIX')) {
    addIssue(issues, filePath, 'missing timestamped logging (tsLog or LOG_PREFIX)');
  }
  if (!body.includes('toISOString')) {
    addIssue(issues, filePath, 'missing ISO timestamp in logs (toISOString)');
  }
  const check = spawnSync(process.execPath, ['--check', filePath], {
    encoding: 'utf8',
  });
  if (check.status !== 0) {
    addIssue(
      issues,
      filePath,
      `syntax check failed: ${(check.stderr || check.stdout || '').trim()}`,
    );
  }
  return issues;
}

/**
 * Validate one extension folder.
 * @param {string} extRoot
 * @returns {Issue[]}
 */
function validateExtension(extRoot) {
  /** @type {Issue[]} */
  const issues = [];
  const manifestPath = path.join(extRoot, 'manifest.json');
  if (!fs.existsSync(manifestPath)) {
    addIssue(issues, manifestPath, 'manifest.json missing');
    return issues;
  }
  let manifest;
  try {
    manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  } catch (e) {
    addIssue(issues, manifestPath, `invalid JSON: ${e}`);
    return issues;
  }
  if (manifest.manifest_version !== 3) {
    addIssue(issues, manifestPath, `manifest_version must be 3, got ${manifest.manifest_version}`);
  }
  if (!manifest.name || !manifest.version) {
    addIssue(issues, manifestPath, 'name and version are required');
  }

  for (const js of referencedJs(extRoot, manifest)) {
    if (!fs.existsSync(js)) {
      addIssue(issues, js, 'referenced JS file missing');
      continue;
    }
    const body = fs.readFileSync(js, 'utf8');
    issues.push(...auditJs(js, body));
  }

  for (const css of referencedCss(extRoot, manifest)) {
    if (!fs.existsSync(css)) {
      addIssue(issues, css, 'referenced CSS file missing');
      continue;
    }
    const body = fs.readFileSync(css, 'utf8');
    if (!body.trimStart().startsWith('/**')) {
      addIssue(issues, css, 'CSS file missing header block comment');
    }
  }

  const action = manifest.action;
  if (action?.default_popup?.endsWith('.html')) {
    const popup = path.join(extRoot, action.default_popup);
    if (!fs.existsSync(popup)) {
      addIssue(issues, popup, 'popup HTML missing');
    } else {
      const html = fs.readFileSync(popup, 'utf8');
      if (!/<html[^>]*\slang=["']en["']/i.test(html)) {
        addIssue(issues, popup, 'popup HTML must set lang="en"');
      }
    }
  }

  return issues;
}

function main() {
  const dirs = listExtensionDirs();
  if (dirs.length === 0) {
    console.error(`No extensions found under ${EXT_DIR}`);
    process.exit(1);
  }
  /** @type {Issue[]} */
  const all = [];
  for (const dir of dirs) {
    const id = path.basename(dir);
    const issues = validateExtension(dir);
    if (issues.length) {
      console.error(`\n✗ ${id}`);
      for (const i of issues) {
        console.error(`  ${i.file}: ${i.reason}`);
      }
      all.push(...issues);
    } else {
      console.log(`✓ ${id}`);
    }
  }
  if (all.length) {
    console.error(`\n${all.length} issue(s) in dev extensions`);
    process.exit(1);
  }
  console.log(`\nAll ${dirs.length} dev extension(s) passed validation`);
}

main();
