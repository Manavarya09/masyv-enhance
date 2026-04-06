#!/usr/bin/env node

import { createRequire } from 'module';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { runCore } from '../src/runner.js';
import { formatResult, formatError } from '../src/logger.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const pkg = JSON.parse(readFileSync(join(__dirname, '..', 'package.json'), 'utf-8'));

const program = new Command();

program
  .name('masyv')
  .description('MASYV Enhance Engine — AI-powered image enhancement')
  .version(pkg.version);

program
  .command('enhance')
  .description('Enhance an image using AI-powered processing')
  .argument('<input>', 'Input image path (or directory with --batch)')
  .option('--smart', 'Auto-detect image type and route to best pipeline', false)
  .option('--scale <factor>', 'Upscale factor (2, 4, 8)', '4')
  .option('--format <fmt>', 'Output format (png, jpeg, webp, svg)', 'png')
  .option('-o, --output <path>', 'Output file path')
  .option('--quality <n>', 'JPEG quality (1-100)', '90')
  .option('--batch', 'Process all images in directory', false)
  .option('--model-dir <path>', 'Directory containing ONNX models')
  .option('--mode <mode>', 'Processing mode (smart, upscale, vectorize, enhance)', 'smart')
  .option('--verbose', 'Verbose output', false)
  .action(async (input, options) => {
    const spinner = ora({
      text: chalk.cyan('Processing image...'),
      spinner: 'dots',
    }).start();

    try {
      // Build args for the Rust binary
      const args = [input];

      const mode = options.smart ? 'smart' : (options.mode || 'smart');
      args.push('--mode', mode);
      args.push('--scale', options.scale);
      args.push('--format', options.format);
      args.push('--quality', options.quality);
      args.push('--json');

      if (options.output) args.push('--output', options.output);
      if (options.batch) args.push('--batch');
      if (options.modelDir) args.push('--model-dir', options.modelDir);
      if (options.verbose) args.push('--verbose');

      const result = await runCore(args, (line) => {
        // Stream stderr as status updates
        spinner.text = chalk.dim(line);
      });

      spinner.stop();

      if (result.exitCode !== 0) {
        console.error(formatError(result.stderr));
        process.exit(result.exitCode);
      }

      // Parse JSON output from Rust binary
      try {
        const data = JSON.parse(result.stdout);
        console.log(formatResult(data));
      } catch {
        // Non-JSON output, display as-is
        if (result.stdout.trim()) {
          console.log(result.stdout.trim());
        }
      }
    } catch (err) {
      spinner.stop();
      console.error(chalk.red(`Error: ${err.message}`));
      process.exit(1);
    }
  });

program
  .command('info')
  .description('Show system info and model status')
  .action(() => {
    console.log(chalk.bold('\nMASYV Enhance Engine'));
    console.log(chalk.dim('─'.repeat(40)));
    console.log(`  Version:  ${chalk.green(pkg.version)}`);
    console.log(`  Runtime:  ${chalk.cyan('Rust + ONNX')}`);
    console.log(`  Models:   Real-ESRGAN x4plus`);
    console.log(`  Formats:  PNG, JPEG, WebP, SVG`);
    console.log(`  Modes:    smart, upscale, vectorize, enhance`);
    console.log();
  });

program.parse();
