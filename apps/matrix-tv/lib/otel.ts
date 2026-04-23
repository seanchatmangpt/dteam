/**
 * OpenTelemetry bootstrap for the matrix-tv client.
 *
 * Emits one span per SprawlEvent. The exporter targets an OTLP/HTTP
 * collector at `/otlp/v1/traces` by default — configure with
 * `NEXT_PUBLIC_OTLP_TRACES_URL=http://localhost:4318/v1/traces` at build
 * time. When no collector is reachable, the exporter fails silently and
 * the app keeps rendering; OTel is a bonus, not a blocker.
 */

'use client';

import { context, trace, type Tracer } from '@opentelemetry/api';
import { ZoneContextManager } from '@opentelemetry/context-zone';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { resourceFromAttributes } from '@opentelemetry/resources';
import { BatchSpanProcessor, WebTracerProvider } from '@opentelemetry/sdk-trace-web';
import {
  ATTR_SERVICE_NAME,
  ATTR_SERVICE_VERSION,
} from '@opentelemetry/semantic-conventions';

let tracer: Tracer | null = null;
let initialized = false;

export function initOtel(): Tracer {
  if (tracer) return tracer;
  if (typeof window === 'undefined') {
    // Guard against SSR evaluation.
    tracer = trace.getTracer('matrix-tv-sprawl-noop');
    return tracer;
  }

  const endpoint =
    process.env.NEXT_PUBLIC_OTLP_TRACES_URL ??
    'http://localhost:4318/v1/traces';

  const provider = new WebTracerProvider({
    resource: resourceFromAttributes({
      [ATTR_SERVICE_NAME]: 'matrix-tv-sprawl',
      [ATTR_SERVICE_VERSION]: '0.3.0',
      'deployment.environment': process.env.NODE_ENV ?? 'development',
    }),
    spanProcessors: [
      new BatchSpanProcessor(
        new OTLPTraceExporter({
          url: endpoint,
          // No credentials; the collector must allow CORS from localhost:3000.
        }),
        {
          maxQueueSize: 1024,
          scheduledDelayMillis: 250,
        }
      ),
    ],
  });

  provider.register({
    contextManager: new ZoneContextManager(),
  });

  initialized = true;
  tracer = trace.getTracer('matrix-tv-sprawl', '0.3.0');
  return tracer;
}

export function isOtelInitialized(): boolean {
  return initialized;
}

/** Re-export `context` and `trace` so call sites don't pin otel versions. */
export { context, trace };
