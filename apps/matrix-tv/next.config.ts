import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  reactStrictMode: true,
  experimental: {
    reactCompiler: false,
  },
  // Three.js transpilation.
  transpilePackages: ['three'],
};

export default nextConfig;
