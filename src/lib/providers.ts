import type { ProviderProfile } from "./types";

export function enabledProviders(providers: ProviderProfile[], kind: string): ProviderProfile[] {
  return providers.filter((provider) => provider.kind === kind && provider.enabled);
}

export function needsApiKey(provider: ProviderProfile): boolean {
  return (
    !provider.id.startsWith("mock-") &&
    provider.vendor !== "ollama" &&
    provider.id !== "comfyui-image"
  );
}

export function preferredProviderId(
  providers: ProviderProfile[],
  kind: string,
  fallback: string,
): string {
  const available = enabledProviders(providers, kind);
  const current = available.find((provider) => provider.id === fallback);
  if (current && !current.id.startsWith("mock-")) return current.id;
  const ready =
    available.find((provider) => needsApiKey(provider) && provider.apiKeySet) ??
    available.find((provider) => !needsApiKey(provider));
  return ready?.id ?? current?.id ?? "";
}
