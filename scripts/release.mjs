import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const srcTauriDir = resolve(root, 'src-tauri')
const BUMP_TYPES = ['patch', 'minor', 'major']

const args = process.argv.slice(2)
const dryRun = args.includes('--dry-run')
const skipChecks = args.includes('--skip-checks')
const bumpType = args.find((arg) => !arg.startsWith('--'))

function fail(message) {
  console.error(`\n✕ ${message}`)
  process.exit(1)
}

function run(command, options = {}) {
  if (dryRun && !options.executeInDryRun) {
    console.log(`\n[dry-run] 跳过: ${command}`)
    return
  }
  console.log(`\n> ${command}`)
  execSync(command, {
    cwd: options.cwd ?? root,
    stdio: options.quiet ? 'ignore' : 'inherit'
  })
}

function capture(command) {
  return execSync(command, { cwd: root, encoding: 'utf8' }).trim()
}

function parseVersion(version) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version)
  if (!match) fail(`package.json 版本格式无效: ${version}`)
  return { major: Number(match[1]), minor: Number(match[2]), patch: Number(match[3]) }
}

function nextVersion(version, type) {
  const current = parseVersion(version)
  if (type === 'major') return `${current.major + 1}.0.0`
  if (type === 'minor') return `${current.major}.${current.minor + 1}.0`
  return `${current.major}.${current.minor}.${current.patch + 1}`
}

if (!bumpType || !BUMP_TYPES.includes(bumpType)) {
  fail(`用法: npm run release -- ${BUMP_TYPES.join('|')} [--dry-run] [--skip-checks]`)
}

console.log(`\n=== Folder Manager 发布流程（${bumpType}${dryRun ? ' / dry-run' : ''}）===`)

console.log('\n[1/5] 预检')
const dirty = capture('git status --porcelain')
if (dirty) fail('工作区有未提交的改动，请先提交后再发布')
const branch = capture('git rev-parse --abbrev-ref HEAD')
if (branch !== 'main') fail(`当前分支为 ${branch}，请在 main 分支上发布`)
run('git fetch origin main', { executeInDryRun: true })
if (capture('git rev-parse HEAD') !== capture('git rev-parse origin/main')) {
  fail('本地 main 与 origin/main 不一致，请先同步后再发布')
}
const packageVersion = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8')).version
const targetVersion = nextVersion(packageVersion, bumpType)
const targetTag = `v${targetVersion}`

console.log('\n[2/5] CHANGELOG 校验')
const changelog = readFileSync(resolve(root, 'docs', 'CHANGELOG.md'), 'utf8')
const versionHeading = new RegExp(`^## \\[${targetVersion.replaceAll('.', '\\.')}\\]`, 'm')
if (!versionHeading.test(changelog)) {
  fail(
    `docs/CHANGELOG.md 中缺少 ## [${targetVersion}] 段落。\n  请先补写该版本说明并提交，再重新运行发布。`
  )
}
console.log(`CHANGELOG 已包含 [${targetVersion}]`)

console.log('\n[3/5] 质量门禁')
if (skipChecks) {
  console.log('已跳过质量门禁（--skip-checks）')
} else {
  run('npm run typecheck')
  run('npm run lint')
  run('npm run test')
  run('cargo fmt --all -- --check', { cwd: srcTauriDir })
  run('cargo clippy --all-targets --all-features -- -D warnings', { cwd: srcTauriDir })
  run('cargo test --all-features', { cwd: srcTauriDir })
}

let versionBumped = false
try {
  console.log('\n[4/5] 版本更新')
  run(`npm version ${bumpType} --no-git-tag-version`)
  versionBumped = true
  run('node scripts/sync-version.mjs')
  run('cargo check --all-features --quiet', { cwd: srcTauriDir })
  run('node scripts/check-version.mjs')

  console.log('\n[5/5] 提交与推送')
  run('git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock')
  run(`git commit -m "chore(release): ${targetTag}"`)
  run(`git tag -a ${targetTag} -m "Folder Manager ${targetVersion}"`)
  run('git push origin main --follow-tags')
} catch (error) {
  console.error(`\n✕ 发布中断: ${error.message}`)
  if (versionBumped) {
    console.error('本地版本文件已修改，可能已提交或已打 tag，请检查并按需回滚：')
    console.error('  git log -3 --oneline')
    console.error(`  git tag --list "${targetTag}"`)
    console.error('  git reset --hard HEAD~1   # 回滚最近一次发布提交（确认无其他改动后使用）')
    console.error(`  git tag -d ${targetTag}    # 删除本地 tag`)
  }
  process.exit(1)
}

if (dryRun) {
  console.log(`\n[dry-run] 完成预演：正式发布将 bump 到 ${targetTag} 并推送 tag`)
} else {
  console.log(`\n✓ 已推送 ${targetTag}，GitHub Actions 将构建并创建 draft Release`)
  console.log('  完成后到 GitHub 检查资产与说明，确认无误再点 Publish release')
}
