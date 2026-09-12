import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const packageJsonPath = resolve(root, 'package.json')
const cargoTomlPath = resolve(root, 'src-tauri', 'Cargo.toml')

const { version } = JSON.parse(readFileSync(packageJsonPath, 'utf8'))
if (!version) {
  console.error('package.json 中缺少 version 字段')
  process.exit(1)
}

const cargoToml = readFileSync(cargoTomlPath, 'utf8')
const versionPattern = /(^\[package\][\s\S]*?^version = ")[^"]+(")/m

if (!versionPattern.test(cargoToml)) {
  console.error('未找到 src-tauri/Cargo.toml 的 [package] version')
  process.exit(1)
}

const updated = cargoToml.replace(versionPattern, `$1${version}$2`)
if (updated !== cargoToml) {
  writeFileSync(cargoTomlPath, updated)
  console.log(`已同步版本 ${version} → src-tauri/Cargo.toml`)
  console.log('提示：Cargo.lock 会在下次 cargo 命令时自动更新')
} else {
  console.log(`src-tauri/Cargo.toml 版本已是最新（${version}）`)
}
