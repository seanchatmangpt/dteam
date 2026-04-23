#!/usr/bin/env -S tsx
/**
 * matrix-tv-doctor — DX/QoL CLI for the Sprawl MUD.
 *
 * Run via `npm run doctor`, `npm run tail`, etc. Each subcommand is a
 * standalone defineCommand; they share the check registry in
 * `cli/checks/`.
 */

import { defineCommand, runMain } from 'citty';
import doctor from './commands/doctor.js';
import ocel from './commands/ocel.js';
import play from './commands/play.js';
import ports from './commands/ports.js';
import replay from './commands/replay.js';
import tail from './commands/tail.js';

const main = defineCommand({
  meta: {
    name: 'matrix-tv-doctor',
    version: '0.1.0',
    description: 'DX health checks and helpers for the matrix-tv Sprawl demo',
  },
  subCommands: {
    doctor,
    ports,
    replay,
    tail,
    ocel,
    play,
  },
});

runMain(main);
