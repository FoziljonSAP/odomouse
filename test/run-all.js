'use strict';

// `npm test`: every suite in its own process.
const { spawnSync } = require('child_process');
const path = require('path');

const suites = ['modules.test.js'];
let failed = 0;
for (const s of suites) {
  const r = spawnSync(process.execPath, [path.join(__dirname, s)], { stdio: 'inherit' });
  if (r.status !== 0) failed += 1;
}
console.log(failed ? `\n${failed} suite(s) failed` : '\nAll suites passed');
process.exit(failed ? 1 : 0);
