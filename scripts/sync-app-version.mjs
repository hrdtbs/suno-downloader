import { readFileSync, writeFileSync } from 'node:fs';

const version = process.argv[2];
if (!version) {
  console.error('Usage: node scripts/sync-app-version.mjs <version>');
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+(-[\w.-]+)?(\+[\w.-]+)?$/.test(version)) {
  console.error(`Invalid semver version: ${version}`);
  process.exit(1);
}

const pkgPath = 'package.json';
const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
pkg.version = version;
writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`);

const tauriConfPath = 'src-tauri/tauri.conf.json';
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf8'));
tauriConf.version = version;
writeFileSync(tauriConfPath, `${JSON.stringify(tauriConf, null, 2)}\n`);

const cargoPath = 'src-tauri/Cargo.toml';
const cargoLines = readFileSync(cargoPath, 'utf8').split('\n');
let inPackage = false;
let replaced = false;

const updatedCargo = cargoLines
  .map((line) => {
    if (line.trim() === '[package]') {
      inPackage = true;
      return line;
    }
    if (line.startsWith('[') && line.trim() !== '[package]') {
      inPackage = false;
    }
    if (inPackage && line.startsWith('version = ')) {
      replaced = true;
      return `version = "${version}"`;
    }
    return line;
  })
  .join('\n');

if (!replaced) {
  console.error('Failed to update version in src-tauri/Cargo.toml');
  process.exit(1);
}

writeFileSync(cargoPath, updatedCargo);
console.log(`Synced app version to ${version}`);
