// E2E with WebdriverIO + tauri-driver (F6-06). Windows only; see e2e/README.md.
import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const application = join(root, "src-tauri", "target", "release", "agendo.exe");

let tauriDriver;
let dataDir;
let exiting = false;

function stopDriver() {
  exiting = true;
  tauriDriver?.kill();
}

export const config = {
  host: "127.0.0.1",
  port: 4444,
  specs: ["./specs/**/*.e2e.mjs"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: { ui: "bdd", timeout: 60000 },

  // The release build the tests drive. `--no-bundle`: no installer is needed.
  onPrepare: () => {
    const build = spawnSync(
      "npm",
      ["run", "tauri", "--", "build", "--no-bundle"],
      {
        cwd: root,
        stdio: "inherit",
        shell: true,
      },
    );
    if (build.status !== 0) throw new Error("tauri build failed");
  },

  // Each session gets an empty data directory, so the tests never touch the user's agenda.
  beforeSession: () => {
    dataDir = mkdtempSync(join(tmpdir(), "miagenda-e2e-"));
    process.env.MIAGENDA_DATA_DIR = dataDir;
    tauriDriver = spawn(join(homedir(), ".cargo", "bin", "tauri-driver"), [], {
      stdio: [null, process.stdout, process.stderr],
    });
    tauriDriver.on("error", (error) => {
      console.error("tauri-driver failed to start:", error);
      process.exit(1);
    });
    tauriDriver.on("exit", (code) => {
      if (!exiting) {
        console.error("tauri-driver exited with code", code);
        process.exit(1);
      }
    });
  },

  afterSession: () => {
    stopDriver();
    if (dataDir) rmSync(dataDir, { recursive: true, force: true });
  },
};

process.on("exit", stopDriver);
