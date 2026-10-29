import { describe, expect, it } from "vitest";
import type { ProviderProfile } from "./types";
import { enabledProviders, preferredProviderId } from "./providers";

const providers = [
  { id: "mock-llm", kind: "llm", vendor: "mock", label: "Mock", enabled: true, apiKeySet: false },
  { id: "deepseek-llm", kind: "llm", vendor: "deepseek", label: "DeepSeek", enabled: true, apiKeySet: true },
  { id: "openai-llm", kind: "llm", vendor: "openai", label: "OpenAI", enabled: false, apiKeySet: true },
  { id: "qwen-llm", kind: "llm", vendor: "qwen", label: "Qwen", enabled: true, apiKeySet: false },
] as unknown as ProviderProfile[];

describe("provider routing", () => {
  it("prefers a configured real model over a mock default", () => {
    expect(preferredProviderId(providers, "llm", "mock-llm")).toBe("deepseek-llm");
  });

  it("keeps an explicitly selected real model", () => {
    expect(preferredProviderId(providers, "llm", "deepseek-llm")).toBe("deepseek-llm");
  });

  it("falls back to mock when no real model has a key", () => {
    const onlyMock = providers.filter((provider) => provider.id !== "deepseek-llm");
    expect(preferredProviderId(onlyMock, "llm", "mock-llm")).toBe("mock-llm");
  });

  it("never offers a disabled provider", () => {
    expect(enabledProviders(providers, "llm").map((provider) => provider.id)).toEqual([
      "mock-llm",
      "deepseek-llm",
      "qwen-llm",
    ]);
  });
});
