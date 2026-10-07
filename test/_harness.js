'use strict';

// Minimal test runner: no dependencies, so `npm test` works offline.
const assert = require('assert');

const tests = [];
function test(name, fn) { tests.push({ name, fn }); }

function approx(actual, expected, eps, msg) {
  assert.ok(Math.abs(actual - expected) <= eps,
    `${msg || 'approx'}: expected ${expected} ± ${eps}, got ${actual}`);
}

async function run(title) {
  let failed = 0;
  console.log(`\n${title}`);
  for (const t of tests) {
    try {
      await t.fn();
      console.log(`  ✓ ${t.name}`);
    } catch (err) {
      failed += 1;
      console.log(`  ✗ ${t.name}\n      ${String(err && err.stack || err).split('\n').slice(0, 3).join('\n      ')}`);
    }
  }
  console.log(`  ${tests.length - failed}/${tests.length} passed`);
  tests.length = 0;
  if (failed) process.exitCode = 1;
  return failed;
}

module.exports = { test, run, approx, assert };
