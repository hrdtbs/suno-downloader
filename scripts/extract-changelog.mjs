import { readFileSync } from 'node:fs';

const version = process.argv[2];
if (!version) {
  console.error('Usage: node scripts/extract-changelog.mjs <version>');
  process.exit(1);
}

const changelog = readFileSync('CHANGELOG.md', 'utf8');
const header = `## [${version}]`;
const start = changelog.indexOf(header);
if (start === -1) {
  console.error(`No changelog section found for version ${version}`);
  process.exit(1);
}

const afterHeader = changelog.indexOf('\n', start) + 1;
const nextSection = changelog.indexOf('\n## [', afterHeader);
const footerLinks = changelog.indexOf('\n[', afterHeader);
let end = changelog.length;
if (nextSection !== -1) {
  end = nextSection;
}
if (footerLinks !== -1) {
  end = Math.min(end, footerLinks);
}
const section = changelog.slice(afterHeader, end).trim();

if (!section) {
  console.error(`Changelog section for version ${version} is empty`);
  process.exit(1);
}

process.stdout.write(`${section}\n`);
