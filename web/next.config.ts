import type { NextConfig } from "next";

const API_URL =
  process.env.REPLAY_API_URL ??
  process.env.NEXT_PUBLIC_REPLAY_API_URL ??
  "http://localhost:8080";

const nextConfig: NextConfig = {
  async rewrites() {
    return [
      {
        source: "/rpc/:path*",
        destination: `${API_URL}/:path*`,
      },
    ];
  },
};

export default nextConfig;
