import { spawn } from 'child_process';
import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * Locate the compiled Rust binary.
 * Search order:
 * 1. cli/bin/masyv-core (bundled with npm package)
 * 2. core/target/release/masyv-core (development)
 */
function findBinary() {
  const candidates = [
    join(__dirname, '..', 'bin', 'masyv-core'),
    join(__dirname, '..', '..', 'core', 'target', 'release', 'masyv-core'),
    join(__dirname, '..', '..', 'core', 'target', 'debug', 'masyv-core'),
  ];

  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    'masyv-core binary not found. Run "cargo build --release" in the core/ directory first.\n' +
    `Searched: ${candidates.join(', ')}`
  );
}

/**
 * Run the Rust core binary with the given arguments.
 * @param {string[]} args - CLI arguments to pass
 * @param {(line: string) => void} onStderr - Callback for stderr lines (status updates)
 * @returns {Promise<{stdout: string, stderr: string, exitCode: number}>}
 */
export function runCore(args, onStderr = () => {}) {
  return new Promise((resolve, reject) => {
    let binary;
    try {
      binary = findBinary();
    } catch (err) {
      reject(err);
      return;
    }

    const child = spawn(binary, args, {
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr.on('data', (data) => {
      const text = data.toString();
      stderr += text;
      // Forward each line to the callback
      for (const line of text.split('\n').filter(Boolean)) {
        onStderr(line.trim());
      }
    });

    child.on('error', (err) => {
      reject(new Error(`Failed to start masyv-core: ${err.message}`));
    });

    child.on('close', (code) => {
      resolve({
        stdout,
        stderr,
        exitCode: code ?? 1,
      });
    });
  });
}
