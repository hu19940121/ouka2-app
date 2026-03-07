const fs = require("fs");
const path = require("path");

const rootDir = path.resolve(__dirname, "..");
const packageJsonPath = path.join(rootDir, "package.json");
const tauriConfigPath = path.join(rootDir, "src-tauri", "tauri.conf.json");
const cargoTomlPath = path.join(rootDir, "src-tauri", "Cargo.toml");

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, data) {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}

function syncTauriConfig(version) {
  const config = readJson(tauriConfigPath);
  if (config.version !== version) {
    config.version = version;
    writeJson(tauriConfigPath, config);
    console.log(`[sync-version] Updated tauri.conf.json -> ${version}`);
  }
}

function syncCargoToml(version) {
  const content = fs.readFileSync(cargoTomlPath, "utf8");
  const nextContent = content.replace(
    /^version\s*=\s*"[^"]+"/m,
    `version = "${version}"`
  );

  if (nextContent !== content) {
    fs.writeFileSync(cargoTomlPath, nextContent, "utf8");
    console.log(`[sync-version] Updated Cargo.toml -> ${version}`);
  }
}

function main() {
  const { version } = readJson(packageJsonPath);
  if (!version) {
    throw new Error("package.json is missing version");
  }

  syncTauriConfig(version);
  syncCargoToml(version);
  console.log(`[sync-version] Version synced to ${version}`);
}

main();
