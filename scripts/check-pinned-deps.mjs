import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync('package.json', 'utf8'));
const sections = ['dependencies', 'devDependencies', 'optionalDependencies', 'peerDependencies'];
const unpinned = [];

for (const section of sections) {
  const deps = pkg[section];
  if (!deps) continue;

  for (const [name, version] of Object.entries(deps)) {
    if (typeof version !== 'string') continue;
    if (/^[~^]/.test(version) || version.includes('||') || version.includes(' - ')) {
      unpinned.push(`${section}.${name}: ${version}`);
    }
  }
}

if (unpinned.length > 0) {
  console.error('Dependencies must be pinned to exact versions (no ^, ~, or ranges):');
  for (const entry of unpinned) {
    console.error(`  - ${entry}`);
  }
  process.exit(1);
}

console.log('All npm dependencies are pinned to exact versions.');
