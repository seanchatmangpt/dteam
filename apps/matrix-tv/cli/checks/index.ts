/**
 * matrix-tv-doctor check registry.
 *
 * Each check is a `() => Promise<CheckResult>`. The `doctor` command
 * runs the whole registry in parallel, prints a summary, and exits with
 * code 0 if nothing is in `miss` state.
 */

export type CheckStatus = 'ok' | 'warn' | 'miss' | 'skip';

export interface CheckResult {
  /** Stable id — used as the key in JSON output. */
  id: string;
  /** Human label shown in the default console output. */
  label: string;
  /** Outcome. `ok` = everything green, `warn` = degraded but usable,
   *  `miss` = blocks the demo, `skip` = inapplicable (e.g. optional
   *  dep not installed). */
  status: CheckStatus;
  /** One-line detail — what was observed. */
  detail: string;
  /** Remediation hint, shown only when status is not `ok`. */
  hint?: string;
  /** Milliseconds the check took. */
  durationMs: number;
}

export type Check = () => Promise<CheckResult>;

/** Wrap a check body with timing + unconditional error capture so one
 *  failing probe can't kill the doctor. */
export function check(
  id: string,
  label: string,
  body: () => Promise<Omit<CheckResult, 'id' | 'label' | 'durationMs'>>
): Check {
  return async () => {
    const t0 = Date.now();
    try {
      const partial = await body();
      return { id, label, ...partial, durationMs: Date.now() - t0 };
    } catch (e) {
      return {
        id,
        label,
        status: 'miss' as const,
        detail: `check threw: ${e instanceof Error ? e.message : String(e)}`,
        hint: 'file a bug — checks should never throw',
        durationMs: Date.now() - t0,
      };
    }
  };
}

export { nodeChecks } from './node.js';
export { portChecks } from './ports.js';
export { fileChecks } from './files.js';
export { httpChecks } from './http.js';
export { rustChecks } from './rust.js';
