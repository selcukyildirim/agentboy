import { describe, expect, it } from "vitest";
import {
  formatBytes,
  formatDuration,
  formatPercent,
  formatTokens,
  formatUsd,
  shortId,
} from "./format";

describe("format helpers", () => {
  it("formats duration across magnitudes", () => {
    expect(formatDuration(850)).toBe("850 ms");
    expect(formatDuration(2500)).toBe("2.5 sn");
    expect(formatDuration(65000)).toBe("1 dk 5 sn");
    expect(formatDuration(null)).toBe("—");
  });

  it("formats usd", () => {
    expect(formatUsd(0)).toBe("$0");
    expect(formatUsd(0.0034)).toBe("$0.0034");
    expect(formatUsd(4.2)).toBe("$4.20");
    expect(formatUsd(undefined)).toBe("—");
  });

  it("formats percent", () => {
    expect(formatPercent(0.984, 1)).toBe("98.4%");
    expect(formatPercent(null)).toBe("—");
  });

  it("formats tokens compactly", () => {
    expect(formatTokens(0)).toBe("0");
    expect(formatTokens(1500)).toBe("1.5k");
    expect(formatTokens(2_400_000)).toBe("2.4M");
  });

  it("formats bytes", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(2048)).toBe("2.0 KB");
  });

  it("shortens ids", () => {
    expect(shortId("abcdefghijkl")).toBe("abcdefgh");
    expect(shortId(null)).toBe("—");
  });
});
