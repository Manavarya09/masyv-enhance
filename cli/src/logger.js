import chalk from 'chalk';

/**
 * Format a successful enhancement result for display.
 * @param {object} result - Parsed JSON from masyv-core
 */
export function formatResult(result) {
  const lines = [];
  lines.push('');
  lines.push(chalk.green.bold('  Enhancement complete'));
  lines.push(chalk.dim('  ' + '─'.repeat(38)));

  if (result.detected_type) {
    lines.push(`  ${chalk.dim('Type:')}     ${chalk.yellow(result.detected_type)}`);
  }
  lines.push(`  ${chalk.dim('Mode:')}     ${chalk.cyan(result.mode_used)}`);
  lines.push(
    `  ${chalk.dim('Input:')}    ${result.input_dimensions[0]}x${result.input_dimensions[1]}`
  );
  lines.push(
    `  ${chalk.dim('Output:')}   ${result.output_dimensions[0]}x${result.output_dimensions[1]}`
  );
  lines.push(`  ${chalk.dim('Format:')}   ${result.format}`);
  lines.push(`  ${chalk.dim('Time:')}     ${result.processing_time_ms}ms`);
  lines.push(`  ${chalk.dim('Saved:')}    ${chalk.underline(result.output_path)}`);
  lines.push('');

  return lines.join('\n');
}

/**
 * Format an error message for display.
 * @param {string} stderr - Raw stderr output
 */
export function formatError(stderr) {
  const lines = stderr.split('\n').filter(Boolean);
  const formatted = lines
    .map((line) => {
      if (line.includes('ERROR')) return chalk.red(`  ${line}`);
      if (line.includes('WARN')) return chalk.yellow(`  ${line}`);
      return chalk.dim(`  ${line}`);
    })
    .join('\n');

  return `\n${chalk.red.bold('  Enhancement failed')}\n${formatted}\n`;
}
