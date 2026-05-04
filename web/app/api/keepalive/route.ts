import { NextResponse } from "next/server";

const API_URL =
  process.env.REPLAY_API_URL ?? "http://localhost:8080";

export async function GET() {
  try {
    const res = await fetch(`${API_URL}/health`, {
      next: { revalidate: 0 },
      signal: AbortSignal.timeout(10_000),
    });
    const ok = res.ok;
    return NextResponse.json({ ok, status: res.status });
  } catch (e) {
    return NextResponse.json({ ok: false, error: String(e) }, { status: 503 });
  }
}
