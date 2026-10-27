export type AspectRatio = "9:16" | "16:9" | "1:1" | "4:5";
export type Route =
  | "dashboard"
  | "script"
  | "characters"
  | "shots"
  | "editor"
  | "assets"
  | "settings";

export type JobStatus = "queued" | "running" | "succeeded" | "failed" | "canceled";
export type AssetKind = "image" | "video" | "audio" | "subtitle";
export type ShotStatus = "draft" | "needs_review" | "approved" | "ready" | "failed";

export interface StyleBible {
  visualStyle: string;
  palette: string;
  lighting: string;
  cameraLanguage: string;
  negativePrompt: string;
  referenceNotes: string;
}

export interface Project {
  id: string;
  name: string;
  premise: string;
  logline: string;
  genre: string;
  tone: string;
  language: string;
  aspectRatio: string;
  targetEpisodeSeconds: number;
  budgetCap: number;
  styleBible: StyleBible;
  createdAt: string;
  updatedAt: string;
}

export interface ProjectCounts {
  characters: number;
  episodes: number;
  scenes: number;
  shots: number;
  videos: number;
  images: number;
}

export interface ProjectSummary {
  id: string;
  name: string;
  logline: string;
  premise: string;
  genre: string;
  aspectRatio: string;
  updatedAt: string;
  coverPath: string | null;
  storagePath: string;
  counts: ProjectCounts;
}

export interface Character {
  id: string;
  name: string;
  role: string;
  description: string;
  appearance: string;
  personality: string;
  voice: string;
  negativePrompt: string;
  lockedTraits: string[];
  referenceAssetId: string | null;
  sheetAssetId: string | null;
  seed: number | null;
  providerId: string;
  model: string;
  status: string;
  createdAt: string;
  updatedAt: string;
}

export interface Shot {
  id: string;
  sceneId: string;
  orderIndex: number;
  title: string;
  description: string;
  prompt: string;
  negativePrompt: string;
  camera: string;
  lens: string;
  durationMs: number;
  dialogue: string;
  sound: string;
  characterIds: string[];
  continuityFromShotId: string | null;
  startFrameAssetId: string | null;
  imageAssetId: string | null;
  videoAssetId: string | null;
  voiceAssetId: string | null;
  status: string;
  providerId: string;
  model: string;
  seed: number | null;
  version: number;
  createdAt: string;
  updatedAt: string;
}

export interface Scene {
  id: string;
  episodeId: string;
  title: string;
  slugline: string;
  location: string;
  timeOfDay: string;
  synopsis: string;
  scriptText: string;
  orderIndex: number;
  status: string;
  shots: Shot[];
  createdAt: string;
  updatedAt: string;
}

export interface Episode {
  id: string;
  title: string;
  summary: string;
  orderIndex: number;
  scenes: Scene[];
  createdAt: string;
  updatedAt: string;
}

export interface Asset {
  id: string;
  kind: AssetKind;
  name: string;
  relPath: string;
  absPath: string;
  thumbPath: string | null;
  mime: string;
  width: number | null;
  height: number | null;
  durationMs: number | null;
  sizeBytes: number | null;
  source: string;
  providerId: string | null;
  model: string | null;
  prompt: string | null;
  negativePrompt: string | null;
  seed: number | null;
  version: number;
  parentAssetId: string | null;
  license?: string | null;
  licenseUrl?: string | null;
  attribution?: string | null;
  sourceUrl?: string | null;
  status: string;
  createdAt: string;
}

export interface ShotTake {
  asset: Asset;
  selected: boolean;
  jobId: string | null;
  costActual: number;
}

export interface GenerationJob {
  id: string;
  projectId: string;
  kind: string;
  targetType: string;
  targetId: string | null;
  providerId: string;
  providerLabel: string;
  model: string;
  status: JobStatus;
  progress: number;
  stage: string;
  error: string | null;
  costEstimate: number;
  costActual: number;
  requestSummary: string;
  request: Record<string, unknown>;
  result: Record<string, unknown> | null;
  resultAssetId: string | null;
  createdAt: string;
  startedAt: string | null;
  finishedAt: string | null;
}

export interface TimelineClip {
  id: string;
  timelineId: string;
  shotId: string | null;
  assetId: string | null;
  orderIndex: number;
  inMs: number;
  outMs: number;
  durationMs: number;
  label: string;
  muted: boolean;
  transition: string;
}

export interface Subtitle {
  id: string;
  timelineId: string;
  startMs: number;
  endMs: number;
  text: string;
}

export interface Timeline {
  id: string;
  name: string;
  aspectRatio: string;
  width: number;
  height: number;
  fps: number;
  clips: TimelineClip[];
  subtitles: Subtitle[];
  musicAssetId?: string | null;
  musicVolume?: number;
  bpm?: number | null;
  beatGridMs?: number[];
  beatOffsetMs?: number;
  updatedAt: string;
}

export interface ProjectSnapshot {
  project: Project;
  characters: Character[];
  episodes: Episode[];
  assets: Asset[];
  jobs: GenerationJob[];
  timeline: Timeline;
  storagePath: string;
}

export interface ProviderCapabilities {
  aspectRatios: string[];
  durationsSec: number[];
  textToVideo: boolean;
  imageToVideo: boolean;
  referenceImage: boolean;
  maxReferenceImages: number;
  structuredOutput: boolean;
}

export interface ProviderProfile {
  id: string;
  kind: "llm" | "image" | "video" | "music";
  label: string;
  vendor: string;
  enabled: boolean;
  builtin: boolean;
  apiBase: string;
  defaultModel: string;
  models: string[];
  apiKeySet: boolean;
  envVar: string;
  options: Record<string, unknown>;
  capabilities: ProviderCapabilities;
}

export interface ProviderDefaults {
  llm: string;
  image: string;
  video: string;
}

export interface Settings {
  providers: ProviderProfile[];
  defaults: ProviderDefaults;
  concurrency: number;
  ffmpegPath: string | null;
  storageDir: string | null;
  uiLanguage: string;
}

export interface ContextAssetRef {
  id: string;
  name: string;
  absPath: string;
  role: string;
}

export interface ContextPreview {
  targetType: string;
  targetId: string;
  title: string;
  providerId: string;
  providerLabel: string;
  model: string;
  systemPrompt: string;
  userPrompt: string;
  negativePrompt: string;
  characterRefs: ContextAssetRef[];
  styleBible: StyleBible;
  warnings: string[];
  notes: string[];
  continuityFromShotId: string | null;
  continuityNote: string | null;
  estimatedCost: number;
  aspectRatio: string;
  durationMs: number;
  tokenEstimate: number;
}

export interface GenerationOptions {
  durationMs?: number | null;
  aspectRatio?: string | null;
  seed?: number | null;
  variant?: string | null;
  count?: number | null;
  extraPrompt?: string;
  useContinuity?: boolean;
  referenceAssetIds?: string[];
  resolution?: string | null;
}

export interface GenerationRequest {
  kind: "image" | "video" | "music";
  targetType: "shot" | "character" | "project" | "timeline";
  targetId: string | null;
  providerId?: string | null;
  model?: string | null;
  options: GenerationOptions;
  compiled?: ContextPreview | null;
}

export interface GenerateScriptRequest {
  premise: string;
  genre: string;
  tone: string;
  language: string;
  episodes: number;
  scenesPerEpisode: number;
  shotsPerScene: number;
  styleNotes: string;
  providerId?: string | null;
  model?: string | null;
  replaceExisting: boolean;
  seed?: number | null;
}

export interface CreateProjectInput {
  name: string;
  premise: string;
  genre: string;
  tone: string;
  language: string;
  aspectRatio: string;
  targetEpisodeSeconds: number;
  budgetCap?: number;
  stylePreset: string | null;
  styleBible?: StyleBible | null;
}

export interface UpdateProjectInput {
  name?: string;
  premise?: string;
  logline?: string;
  genre?: string;
  tone?: string;
  language?: string;
  aspectRatio?: string;
  targetEpisodeSeconds?: number;
  budgetCap?: number;
  styleBible?: StyleBible;
}

export interface UpdateSceneInput {
  title?: string;
  slugline?: string;
  location?: string;
  timeOfDay?: string;
  synopsis?: string;
  scriptText?: string;
  status?: string;
}

export interface UpdateShotInput {
  title?: string;
  description?: string;
  prompt?: string;
  negativePrompt?: string;
  camera?: string;
  lens?: string;
  durationMs?: number;
  dialogue?: string;
  sound?: string;
  characterIds?: string[];
  continuityFromShotId?: string;
  clearContinuity?: boolean;
  startFrameAssetId?: string;
  status?: string;
  providerId?: string;
  model?: string;
  seed?: number;
}

export interface CreateCharacterInput {
  name: string;
  role: string;
  description: string;
  appearance: string;
  personality: string;
  voice: string;
  lockedTraits: string[];
}

export interface UpdateCharacterInput {
  name?: string;
  role?: string;
  description?: string;
  appearance?: string;
  personality?: string;
  voice?: string;
  negativePrompt?: string;
  lockedTraits?: string[];
  referenceAssetId?: string;
  sheetAssetId?: string;
  seed?: number;
  providerId?: string;
  model?: string;
  status?: string;
  clearReferenceAsset?: boolean;
  clearSheetAsset?: boolean;
}

export interface TimelineClipInput {
  shotId?: string | null;
  assetId?: string | null;
  inMs: number;
  outMs: number;
  label: string;
  muted: boolean;
  transition: string;
}

export interface SubtitleInput {
  startMs: number;
  endMs: number;
  text: string;
}

export interface TimelinePatch {
  name?: string;
  aspectRatio?: string;
  width?: number;
  height?: number;
  fps?: number;
  clips?: TimelineClipInput[];
  subtitles?: SubtitleInput[];
  musicAssetId?: string | null;
  clearMusic?: boolean;
  musicVolume?: number;
  clearBeats?: boolean;
}

export interface ExportOptions {
  width: number;
  height: number;
  fps: number;
  burnSubtitles: boolean;
  musicAssetId?: string | null;
  filename: string;
}

export interface SystemStatus {
  ffmpegAvailable: boolean;
  ffmpegPath: string | null;
  ffmpegVersion: string | null;
  platform: string;
  appVersion: string;
  storageDir: string;
  projectCount: number;
  dataDir: string;
}

export interface ProviderTestResult {
  ok: boolean;
  message: string;
  latencyMs: number;
}

export interface ProjectReference {
  projectId: string;
  projectName: string;
  characters: Character[];
  styleBible: StyleBible;
}

export interface MusicTrack {
  id: string;
  title: string;
  artist: string;
  providerId: string;
  providerLabel: string;
  previewUrl: string;
  downloadUrl: string;
  durationMs: number;
  license: string;
  licenseUrl: string;
  requiresAttribution: boolean;
  tags: string[];
  coverUrl: string | null;
}

export interface MusicSearchResult {
  tracks: MusicTrack[];
  providerLabel: string;
  query: string;
  error?: string | null;
}

export interface BeatAnalysis {
  assetId: string;
  bpm: number;
  confidence: number;
  offsetMs: number;
  beatGridMs: number[];
  waveform: number[];
  durationMs: number;
}

export interface BeatSyncResult {
  alignedCuts: number;
  totalCuts: number;
  averageOffsetMs: number;
  timeline: Timeline;
}

export interface MusicGenerationRequest {
  prompt: string;
  durationMs: number;
  instrumental: boolean;
  providerId?: string | null;
  model?: string | null;
  attachToTimeline: boolean;
}

export interface ProviderPatch {
  enabled?: boolean;
  apiBase?: string;
  defaultModel?: string;
  models?: string[];
  options?: Record<string, unknown>;
}

export interface PreferencesPatch {
  defaults?: ProviderDefaults;
  concurrency?: number;
  ffmpegPath?: string;
  clearFfmpegPath?: boolean;
  uiLanguage?: string;
}

export interface Toast {
  id: string;
  kind: "info" | "success" | "error";
  title: string;
  message?: string;
}
