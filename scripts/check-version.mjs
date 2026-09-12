import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')

const packageVersion = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8')).version

const cargoToml = readFileSync(resolve(root, 'src-tauri', 'Cargo.toml'), 'utf8')
const cargoVersion = cargoToml.match(/^\[package\][\s\S]*?^version = "([^"]+)"/m)?.[1]

const tauriConf = JSON.parse(readFileSync(resolve(root, 'src-tauri', 'tauri.conf.json'), 'utf8'))
let tauriVersion = tauriConf.version
if (typeof tauriVersion === 'string' && tauriVersion.endsWith('.json')) {
  tauriVersion = JSON.parse(readFileSync(resolve(root, 'src-tauri', tauriVersion), 'utf8')).version
}

const tag = process.argv[2] ?? process.env.GITHUB_REF_NAME ?? ''
const problems = []

if (!packageVersion) problems.push('package.json 缺少 version')
if (cargoVersion !== packageVersion) {
  problems.push(`Cargo.toml version=${cargoVersion} 与 package.json ${packageVersion} 不一致`)
}
if (tauriVersion !== packageVersion) {
  problems.push(`tauri.conf.json version=${tauriVersion} 与 package.json ${packageVersion} 不一致`)
}
if (tag) {
  const expected = tag.startsWith('v') ? tag.slice(1) : tag
  if (expected !== packageVersion) {
    problems.push(`tag ${tag} 与 package.json ${packageVersion} 不一致`)
  }
}

if (problems.length > 0) {
  console.error('版本校验失败:')
  for (const problem of problems) console.error(` - ${problem}`)
  process.exit(1)
}

console.log(`版本校验通过: ${packageVersion}${tag ? ` (tag ${tag})` : ''}`)
