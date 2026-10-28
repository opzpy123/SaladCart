import type {
  Asset,
  BeatAnalysis,
  BeatSyncResult,
  Character,
  ContextPreview,
  CreateProjectInput,
  Episode,
  ExportOptions,
  GenerateScriptRequest,
  GenerationJob,
  GenerationRequest,
  MusicGenerationRequest,
  MusicSearchResult,
  MusicTrack,
  NovelAdaptRequest,
  NovelInput,
  NovelPlan,
  NovelSummary,
  PreferencesPatch,
  Project,
  ProjectReference,
  ProjectSnapshot,
  ProjectSummary,
  ProviderPatch,
  ProviderProfile,
  ProviderTestResult,
  Scene,
  Settings,
  Shot,
  ShotTake,
  StyleBible,
  SystemStatus,
  Timeline,
  TimelinePatch,
  UpdateCharacterInput,
  UpdateProjectInput,
  UpdateSceneInput,
  UpdateShotInput,
} from "./types";

const listeners = new Map<string, Set<(event: { payload: unknown }) => void>>();
const cancelled = new Set<string>();
const shotTakeLinks = new Map<string, Set<string>>();
const now = () => new Date().toISOString();
const uid = () => crypto.randomUUID();
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function rememberShotTake(shotId: string, assetId: string) {
  const links = shotTakeLinks.get(shotId) ?? new Set<string>();
  links.add(assetId);
  shotTakeLinks.set(shotId, links);
}

const STYLE: StyleBible = {
  visualStyle:
    "premium Chinese anime series, clean cel shading, expressive faces, detailed costumes, rain-soaked cinematic frames",
  palette: "warm sunlit neutrals, emerald and teal accents, vermilion highlights",
  lighting: "soft cinematic light, warm-cool separation, gentle bloom, wet reflections",
  cameraLanguage: "storyboard-driven compositions, 24–35mm equivalents, motivated camera moves",
  negativePrompt:
    "photorealistic, 3D render, deformed hands, inconsistent face, text, watermark, low resolution",
  referenceNotes:
    "Each character must keep identical hair, face shape, costume and color palette in every frame.",
};

interface ArtPalette {
  bg1: string;
  bg2: string;
  ink: string;
  accent: string;
  soft: string;
}

const PALETTES: ArtPalette[] = [
  { bg1: "#111a16", bg2: "#33472c", ink: "#e9f0d8", accent: "#b5d464", soft: "#75905a" },
  { bg1: "#1b1210", bg2: "#5c2d22", ink: "#f6e9dd", accent: "#e58b5b", soft: "#a76146" },
  { bg1: "#0f171d", bg2: "#1f3c4a", ink: "#e5f1ef", accent: "#74c6bd", soft: "#477f83" },
  { bg1: "#171222", bg2: "#3b3059", ink: "#f1eaf7", accent: "#c3a3e8", soft: "#7f6ba5" },
];

function esc(value: string): string {
  return value.replace(/[<>&"']/g, (c) => ({ "<": "&lt;", ">": "&gt;", "&": "&amp;", '"': "&quot;", "'": "&apos;" })[c] ?? c);
}

function art(title: string, subtitle: string, index: number, kind: "character" | "shot" | "sheet" = "shot"): string {
  const p = PALETTES[index % PALETTES.length];
  const lines = title.split("\n");
  const rain =
    kind === "shot"
      ? Array.from({ length: 34 }, (_, i) => {
          const x = (i * 97 + index * 31) % 900;
          return `<line x1="${x}" y1="${(i * 53) % 500}" x2="${x - 26}" y2="${((i * 53) % 500) + 110}" stroke="${p.ink}" stroke-opacity="0.08" stroke-width="2"/>`;
        }).join("")
      : "";
  const portrait =
    kind === "character" || kind === "sheet"
      ? `<circle cx="450" cy="430" r="165" fill="${p.bg1}" stroke="${p.accent}" stroke-width="5" stroke-opacity="0.65"/>
         <ellipse cx="450" cy="400" rx="108" ry="138" fill="#101410" opacity="0.92"/>
         <path d="M250 760 Q270 590 450 575 Q630 590 650 760 Z" fill="#101410" opacity="0.95"/>
         <path d="M342 398 Q450 480 558 398" fill="none" stroke="${p.accent}" stroke-width="4" opacity="0.7"/>
         <path d="M402 420 L438 450 M498 420 L462 450" stroke="${p.ink}" stroke-width="3" opacity="0.4"/>`
      : "";
  const sheet =
    kind === "sheet"
      ? `<g opacity="0.5" stroke="${p.accent}" stroke-width="2" fill="none">
           <rect x="110" y="750" width="150" height="210" rx="12"/>
           <rect x="290" y="750" width="150" height="210" rx="12"/>
           <rect x="470" y="750" width="150" height="210" rx="12"/>
           <rect x="650" y="750" width="150" height="210" rx="12"/>
         </g>`
      : "";
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="900" height="1200" viewBox="0 0 900 1200">
    <defs>
      <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="${p.bg1}"/><stop offset="1" stop-color="${p.bg2}"/>
      </linearGradient>
      <radialGradient id="glow" cx="55%" cy="38%" r="60%">
        <stop offset="0" stop-color="${p.accent}" stop-opacity="0.38"/>
        <stop offset="1" stop-color="${p.accent}" stop-opacity="0"/>
      </radialGradient>
      <filter id="grain"><feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2" stitchTiles="stitch"/><feColorMatrix type="saturate" values="0"/><feComponentTransfer><feFuncA type="linear" slope="0.07"/></feComponentTransfer><feBlend in="SourceGraphic" mode="overlay"/></filter>
    </defs>
    <rect width="900" height="1200" fill="url(#g)"/>
    <rect width="900" height="1200" fill="url(#glow)"/>
    ${rain}
    ${portrait}
    ${sheet}
    <g opacity="0.8" fill="${p.ink}" font-family="Helvetica Neue, Arial, sans-serif">
      <text x="56" y="92" font-size="22" letter-spacing="5">SALADCART · PROJECT BIBLE</text>
      <text x="56" y="124" font-size="15" fill="${p.soft}" letter-spacing="3">${esc(kind.toUpperCase())} / MOCK PREVIEW</text>
    </g>
    <g>
      <rect x="0" y="980" width="900" height="220" fill="#0d110d" opacity="0.78"/>
      <text x="56" y="1060" fill="${p.ink}" font-family="Iowan Old Style, Songti SC, Georgia, serif" font-size="58" font-weight="600">${esc(lines[0] ?? "")}</text>
      <text x="56" y="1116" fill="${p.accent}" font-family="Helvetica Neue, Arial, sans-serif" font-size="22" letter-spacing="2">${esc(subtitle)}</text>
      <text x="56" y="1158" fill="${p.soft}" font-family="Helvetica Neue, Arial, sans-serif" font-size="16" letter-spacing="1">MOCK ARTWORK · REPLACE WITH YOUR GENERATED TAKE</text>
    </g>
    <rect width="900" height="1200" filter="url(#grain)" opacity="0.5" fill="transparent"/>
  </svg>`;
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}

function makeAsset(
  id: string,
  kind: Asset["kind"],
  name: string,
  path: string,
  artwork: string | null,
  options: Partial<Asset> = {},
): Asset {
  return {
    id,
    kind,
    name,
    relPath: path,
    absPath: path,
    thumbPath: artwork,
    mime: kind === "video" ? "video/mp4" : kind === "audio" ? "audio/wav" : "image/svg+xml",
    width: kind === "video" ? 1080 : 900,
    height: kind === "video" ? 1920 : 1200,
    durationMs: kind === "video" ? 5000 : kind === "audio" ? 24000 : null,
    sizeBytes: 1_500_000,
    source: "generated",
    providerId: "mock-video",
    model: "saladcart-mock-video",
    prompt: null,
    negativePrompt: null,
    seed: 42,
    version: 1,
    parentAssetId: null,
    license: null,
    licenseUrl: null,
    attribution: null,
    sourceUrl: null,
    status: "ready",
    createdAt: now(),
    ...options,
  };
}

function makeCharacter(
  id: string,
  name: string,
  role: string,
  description: string,
  appearance: string,
  traits: string[],
  artIndex: number,
): Character {
  const ref = makeAsset(`asset-${id}-ref`, "image", `${name} · 定妆照`, art(name, role, artIndex, "character"), null, {
    source: "generated",
    providerId: "mock-image",
    model: "saladcart-mock-image",
  });
  const sheet = makeAsset(`asset-${id}-sheet`, "image", `${name} · 三视图`, art(name, "CHARACTER SHEET", artIndex, "sheet"), null, {
    providerId: "mock-image",
    model: "saladcart-mock-image",
  });
  assets.push(ref, sheet);
  return {
    id,
    name,
    role,
    description,
    appearance,
    personality: "",
    voice: "",
    negativePrompt: "",
    lockedTraits: traits,
    referenceAssetId: ref.id,
    sheetAssetId: sheet.id,
    seed: 42 + artIndex,
    providerId: "mock-image",
    model: "saladcart-mock-image",
    status: "ready",
    createdAt: now(),
    updatedAt: now(),
  };
}

const assets: Asset[] = [];
const characters: Character[] = [
  makeCharacter(
    "char-shen",
    "沈知遥",
    "主角 / 记忆修复师",
    "24岁，克制、敏锐，习惯先观察再行动。她能触碰旧物读到残留情绪，但每次读取都会失去一段自己的记忆。",
    "冷白皮，黑色及肩短发，左侧一绺银灰挑染，琥珀色眼睛；深灰高领风衣、旧铜色耳夹，右手缠白色绷带。",
    ["黑色及肩短发", "左侧银灰挑染", "琥珀色眼睛", "深灰高领风衣", "右腕白色绷带"],
    0,
  ),
  makeCharacter(
    "char-gu",
    "顾临川",
    "对手 / 调查记者",
    "28岁，表面散漫，实际把每一条线索都记在心里。他追踪同一桩失踪案，却从不解释自己的动机。",
    "深色微卷短发，眉骨有浅疤，灰绿色眼睛；卡其色短夹克、黑色高领衫，随身带一台老式录音机。",
    ["深色微卷短发", "眉骨浅疤", "灰绿色眼睛", "卡其色短夹克", "老式录音机"],
    1,
  ),
  makeCharacter(
    "char-ruo",
    "阿箬",
    "伙伴 / 旧物店老板",
    "52岁，经营一家只在雨天开门的旧物店，知道城市里被刻意遗忘的事，但从不直接说破。",
    "银灰盘发，细框眼镜，深青色盘扣外套，手边总有一盏黄铜台灯。",
    ["银灰盘发", "细框眼镜", "深青色盘扣外套", "黄铜台灯", "旧铜钥匙串"],
    2,
  ),
];

const shotSpecs = [
  {
    title: "钩子：雨中的回声",
    description:
      "开场3秒钩子：一枚旧铜钥匙在积水里震动，沈知遥停住脚步。远处站牌灯灭的瞬间，顾临川的身影从玻璃反光中闪过。",
    camera: "slow push-in",
    lens: "35mm",
    dialogue: "沈知遥（低声）：……它在叫我的名字。",
    sound: "雨声突然抽空，只剩钥匙的高频震动",
    cast: ["char-shen", "char-gu"],
    status: "approved",
    artIndex: 2,
  },
  {
    title: "旧物开口",
    description:
      "沈知遥戴上绷带，指尖触到钥匙。画面边缘浮起不属于现在的记忆碎片，阿箬在门口抬手关掉一半的灯。",
    camera: "handheld follow",
    lens: "28mm",
    dialogue: "阿箬：你每读一次，就少一段自己的。\n沈知遥：那我就先记住今天。",
    sound: "台灯电流声、远处列车进站",
    cast: ["char-shen", "char-ruo"],
    status: "needs_review",
    artIndex: 0,
  },
  {
    title: "对质",
    description:
      "顾临川把录音机放在桌上，按下播放。磁带里传来沈知遥自己的声音，说出一个她从未去过的地址。",
    camera: "shot / reverse shot",
    lens: "50mm",
    dialogue: "顾临川：你不是忘了。你是被人从记忆里剪掉了。",
    sound: "磁带底噪、雨点敲铁皮",
    cast: ["char-shen", "char-gu"],
    status: "needs_review",
    artIndex: 1,
  },
  {
    title: "追到站台",
    description:
      "沈知遥冲进雨夜车站，灯光一盏盏熄灭。她在最后一节车厢玻璃上看见童年的自己。",
    camera: "lateral tracking",
    lens: "24mm",
    dialogue: "沈知遥（喘息）：别关门——",
    sound: "列车制动、急促脚步",
    cast: ["char-shen"],
    status: "draft",
    artIndex: 3,
  },
];

const shotArtAssets: Asset[] = shotSpecs.map((spec, i) =>
  makeAsset(`asset-shot-${i + 1}-image`, "image", `${spec.title} · 画面`, art(spec.title, "KEYFRAME / MOCK", spec.artIndex), null, {
    providerId: "mock-image",
    model: "saladcart-mock-image",
  }),
);
const videoPaths = ["/demo/clip-a.mp4", "/demo/clip-b.mp4", "/demo/clip-a.mp4", "/demo/clip-b.mp4"];
const videoAssets: Asset[] = shotSpecs.map((spec, i) =>
  makeAsset(`asset-shot-${i + 1}-video`, "video", `${spec.title} · 镜头成片`, videoPaths[i], art(spec.title, "SHOT / MOCK FOOTAGE", spec.artIndex), {
    providerId: "mock-video",
    model: "saladcart-mock-video",
    width: 720,
    height: 1280,
    durationMs: 5000,
    status: i < 2 ? "approved" : "ready",
  }),
);
const musicAsset = makeAsset("asset-score", "audio", "雨幕之下 · 主题配乐", "/demo/score.wav", null, {
  providerId: "mock-music",
  model: "saladcart-mock-music",
  mime: "audio/wav",
  durationMs: 24000,
  width: null,
  height: null,
  sizeBytes: 2_116_844,
});
assets.push(...shotArtAssets, ...videoAssets, musicAsset);

const shots: Shot[] = shotSpecs.map((spec, i) => ({
  id: `shot-${i + 1}`,
  sceneId: i < 2 ? "scene-1" : "scene-2",
  orderIndex: i < 2 ? i : i - 2,
  title: spec.title,
  description: spec.description,
  prompt: `${spec.description} ${spec.camera}, ${spec.lens}, cinematic Chinese anime, rain-soaked night, emerald and vermilion palette, no text.`,
  negativePrompt: STYLE.negativePrompt,
  camera: spec.camera,
  lens: spec.lens,
  durationMs: 5000,
  dialogue: spec.dialogue,
  sound: spec.sound,
  characterIds: spec.cast,
  continuityFromShotId: i > 0 ? `shot-${i}` : null,
  startFrameAssetId: null,
  imageAssetId: shotArtAssets[i].id,
  videoAssetId: i < 3 ? videoAssets[i].id : null,
  voiceAssetId: null,
  status: spec.status,
  providerId: i < 3 ? "mock-video" : "",
  model: i < 3 ? "saladcart-mock-video" : "",
  seed: 100 + i,
  version: i < 3 ? 1 : 0,
  createdAt: now(),
  updatedAt: now(),
}));

const scenes: Scene[] = [
  {
    id: "scene-1",
    episodeId: "ep-1",
    title: "第1场 · 旧城区天台",
    slugline: "EXT. 旧城区天台 - NIGHT",
    location: "旧城区天台",
    timeOfDay: "NIGHT",
    synopsis: "一枚没有主人的铜钥匙在雨里震动，沈知遥第一次听见自己的过去。顾临川在玻璃反光中现身。",
    scriptText:
      "雨水沿着铁皮屋檐落下。沈知遥停在天台边缘，积水里的旧铜钥匙发出细微的震动。\n\n远处站牌灯灭。玻璃反光里，顾临川的身影一闪而过。\n\n沈知遥（低声）：……它在叫我的名字。",
    orderIndex: 0,
    status: "ready",
    shots: shots.slice(0, 2),
    createdAt: now(),
    updatedAt: now(),
  },
  {
    id: "scene-2",
    episodeId: "ep-1",
    title: "第2场 · 雨夜车站",
    slugline: "EXT. 雨夜车站 - NIGHT",
    location: "雨夜车站",
    timeOfDay: "NIGHT",
    synopsis: "顾临川播放一段不该存在的录音。沈知遥追向最后一班列车，在玻璃上看见童年的自己。",
    scriptText:
      "顾临川把录音机放在长椅上，按下播放。磁带里传来沈知遥自己的声音。\n\n顾临川：你不是忘了。你是被人从记忆里剪掉了。\n\n列车进站，灯光一盏盏熄灭。沈知遥冲向最后一节车厢。",
    orderIndex: 1,
    status: "draft",
    shots: shots.slice(2),
    createdAt: now(),
    updatedAt: now(),
  },
];

const episodes: Episode[] = [
  {
    id: "ep-1",
    title: "第1集 · 雨中的钥匙",
    summary: "沈知遥触碰一枚旧铜钥匙，听见自己的声音。顾临川带着十年前的录音出现，阿箬则要求她立刻停下。",
    orderIndex: 0,
    scenes,
    createdAt: now(),
    updatedAt: now(),
  },
];

const timelineClips = [0, 1, 2].map((i) => ({
  id: `clip-${i + 1}`,
  timelineId: "timeline-1",
  shotId: shots[i].id,
  assetId: videoAssets[i].id,
  orderIndex: i,
  inMs: 0,
  outMs: 5000,
  durationMs: 5000,
  label: `第${i < 2 ? 1 : 2}场 · ${shots[i].title}`,
  muted: false,
  transition: "cut",
}));

const timeline: Timeline = {
  id: "timeline-1",
  name: "Main cut",
  aspectRatio: "9:16",
  width: 1080,
  height: 1920,
  fps: 30,
  clips: timelineClips,
  subtitles: [
    { id: "sub-1", timelineId: "timeline-1", startMs: 300, endMs: 3100, text: "沈知遥：……它在叫我的名字。" },
    { id: "sub-2", timelineId: "timeline-1", startMs: 5300, endMs: 7600, text: "阿箬：你每读一次，就少一段自己的。" },
    { id: "sub-3", timelineId: "timeline-1", startMs: 7900, endMs: 10800, text: "沈知遥：那我就先记住今天。" },
    { id: "sub-4", timelineId: "timeline-1", startMs: 11200, endMs: 14000, text: "顾临川：你是被人从记忆里剪掉了。" },
  ],
  musicAssetId: musicAsset.id,
  musicVolume: 0.8,
  bpm: 100,
  beatGridMs: Array.from({ length: 41 }, (_, i) => i * 600),
  beatOffsetMs: 0,
  updatedAt: now(),
};

const project: Project = {
  id: "demo-fog-harbor",
  name: "雾港来信",
  premise:
    "在常年落雨的海临城，一个能触碰旧物读取记忆的女孩，发现一枚铜钥匙里存着自己的声音，而每次读取都会让她失去一段记忆。",
  logline: "她靠别人的过去谋生，却在一枚旧钥匙里听见了自己被剪掉的现在。",
  genre: "悬疑 / 都市奇幻",
  tone: "冷冽、诗意、克制的浪漫",
  language: "zh-CN",
  aspectRatio: "9:16",
  targetEpisodeSeconds: 90,
  budgetCap: 0,
  styleBible: STYLE,
  createdAt: now(),
  updatedAt: now(),
};

const pastJobs: GenerationJob[] = [
  aiJob("script", "project", project.id, "mock-llm", "Mock 编剧 (离线)", "saladcart-mock-llm", "成功生成 1 集 / 2 场 / 4 镜", 100),
  aiJob("video", "shot", "shot-1", "mock-video", "Mock 动画 (离线)", "saladcart-mock-video", "钩子：雨中的回声 · 镜头成片", 100),
  aiJob("image", "character", "char-shen", "mock-image", "Mock 画师 (离线)", "saladcart-mock-image", "沈知遥 · 角色定妆照", 100),
];

function aiJob(
  kind: string,
  targetType: string,
  targetId: string | null,
  providerId: string,
  providerLabel: string,
  model: string,
  requestSummary: string,
  progress: number,
): GenerationJob {
  return {
    id: uid(),
    projectId: project.id,
    kind,
    targetType,
    targetId,
    providerId,
    providerLabel,
    model,
    status: "succeeded",
    progress,
    stage: "done",
    error: null,
    costEstimate: 0,
    costActual: 0,
    requestSummary,
    request: {},
    result: {},
    resultAssetId: null,
    createdAt: new Date(Date.now() - 1000 * 60 * 60).toISOString(),
    startedAt: new Date(Date.now() - 1000 * 60 * 60).toISOString(),
    finishedAt: new Date(Date.now() - 1000 * 60 * 59).toISOString(),
  };
}

const secondProject: ProjectSnapshot | null = null;
let novelText = "";
let snapshot: ProjectSnapshot = {
  project,
  characters,
  episodes,
  assets,
  jobs: pastJobs,
  timeline,
  novel: null,
  storagePath: "~/Library/Application Support/com.saladcart.studio/projects/demo-fog-harbor",
};
let projects: ProjectSummary[] = [summaryOf(snapshot)];

function summaryOf(s: ProjectSnapshot): ProjectSummary {
  return {
    id: s.project.id,
    name: s.project.name,
    logline: s.project.logline,
    premise: s.project.premise,
    genre: s.project.genre,
    aspectRatio: s.project.aspectRatio,
    updatedAt: s.project.updatedAt,
    coverPath: s.characters[0]?.referenceAssetId
      ? s.assets.find((a) => a.id === s.characters[0].referenceAssetId)?.absPath ?? null
      : null,
    storagePath: s.storagePath,
    counts: {
      characters: s.characters.length,
      episodes: s.episodes.length,
      scenes: s.episodes.reduce((n, e) => n + e.scenes.length, 0),
      shots: s.episodes.reduce((n, e) => n + e.scenes.reduce((m, x) => m + x.shots.length, 0), 0),
      videos: s.assets.filter((a) => a.kind === "video").length,
      images: s.assets.filter((a) => a.kind === "image").length,
    },
  };
}

function touch() {
  snapshot.project.updatedAt = now();
  const index = projects.findIndex((p) => p.id === snapshot.project.id);
  const summary = summaryOf(snapshot);
  if (index >= 0) projects[index] = summary;
  else projects.unshift(summary);
}

function emit(event: string, payload: unknown) {
  listeners.get(event)?.forEach((callback) => callback({ payload }));
}

function emitJob(job: GenerationJob) {
  const index = snapshot.jobs.findIndex((j) => j.id === job.id);
  if (index >= 0) snapshot.jobs[index] = { ...job };
  else snapshot.jobs.unshift({ ...job });
  emit("job:update", { ...job });
}

async function simulate(
  job: GenerationJob,
  stages: Array<[number, string]>,
  finish: () => void,
) {
  for (const [progress, stage] of stages) {
    await sleep(420);
    if (cancelled.has(job.id)) return;
    job.status = "running";
    job.progress = progress;
    job.stage = stage;
    job.startedAt ??= now();
    emitJob(job);
  }
  await sleep(700);
  if (cancelled.has(job.id)) return;
  finish();
  job.status = "succeeded";
  job.progress = 100;
  job.stage = "done";
  job.finishedAt = now();
  touch();
  emitJob(job);
  emit("project:changed", { projectId: job.projectId });
}

function newJob(
  kind: string,
  targetType: string,
  targetId: string | null,
  providerId: string,
  providerLabel: string,
  model: string,
  requestSummary: string,
  request: Record<string, unknown>,
  costEstimate = 0,
): GenerationJob {
  const job: GenerationJob = {
    id: uid(),
    projectId: snapshot.project.id,
    kind,
    targetType,
    targetId,
    providerId,
    providerLabel,
    model,
    status: "queued",
    progress: 0,
    stage: "queued",
    error: null,
    costEstimate,
    costActual: 0,
    requestSummary,
    request,
    result: null,
    resultAssetId: null,
    createdAt: now(),
    startedAt: null,
    finishedAt: null,
  };
  emitJob(job);
  return job;
}

const settings: Settings = {
  providers: [
    provider("mock-llm", "llm", "Mock 编剧 (离线)", "local://mock", "saladcart-mock-llm", true, false, "", { structuredOutput: true }),
    provider("openai-llm", "llm", "OpenAI / 兼容接口", "https://api.openai.com/v1", "gpt-4.1-mini", true, false, "OPENAI_API_KEY", { structuredOutput: true }),
    provider("deepseek-llm", "llm", "DeepSeek", "https://api.deepseek.com/v1", "deepseek-chat", true, false, "DEEPSEEK_API_KEY", { structuredOutput: true }),
    provider("qwen-llm", "llm", "通义千问 / DashScope", "https://dashscope.aliyuncs.com/compatible-mode/v1", "qwen-plus", true, false, "DASHSCOPE_API_KEY", { structuredOutput: true }),
    provider("ollama-llm", "llm", "Ollama (本地)", "http://127.0.0.1:11434/v1", "qwen2.5:7b", true, false, "", { structuredOutput: true }),
    provider("mock-image", "image", "Mock 画师 (离线)", "local://mock", "saladcart-mock-image", true, false, "", { referenceImage: true, aspectRatios: ["9:16", "16:9", "1:1"], maxReferenceImages: 4 }),
    provider("openai-image", "image", "OpenAI Images", "https://api.openai.com/v1", "gpt-image-1", true, false, "OPENAI_API_KEY", { referenceImage: true, aspectRatios: ["9:16", "16:9", "1:1"], maxReferenceImages: 4 }),
    provider("gemini-image", "image", "Gemini Image", "https://generativelanguage.googleapis.com/v1beta", "gemini-2.5-flash-image", true, false, "GEMINI_API_KEY", { referenceImage: true, aspectRatios: ["9:16", "16:9", "1:1"], maxReferenceImages: 4 }),
    provider("replicate-image", "image", "Replicate", "https://api.replicate.com/v1", "black-forest-labs/flux-schnell", true, false, "REPLICATE_API_TOKEN", { aspectRatios: ["9:16", "16:9", "1:1"] }),
    provider("comfyui-image", "image", "ComfyUI (本地)", "http://127.0.0.1:8188", "default", true, false, "", {}),
    provider("mock-video", "video", "Mock 动画 (离线)", "local://mock", "saladcart-mock-video", true, false, "", { textToVideo: true, imageToVideo: true, aspectRatios: ["9:16", "16:9", "1:1"], durationsSec: [3, 5, 8], referenceImage: true, maxReferenceImages: 1 }),
    provider("kling-video", "video", "可灵 Kling", "https://api-singapore.klingai.com", "kling-v2-1", true, false, "KLING_API_KEY", { textToVideo: true, imageToVideo: true, aspectRatios: ["9:16", "16:9", "1:1"], durationsSec: [5, 10], referenceImage: true, maxReferenceImages: 1 }),
    provider("runway-video", "video", "Runway", "https://api.dev.runwayml.com", "gen4_turbo", true, false, "RUNWAY_API_KEY", { textToVideo: true, imageToVideo: true, aspectRatios: ["9:16", "16:9", "1:1"], durationsSec: [5, 10], referenceImage: true, maxReferenceImages: 1 }),
    provider("luma-video", "video", "Luma Dream Machine", "https://api.lumalabs.ai", "ray-2", true, false, "LUMA_API_KEY", { textToVideo: true, imageToVideo: true, aspectRatios: ["9:16", "16:9", "1:1"], durationsSec: [5, 10], referenceImage: true, maxReferenceImages: 1 }),
    provider("minimax-video", "video", "MiniMax 海螺", "https://api.minimax.io", "MiniMax-Hailuo-02", true, false, "MINIMAX_API_KEY", { textToVideo: true, imageToVideo: true, aspectRatios: ["9:16", "16:9"], durationsSec: [6, 10], referenceImage: true, maxReferenceImages: 1 }),
    provider("replicate-video", "video", "Replicate Video", "https://api.replicate.com/v1", "minimax/hailuo-02", true, false, "REPLICATE_API_TOKEN", { textToVideo: true, imageToVideo: true, aspectRatios: ["9:16", "16:9"], durationsSec: [5, 10], referenceImage: true, maxReferenceImages: 1 }),
    provider("veo-video", "video", "Google Veo", "https://generativelanguage.googleapis.com/v1beta", "veo-3.1-generate-preview", true, false, "GEMINI_API_KEY", { textToVideo: true, imageToVideo: true, aspectRatios: ["9:16", "16:9"], durationsSec: [4, 6, 8], referenceImage: true, maxReferenceImages: 1 }),
    provider("fal-video", "video", "fal.ai Video", "https://queue.fal.run", "fal-ai/kling-video/v2.5-turbo/pro/image-to-video", true, false, "FAL_KEY", { textToVideo: true, imageToVideo: true, aspectRatios: ["9:16", "16:9", "1:1"], durationsSec: [5, 10], referenceImage: true, maxReferenceImages: 1 }),
    provider("mock-music", "music", "Mock 配乐 (离线)", "local://mock", "saladcart-mock-music", true, false, "", {}),
    provider("jamendo-music", "music", "Jamendo 音乐库", "https://api.jamendo.com/v3.0", "search", true, false, "JAMENDO_CLIENT_ID", {}),
    provider("stability-audio", "music", "Stability Audio", "https://api.stability.ai", "stable-audio-3", true, false, "STABILITY_API_KEY", {}),
    provider("elevenlabs-music", "music", "ElevenLabs Music", "https://api.elevenlabs.io", "music", true, false, "ELEVENLABS_API_KEY", {}),
  ],
  defaults: { llm: "mock-llm", image: "mock-image", video: "mock-video" },
  concurrency: 2,
  ffmpegPath: "/opt/homebrew/bin/ffmpeg",
  storageDir: null,
  uiLanguage: "zh-CN",
};

function provider(
  id: string,
  kind: ProviderProfile["kind"],
  label: string,
  apiBase: string,
  defaultModel: string,
  enabled: boolean,
  apiKeySet: boolean,
  envVar: string,
  capabilities: Partial<ProviderProfile["capabilities"]>,
): ProviderProfile {
  return {
    id,
    kind,
    label,
    vendor: id.split("-")[0],
    enabled,
    builtin: true,
    apiBase,
    defaultModel,
    models: [defaultModel],
    apiKeySet,
    envVar,
    options: {},
    capabilities: {
      aspectRatios: [],
      durationsSec: [],
      textToVideo: false,
      imageToVideo: false,
      referenceImage: false,
      maxReferenceImages: 0,
      structuredOutput: false,
      ...capabilities,
    },
  };
}

function findShot(shotId: string): Shot | undefined {
  for (const episode of snapshot.episodes) {
    for (const scene of episode.scenes) {
      const shot = scene.shots.find((s) => s.id === shotId);
      if (shot) return shot;
    }
  }
  return undefined;
}

function findScene(sceneId: string): Scene | undefined {
  for (const episode of snapshot.episodes) {
    const scene = episode.scenes.find((s) => s.id === sceneId);
    if (scene) return scene;
  }
  return undefined;
}

function updateShotRecord(shotId: string, patch: Partial<Shot>) {
  for (const episode of snapshot.episodes) {
    for (const scene of episode.scenes) {
      scene.shots = scene.shots.map((shot) => (shot.id === shotId ? { ...shot, ...patch, updatedAt: now() } : shot));
    }
  }
}

function contextFor(request: GenerationRequest): ContextPreview {
  const profile =
    settings.providers.find((p) => p.id === (request.providerId || (request.kind === "video" ? settings.defaults.video : settings.defaults.image))) ??
    settings.providers[0];
  if (request.targetType === "character") {
    const character = snapshot.characters.find((c) => c.id === request.targetId);
    if (!character) throw new Error("角色不存在");
    const ref = character.referenceAssetId ? snapshot.assets.find((a) => a.id === character.referenceAssetId) : null;
    return {
      targetType: "character",
      targetId: character.id,
      title: character.name,
      providerId: profile.id,
      providerLabel: profile.label,
      model: profile.defaultModel,
      systemPrompt: "You are a character designer for a premium AI comic drama.",
      userPrompt: `[PROJECT STYLE]\n${snapshot.project.styleBible.visualStyle}\nPalette: ${snapshot.project.styleBible.palette}\n\n[CHARACTER BIBLE]\nName: ${character.name}\nRole: ${character.role}\nAppearance: ${character.appearance}\nIdentity locks: ${character.lockedTraits.join(", ")}\n\n[OUTPUT]\nCharacter turnaround sheet, neutral background, same costume and face.`,
      negativePrompt: snapshot.project.styleBible.negativePrompt,
      characterRefs: ref ? [{ id: ref.id, name: `${character.name} · 定妆照`, absPath: ref.absPath, role: "existing_reference" }] : [],
      styleBible: snapshot.project.styleBible,
      warnings: character.lockedTraits.length ? [] : ["角色还没有锁定外观特征。"],
      notes: ["Mock 模式在本地生成占位画面，不消耗额度。"],
      continuityFromShotId: null,
      continuityNote: null,
      estimatedCost: 0,
      aspectRatio: request.options.aspectRatio || snapshot.project.aspectRatio,
      durationMs: 0,
      tokenEstimate: Math.max(
        1,
        Math.round(
          (character.appearance.length +
            character.lockedTraits.join("").length +
            snapshot.project.styleBible.visualStyle.length +
            180) /
            2,
        ),
      ),
    };
  }
  const shot = findShot(request.targetId ?? "");
  if (!shot) throw new Error("镜头不存在");
  const scene = findScene(shot.sceneId);
  const cast = snapshot.characters.filter((c) => shot.characterIds.includes(c.id));
  const refs = cast.flatMap((c) => {
    const ref = c.referenceAssetId ? snapshot.assets.find((a) => a.id === c.referenceAssetId) : null;
    return ref ? [{ id: ref.id, name: `${c.name} · 定妆照`, absPath: ref.absPath, role: "character_reference" }] : [];
  });
  const prior = shot.continuityFromShotId ? findShot(shot.continuityFromShotId) : null;
  const prompt = [
    `[PROJECT STYLE]\n${snapshot.project.styleBible.visualStyle}\nPalette: ${snapshot.project.styleBible.palette}\nLighting: ${snapshot.project.styleBible.lighting}`,
    `[SCENE]\n${scene?.title ?? ""} — ${scene?.slugline ?? ""}\n${scene?.synopsis ?? ""}`,
    `[SHOT]\n${shot.title} — ${shot.description}\nCamera: ${shot.camera}. Lens: ${shot.lens}. Duration: ${Math.round(shot.durationMs / 1000)}s.`,
    `[CAST — identity locked]\n${cast.map((c) => `- ${c.name}: ${c.appearance}. Locks: ${c.lockedTraits.join(", ")}`).join("\n")}`,
    prior ? `[CONTINUITY]\nContinue from “${prior.title}”. Match costume, lighting and screen direction.` : "",
    request.options.extraPrompt ? `[EXTRA DIRECTION]\n${request.options.extraPrompt}` : "",
    `[OUTPUT]\n${request.options.aspectRatio || snapshot.project.aspectRatio}, ${Math.round((request.options.durationMs || shot.durationMs) / 1000)} seconds, coherent motion, stable face, no on-screen text.`,
  ]
    .filter(Boolean)
    .join("\n\n");
  return {
    targetType: "shot",
    targetId: shot.id,
    title: `${scene?.title ?? ""} · ${shot.title}`,
    providerId: profile.id,
    providerLabel: profile.label,
    model: profile.defaultModel,
    systemPrompt: "You are the supervising art director of a premium AI comic drama.",
    userPrompt: prompt,
    negativePrompt: [snapshot.project.styleBible.negativePrompt, shot.negativePrompt].filter(Boolean).join(", "),
    characterRefs: refs,
    styleBible: snapshot.project.styleBible,
    warnings: [
      ...cast.filter((c) => !c.referenceAssetId).map((c) => `${c.name} 还没有定妆照，跨镜头一致性无法保证。`),
      ...(cast.length ? [] : ["这个镜头没有绑定角色。"]),
    ],
    notes: [
      "Mock 模式在本地生成占位素材，不消耗额度。",
      ...(prior?.videoAssetId ? ["生成时可自动提取上一镜头尾帧作为首帧。"] : []),
    ],
    continuityFromShotId: prior?.id ?? null,
    continuityNote: prior ? `从「${prior.title}」的尾帧续接。` : null,
    estimatedCost: 0,
    aspectRatio: request.options.aspectRatio || snapshot.project.aspectRatio,
    durationMs: request.options.durationMs || shot.durationMs,
    tokenEstimate: Math.max(1, Math.round((prompt.length + 180) / 2)),
  };
}

function generateScriptContent(request: GenerateScriptRequest) {
  const premise = request.premise || snapshot.project.premise || "一个能听见旧物记忆的女孩，追查自己失踪的过去。";
  const title = premise.slice(0, 12) + "…";
  const cast =
    snapshot.characters.length > 0
      ? snapshot.characters
      : characters.map((c) => ({ ...c, id: uid(), referenceAssetId: c.referenceAssetId, sheetAssetId: c.sheetAssetId }));
  const newShotSpecs = shotSpecs.slice(0, Math.min(4, request.shotsPerScene * 2));
  const newScenes: Scene[] = [0, 1].slice(0, Math.max(1, Math.min(2, request.scenesPerEpisode))).map((sceneIndex) => {
    const sceneShots = newShotSpecs.map((spec, i) => ({
      ...shots[i],
      id: uid(),
      sceneId: "",
      characterIds: cast.slice(0, 2).map((c) => c.id),
      imageAssetId: null,
      videoAssetId: null,
      status: "draft",
      providerId: "",
      model: "",
      version: 0,
      dialogue: spec.dialogue,
      description: spec.description,
    }));
    return {
      ...scenes[sceneIndex],
      id: uid(),
      title: `第${sceneIndex + 1}场 · ${sceneIndex === 0 ? "雨中的钥匙" : "被剪掉的人"}`,
      synopsis: `${premise} ${sceneIndex === 0 ? "沈知遥第一次听见钥匙里的声音。" : "顾临川带来一段不该存在的录音。"}`,
      shots: sceneShots,
      status: "draft",
    };
  });
  newScenes.forEach((scene) => scene.shots.forEach((shot) => (shot.sceneId = scene.id)));
  snapshot.episodes = [
    {
      ...episodes[0],
      id: uid(),
      title: `第1集 · ${title}`,
      summary: premise,
      scenes: newScenes,
    },
  ];
  snapshot.project.name = snapshot.project.name === "未命名漫剧" || !snapshot.project.name ? title : snapshot.project.name;
  snapshot.project.logline = premise;
  snapshot.project.premise = premise;
  if (snapshot.characters.length === 0) snapshot.characters = cast as Character[];
}

function splitMockNovel(text: string): Array<{ title: string; text: string }> {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  const chapters: Array<{ title: string; text: string }> = [];
  let title = "";
  let body: string[] = [];
  const flush = () => {
    const content = body.join("\n").trim();
    if (content.length < 2) {
      body = [];
      return;
    }
    chapters.push({ title: title || `第${chapters.length + 1}节`, text: content });
    body = [];
  };
  for (const line of lines) {
    const trimmed = line.trim();
    const heading =
      trimmed.length > 0 &&
      trimmed.length <= 60 &&
      (/^第[0-9一二三四五六七八九十百千]+[章回节卷]/.test(trimmed) ||
        /^chapter\s+\d+/i.test(trimmed) ||
        trimmed.startsWith("#") ||
        trimmed.startsWith("序章") ||
        trimmed.startsWith("楔子"));
    if (heading) {
      flush();
      title = trimmed.replace(/^#+\s*/, "");
    } else {
      body.push(line);
    }
  }
  flush();
  if (!chapters.length && text.trim()) chapters.push({ title: "全文", text: text.trim() });
  return chapters;
}

function novelPlan(text: string): NovelPlan {
  const chapters = splitMockNovel(text);
  const chunkCount = chapters.reduce((sum, chapter) => sum + Math.max(1, Math.ceil(chapter.text.length / 6000)), 0);
  return {
    charCount: text.trim().length,
    chapterCount: chapters.length,
    chunkCount,
    estimatedCalls: chunkCount,
    chapters: chapters.map((chapter, index) => ({
      index: index + 1,
      title: chapter.title,
      charCount: chapter.text.length,
    })),
  };
}

function expandNovelEpisodes(chapters: Array<{ title: string; text: string }>, maxEpisodes: number): Episode[] {
  const base = snapshot.episodes[0];
  if (!base) return [];
  const count = Math.max(1, Math.min(chapters.length, maxEpisodes > 0 ? maxEpisodes : chapters.length, 8));
  return Array.from({ length: count }, (_, index) => {
    const chapter = chapters[index] ?? chapters[chapters.length - 1];
    const episodeId = uid();
    return {
      ...structuredClone(base),
      id: episodeId,
      orderIndex: index,
      title: `第${index + 1}集 · ${chapter.title}`,
      summary: chapter.text.slice(0, 180),
      scenes: base.scenes.map((scene) => {
        const sceneId = uid();
        return {
          ...structuredClone(scene),
          id: sceneId,
          episodeId,
          title: `第${index + 1}集 · ${chapter.title}`,
          synopsis: chapter.text.slice(0, 160),
          shots: scene.shots.map((shot) => ({ ...structuredClone(shot), id: uid(), sceneId })),
        };
      }),
      createdAt: now(),
      updatedAt: now(),
    };
  });
}

async function mockInvoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  await sleep(30);
  switch (cmd) {
    case "health_check":
      return { ok: true, app: "SaladCart", mode: "web-demo" } as T;
    case "list_projects":
      return projects as T;
    case "open_project": {
      if (args.projectId !== snapshot.project.id) throw { code: "not_found", message: "演示模式只有一个项目" };
      return structuredClone(snapshot) as T;
    }
    case "create_project": {
      const input = args.input as CreateProjectInput;
      const id = `mock-${uid()}`;
      const fresh: Project = {
        id,
        name: input.name || "未命名漫剧",
        premise: input.premise,
        logline: "",
        genre: input.genre,
        tone: input.tone,
        language: input.language,
        aspectRatio: input.aspectRatio,
        targetEpisodeSeconds: input.targetEpisodeSeconds,
        budgetCap: input.budgetCap ?? 0,
        styleBible: input.styleBible ?? STYLE,
        createdAt: now(),
        updatedAt: now(),
      };
      snapshot = {
        project: fresh,
        characters: [],
        episodes: [],
        assets: [],
        jobs: [],
        novel: null,
        timeline: {
          id: uid(),
          name: "Main cut",
          aspectRatio: fresh.aspectRatio,
          width: fresh.aspectRatio === "16:9" ? 1920 : 1080,
          height: fresh.aspectRatio === "16:9" ? 1080 : fresh.aspectRatio === "1:1" ? 1080 : 1920,
          fps: 30,
          clips: [],
          subtitles: [],
          musicAssetId: null,
          musicVolume: 0.8,
          bpm: null,
          beatGridMs: [],
          beatOffsetMs: 0,
          updatedAt: now(),
        },
        storagePath: `~/Library/Application Support/com.saladcart.studio/projects/${id}`,
      };
      projects.unshift(summaryOf(snapshot));
      return structuredClone(snapshot) as T;
    }
    case "create_demo_project":
      return structuredClone(snapshot) as T;
    case "update_project": {
      const input = args.input as UpdateProjectInput;
      snapshot.project = { ...snapshot.project, ...input, updatedAt: now() };
      touch();
      return { ...snapshot.project } as T;
    }
    case "update_scene": {
      const input = args.input as UpdateSceneInput;
      for (const episode of snapshot.episodes) {
        episode.scenes = episode.scenes.map((scene) =>
          scene.id === args.sceneId ? { ...scene, ...input, updatedAt: now() } : scene,
        );
      }
      const scene = findScene(args.sceneId as string);
      touch();
      return structuredClone(scene) as T;
    }
    case "update_shot": {
      const input = args.input as UpdateShotInput;
      updateShotRecord(args.shotId as string, input as Partial<Shot>);
      const shot = findShot(args.shotId as string);
      touch();
      return structuredClone(shot) as T;
    }
    case "set_shot_status": {
      updateShotRecord(args.shotId as string, { status: args.status as string });
      touch();
      return structuredClone(findShot(args.shotId as string)) as T;
    }
    case "list_shot_takes": {
      const shot = findShot(args.shotId as string);
      if (!shot) throw new Error("镜头不存在");
      const linked = shotTakeLinks.get(shot.id) ?? new Set<string>();
      if (shot.videoAssetId) linked.add(shot.videoAssetId);
      shotTakeLinks.set(shot.id, linked);
      const takes: ShotTake[] = [];
      for (const assetId of linked) {
        const asset = snapshot.assets.find((item) => item.id === assetId);
        if (!asset) continue;
        const job = snapshot.jobs.find((item) => item.kind === "video" && item.resultAssetId === asset.id);
        takes.push({
          asset,
          selected: asset.id === shot.videoAssetId,
          jobId: job?.id ?? null,
          costActual: job?.costActual ?? 0,
        });
      }
      takes.sort((a, b) => Number(b.selected) - Number(a.selected));
      return structuredClone(takes) as T;
    }
    case "select_shot_take": {
      const shot = findShot(args.shotId as string);
      const asset = snapshot.assets.find((item) => item.id === args.assetId);
      if (!shot || !asset || asset.kind !== "video") throw new Error("镜头版本不存在");
      rememberShotTake(shot.id, asset.id);
      updateShotRecord(shot.id, {
        videoAssetId: asset.id,
        status: "needs_review",
        version: shot.version + 1,
        providerId: asset.providerId ?? "",
        model: asset.model ?? "",
      });
      touch();
      return structuredClone(findShot(shot.id)) as T;
    }
    case "delete_shot": {
      for (const episode of snapshot.episodes) {
        for (const scene of episode.scenes) scene.shots = scene.shots.filter((s) => s.id !== args.shotId);
      }
      touch();
      return undefined as T;
    }
    case "delete_scene": {
      for (const episode of snapshot.episodes) episode.scenes = episode.scenes.filter((s) => s.id !== args.sceneId);
      touch();
      return undefined as T;
    }
    case "create_character": {
      const input = args.input as Character["lockedTraits"] extends never ? never : {
        name: string;
        role: string;
        description: string;
        appearance: string;
        personality: string;
        voice: string;
        lockedTraits: string[];
      };
      const index = snapshot.characters.length + 3;
      const ref = makeAsset(uid(), "image", `${input.name} · 定妆照`, art(input.name, input.role || "NEW CAST", index, "character"), null, {
        providerId: "mock-image",
        model: "saladcart-mock-image",
      });
      snapshot.assets.unshift(ref);
      const character: Character = {
        id: uid(),
        name: input.name,
        role: input.role,
        description: input.description,
        appearance: input.appearance,
        personality: input.personality,
        voice: input.voice,
        negativePrompt: "",
        lockedTraits: input.lockedTraits,
        referenceAssetId: ref.id,
        sheetAssetId: null,
        seed: null,
        providerId: "mock-image",
        model: "saladcart-mock-image",
        status: "draft",
        createdAt: now(),
        updatedAt: now(),
      };
      snapshot.characters.push(character);
      touch();
      return structuredClone(character) as T;
    }
    case "update_character": {
      const input = args.input as UpdateCharacterInput;
      snapshot.characters = snapshot.characters.map((c) =>
        c.id === args.characterId ? { ...c, ...input, updatedAt: now() } : c,
      );
      touch();
      return structuredClone(snapshot.characters.find((c) => c.id === args.characterId)) as T;
    }
    case "delete_character": {
      snapshot.characters = snapshot.characters.filter((c) => c.id !== args.characterId);
      touch();
      return undefined as T;
    }
    case "list_project_references":
      return [] as ProjectReference[] as T;
    case "import_character": {
      throw { code: "unavailable", message: "浏览器演示模式不支持跨项目导入，请使用桌面版。" };
    }
    case "import_project_style":
      return structuredClone(snapshot.project) as T;
    case "export_provenance":
      return `${snapshot.storagePath}/exports/provenance.json` as T;
    case "generate_shot_voice": {
      const shot = findShot(args.shotId as string);
      if (!shot) throw new Error("镜头不存在");
      const asset = makeAsset(uid(), "audio", "配音 · 演示对白", "/demo/voice.wav", null, {
        durationMs: 4540,
        mime: "audio/wav",
        width: null,
        height: null,
        providerId: "system-tts",
        model: "demo-voice",
        source: "tts",
      });
      snapshot.assets.unshift(asset);
      updateShotRecord(shot.id, { voiceAssetId: asset.id });
      touch();
      return structuredClone(asset) as T;
    }
    case "get_context_preview":
      return contextFor(args.request as GenerationRequest) as T;
    case "preview_novel": {
      const input = args.input as NovelInput;
      if (input.text.trim().length < 50) throw new Error("小说正文至少 50 个字符");
      return novelPlan(input.text) as T;
    }
    case "get_novel": {
      return (snapshot.novel ? { ...snapshot.novel, text: novelText } : null) as T;
    }
    case "start_novel_adaptation": {
      const input = args.novel as NovelInput;
      const options = args.options as NovelAdaptRequest;
      if (input.text.trim().length < 50) throw new Error("小说正文至少 50 个字符");
      const chapters = splitMockNovel(input.text);
      const plan = novelPlan(input.text);
      novelText = input.text;
      snapshot.novel = {
        id: "mock-novel",
        title: input.title.trim() || chapters[0]?.title || "未命名小说",
        sourcePath: input.sourcePath ?? null,
        charCount: plan.charCount,
        chapterCount: plan.chapterCount,
        chunkCount: plan.chunkCount,
        status: "ready",
        analysis: {},
        createdAt: now(),
        updatedAt: now(),
      };
      const job = newJob(
        "novel",
        "project",
        snapshot.project.id,
        "mock-llm",
        "Mock 编剧 (离线)",
        "saladcart-mock-llm",
        `小说改编 · ${plan.chapterCount} 章 · ${plan.chunkCount} 块`,
        options as unknown as Record<string, unknown>,
      );
      void simulate(
        job,
        [[8, "splitting chapters"], [32, "analyzing chapters"], [62, "merging characters"], [82, "breaking down shots"]],
        () => {
          generateScriptContent({
            premise: input.text.slice(0, 300),
            genre: snapshot.project.genre,
            tone: snapshot.project.tone,
            language: snapshot.project.language,
            episodes: 1,
            scenesPerEpisode: 1,
            shotsPerScene: 3,
            styleNotes: options.styleNotes,
            providerId: "mock-llm",
            model: null,
            replaceExisting: true,
            seed: null,
          });
          snapshot.episodes = expandNovelEpisodes(chapters, options.maxEpisodes);
          snapshot.novel = {
            ...snapshot.novel!,
            status: "adapted",
            analysis: {
              mode: "mock",
              chapters: chapters.map((chapter, index) => ({
                index: index + 1,
                title: chapter.title,
                summary: chapter.text.slice(0, 180),
              })),
            },
            updatedAt: now(),
          };
          job.result = {
            chapters: plan.chapterCount,
            episodes: snapshot.episodes.length,
            scenes: snapshot.episodes.reduce((sum, episode) => sum + episode.scenes.length, 0),
            shots: snapshot.episodes.reduce(
              (sum, episode) => sum + episode.scenes.reduce((inner, scene) => inner + scene.shots.length, 0),
              0,
            ),
          };
        },
      );
      return job as T;
    }
    case "generate_script": {
      const request = args.request as GenerateScriptRequest;
      const job = newJob(
        "script",
        "project",
        snapshot.project.id,
        "mock-llm",
        "Mock 编剧 (离线)",
        "saladcart-mock-llm",
        `剧本 · ${request.episodes} 集`,
        request as unknown as Record<string, unknown>,
      );
      void simulate(job, [[12, "building story bible"], [46, "writing scenes"], [76, "breaking down shots"]], () => {
        generateScriptContent(request);
        job.result = { episodes: 1, scenes: 2, shots: 4 };
      });
      return job as T;
    }
    case "generate_media": {
      const request = args.request as GenerationRequest;
      const context = contextFor(request);
      if (snapshot.project.budgetCap > 0) {
        const committed = snapshot.jobs.reduce(
          (sum, item) =>
            sum + (item.status === "queued" || item.status === "running" ? item.costEstimate : item.costActual),
          0,
        );
        if (committed + context.estimatedCost > snapshot.project.budgetCap + 1e-6) {
          throw new Error(
            `项目预算上限 ¥${snapshot.project.budgetCap.toFixed(2)}，已占用 ¥${committed.toFixed(2)}，本次预计 ¥${context.estimatedCost.toFixed(2)}`,
          );
        }
      }
      const profile = settings.providers.find((p) => p.id === context.providerId) ?? settings.providers[0];
      const job = newJob(
        request.kind,
        request.targetType,
        request.targetId,
        profile.id,
        profile.label,
        profile.defaultModel,
        context.title,
        request as unknown as Record<string, unknown>,
        context.estimatedCost,
      );
      const stages: Array<[number, string]> =
        request.kind === "video"
          ? [[18, "submitting to provider"], [48, "rendering motion"], [78, "downloading take"]]
          : [[22, "generating image"], [64, "saving asset"]];
      void simulate(job, stages, () => {
        if (request.kind === "video") {
          const shot = findShot(request.targetId ?? "");
          if (!shot) return;
          const index = Math.floor(Math.random() * 3);
          const asset = makeAsset(uid(), "video", `${context.title} · 镜头成片`, videoPaths[index], art(shot.title, "NEW TAKE / MOCK", index), {
            source: "generated",
            providerId: profile.id,
            model: profile.defaultModel,
            durationMs: request.options.durationMs || shot.durationMs,
            status: "ready",
          });
          snapshot.assets.unshift(asset);
          rememberShotTake(shot.id, asset.id);
          updateShotRecord(shot.id, {
            videoAssetId: asset.id,
            status: "needs_review",
            version: shot.version + 1,
            providerId: profile.id,
            model: profile.defaultModel,
          });
          job.resultAssetId = asset.id;
        } else if (request.targetType === "character") {
          const character = snapshot.characters.find((c) => c.id === request.targetId);
          if (!character) return;
          const isSheet = request.options.variant === "sheet";
          const asset = makeAsset(
            uid(),
            "image",
            `${character.name} · ${isSheet ? "角色三视图" : "角色定妆照"}`,
            art(character.name, isSheet ? "CHARACTER SHEET" : "NEW PORTRAIT", Math.floor(Math.random() * 4), isSheet ? "sheet" : "character"),
            null,
            { providerId: profile.id, model: profile.defaultModel, status: "ready" },
          );
          snapshot.assets.unshift(asset);
          character.referenceAssetId = isSheet ? character.referenceAssetId : asset.id;
          character.sheetAssetId = isSheet ? asset.id : character.sheetAssetId;
          character.status = "ready";
          character.updatedAt = now();
          job.resultAssetId = asset.id;
        } else {
          const shot = findShot(request.targetId ?? "");
          if (!shot) return;
          const asset = makeAsset(uid(), "image", `${context.title} · 画面`, art(shot.title, "NEW KEYFRAME / MOCK", Math.floor(Math.random() * 4)), null, {
            providerId: profile.id,
            model: profile.defaultModel,
            status: "ready",
          });
          snapshot.assets.unshift(asset);
          updateShotRecord(shot.id, { imageAssetId: asset.id, status: "needs_review" });
          job.resultAssetId = asset.id;
        }
        job.result = { assetId: job.resultAssetId, mock: true };
      });
      return job as T;
    }
    case "list_jobs":
      return snapshot.jobs.slice(0, (args.limit as number) ?? 80) as T;
    case "cancel_job": {
      cancelled.add(args.jobId as string);
      const job = snapshot.jobs.find((j) => j.id === args.jobId);
      if (job) {
        job.status = "canceled";
        job.stage = "canceled";
        job.finishedAt = now();
        emitJob(job);
        return { ...job } as T;
      }
      throw new Error("job not found");
    }
    case "retry_job": {
      const original = snapshot.jobs.find((j) => j.id === args.jobId);
      if (!original) throw new Error("job not found");
      const job = newJob(
        original.kind,
        original.targetType,
        original.targetId,
        original.providerId,
        original.providerLabel,
        original.model,
        original.requestSummary,
        original.request,
        original.costEstimate,
      );
      void simulate(job, [[35, "retrying"], [74, "finishing"]], () => {
        job.result = { retried: true };
      });
      return job as T;
    }
    case "get_timeline":
      return structuredClone(snapshot.timeline) as T;
    case "update_timeline": {
      const patch = args.patch as TimelinePatch;
      const next = { ...snapshot.timeline, ...patch, updatedAt: now() } as Timeline;
      if (patch.clearMusic) next.musicAssetId = null;
      if (patch.clips) {
        next.clips = patch.clips.map((clip, index) => ({
          id: uid(),
          timelineId: next.id,
          shotId: clip.shotId ?? null,
          assetId: clip.assetId ?? null,
          orderIndex: index,
          inMs: clip.inMs,
          outMs: clip.outMs,
          durationMs: Math.max(1, clip.outMs - clip.inMs),
          label: clip.label,
          muted: clip.muted,
          transition: clip.transition,
        }));
      }
      if (patch.subtitles) {
        next.subtitles = patch.subtitles.map((subtitle) => ({
          id: uid(),
          timelineId: next.id,
          startMs: subtitle.startMs,
          endMs: subtitle.endMs,
          text: subtitle.text,
        }));
      }
      snapshot.timeline = next;
      touch();
      return structuredClone(next) as T;
    }
    case "auto_assemble_timeline": {
      const approvedOnly = Boolean(args.approvedOnly);
      const clips = [];
      let cursor = 0;
      const subtitles = [];
      for (const episode of snapshot.episodes) {
        for (const scene of episode.scenes) {
          for (const shot of scene.shots) {
            if (!shot.videoAssetId) continue;
            if (approvedOnly && shot.status !== "approved") continue;
            const asset = snapshot.assets.find((a) => a.id === shot.videoAssetId);
            const duration = asset?.durationMs ?? shot.durationMs;
            clips.push({
              id: uid(),
              timelineId: snapshot.timeline.id,
              shotId: shot.id,
              assetId: shot.videoAssetId,
              orderIndex: clips.length,
              inMs: 0,
              outMs: duration,
              durationMs: duration,
              label: `${scene.title} · ${shot.title}`,
              muted: false,
              transition: "cut",
            });
            if (shot.dialogue) {
              subtitles.push({
                id: uid(),
                timelineId: snapshot.timeline.id,
                startMs: cursor + 120,
                endMs: cursor + duration - 120,
                text: shot.dialogue,
              });
            }
            cursor += duration;
          }
        }
      }
      snapshot.timeline = { ...snapshot.timeline, clips, subtitles, updatedAt: now() };
      touch();
      return structuredClone(snapshot.timeline) as T;
    }
    case "export_timeline": {
      const options = args.options as ExportOptions;
      const job = newJob("export", "timeline", snapshot.timeline.id, "ffmpeg", "本地 FFmpeg", "ffmpeg", `导出 · ${options.width}×${options.height}`, options as unknown as Record<string, unknown>);
      void simulate(job, [[20, "preparing clips"], [50, "rendering with FFmpeg"], [85, "finalizing MP4"]], () => {
        job.result = { path: "/demo/saladcart-export.mp4", width: options.width, height: options.height, mock: true };
      });
      return job as T;
    }
    case "get_settings":
      return structuredClone(settings) as T;
    case "update_provider": {
      const patch = args.patch as ProviderPatch;
      settings.providers = settings.providers.map((p) => (p.id === args.providerId ? { ...p, ...patch } : p));
      return structuredClone(settings) as T;
    }
    case "update_preferences": {
      const patch = args.patch as PreferencesPatch;
      if (patch.defaults) settings.defaults = patch.defaults;
      if (patch.concurrency) settings.concurrency = patch.concurrency;
      if (patch.ffmpegPath !== undefined) settings.ffmpegPath = patch.ffmpegPath;
      if (patch.clearFfmpegPath) settings.ffmpegPath = null;
      return structuredClone(settings) as T;
    }
    case "set_provider_secret": {
      settings.providers = settings.providers.map((p) => (p.id === args.providerId ? { ...p, apiKeySet: true } : p));
      return structuredClone(settings) as T;
    }
    case "clear_provider_secret": {
      settings.providers = settings.providers.map((p) => (p.id === args.providerId ? { ...p, apiKeySet: false } : p));
      return structuredClone(settings) as T;
    }
    case "test_provider": {
      const profile = settings.providers.find((p) => p.id === args.providerId);
      if (!profile) throw new Error("provider not found");
      const local = profile.id.startsWith("mock-") || profile.vendor === "ollama" || profile.id === "comfyui-image";
      return {
        ok: local || profile.apiKeySet,
        message: local
          ? "离线/本地供应商可用，不消耗额度。"
          : profile.apiKeySet
            ? "API Key 已保存在系统钥匙串。真实生成会验证账号并消耗额度。"
            : "还没有配置 API Key。",
        latencyMs: 1,
      } as ProviderTestResult as T;
    }
    case "get_system_status":
      return {
        ffmpegAvailable: true,
        ffmpegPath: "/opt/homebrew/bin/ffmpeg",
        ffmpegVersion: "ffmpeg version 8.0 (demo bridge)",
        platform: "browser demo · 原生构建请使用 Tauri",
        appVersion: "0.3.0",
        storageDir: "~/Library/Application Support/com.saladcart.studio",
        projectCount: projects.length,
        dataDir: "~/Library/Application Support/com.saladcart.studio",
      } as SystemStatus as T;
    case "open_storage_folder":
    case "reveal_project_folder":
      return undefined as T;
    case "import_asset":
    case "import_asset_path": {
      const input = args.input as { kind: string; name: string };
      const asset = makeAsset(uid(), (input.kind || "image") as Asset["kind"], input.name || "导入素材", art(input.name || "导入素材", "IMPORTED ASSET", 1), null, {
        source: "upload",
      });
      snapshot.assets.unshift(asset);
      touch();
      return structuredClone(asset) as T;
    }
    case "list_assets": {
      const kind = args.kind as string | undefined;
      return snapshot.assets.filter((a) => !kind || a.kind === kind) as T;
    }
    case "set_asset_status": {
      snapshot.assets = snapshot.assets.map((a) => (a.id === args.assetId ? { ...a, status: args.status as string } : a));
      return structuredClone(snapshot.assets.find((a) => a.id === args.assetId)) as T;
    }
    case "search_music": {
      const query = String(args.query ?? "");
      const tracks: MusicTrack[] = [
        {
          id: "jam-rain",
          title: "Rain on the Old Line",
          artist: "Lumen Field",
          providerId: "jamendo-music",
          providerLabel: "Jamendo",
          previewUrl: "/demo/score.wav",
          downloadUrl: "/demo/score.wav",
          durationMs: 24000,
          license: "CC BY 4.0",
          licenseUrl: "https://creativecommons.org/licenses/by/4.0/",
          requiresAttribution: true,
          tags: ["cinematic", "rain", "piano"],
          coverUrl: art("Rain on the Old Line", "JAMENDO / CC BY", 2),
        },
        {
          id: "jam-signal",
          title: "Signal in the Fog",
          artist: "North Window",
          providerId: "jamendo-music",
          providerLabel: "Jamendo",
          previewUrl: "/demo/score.wav",
          downloadUrl: "/demo/score.wav",
          durationMs: 24000,
          license: "CC BY-SA 4.0",
          licenseUrl: "https://creativecommons.org/licenses/by-sa/4.0/",
          requiresAttribution: true,
          tags: ["mystery", "pulse", "score"],
          coverUrl: art("Signal in the Fog", "JAMENDO / CC BY-SA", 3),
        },
      ];
      return { tracks: query ? tracks : tracks, providerLabel: "Jamendo · 演示数据", query } as MusicSearchResult as T;
    }
    case "import_music_url": {
      const track = args.track as MusicTrack;
      const asset = makeAsset(uid(), "audio", track.title, track.downloadUrl, null, {
        providerId: track.providerId,
        model: "stock",
        source: "stock",
        durationMs: track.durationMs,
        mime: "audio/wav",
        width: null,
        height: null,
        license: track.license,
        licenseUrl: track.licenseUrl,
        attribution: `${track.title} — ${track.artist}`,
        sourceUrl: track.downloadUrl,
      });
      snapshot.assets.unshift(asset);
      touch();
      return structuredClone(asset) as T;
    }
    case "generate_music": {
      const request = args.request as MusicGenerationRequest;
      const job = newJob("music", "project", snapshot.project.id, "mock-music", "Mock 配乐 (离线)", "saladcart-mock-music", `AI 配乐 · ${request.prompt || "无提示词"}`, request as unknown as Record<string, unknown>);
      void simulate(job, [[24, "composing motif"], [62, "arranging stems"], [84, "mastering"]], () => {
        const asset = makeAsset(uid(), "audio", `${request.prompt || "原创配乐"} · AI 配乐`, "/demo/score.wav", null, {
          providerId: "mock-music",
          model: "saladcart-mock-music",
          source: "generated",
          durationMs: request.durationMs,
          mime: "audio/wav",
          width: null,
          height: null,
        });
        snapshot.assets.unshift(asset);
        if (request.attachToTimeline) {
          snapshot.timeline.musicAssetId = asset.id;
          snapshot.timeline.bpm = 100;
          snapshot.timeline.beatGridMs = Array.from({ length: Math.floor(request.durationMs / 600) + 1 }, (_, i) => i * 600);
          snapshot.timeline.beatOffsetMs = 0;
        }
        job.resultAssetId = asset.id;
        job.result = { assetId: asset.id, bpm: 100 };
      });
      return job as T;
    }
    case "analyze_music_beats": {
      const asset = snapshot.assets.find((a) => a.id === snapshot.timeline.musicAssetId);
      if (!asset) throw new Error("时间线还没有配乐");
      const duration = asset.durationMs ?? 24000;
      const bpm = 100;
      const beatGridMs = Array.from({ length: Math.floor(duration / 600) + 1 }, (_, i) => i * 600);
      const waveform = Array.from({ length: 160 }, (_, i) => 0.15 + Math.abs(Math.sin(i * 0.23)) * 0.5 + Math.abs(Math.sin(i * 0.63)) * 0.25);
      snapshot.timeline = { ...snapshot.timeline, bpm, beatGridMs, beatOffsetMs: 0, updatedAt: now() };
      touch();
      return {
        assetId: asset.id,
        bpm,
        confidence: 0.92,
        offsetMs: 0,
        beatGridMs,
        waveform,
        durationMs: duration,
      } as BeatAnalysis as T;
    }
    case "auto_beat_sync": {
      const beats = snapshot.timeline.beatGridMs ?? [];
      if (!beats.length) throw new Error("请先分析配乐节拍");
      let alignedCuts = 0;
      let offsetTotal = 0;
      let cursor = 0;
      const clips = snapshot.timeline.clips.map((clip) => {
        const sourceDuration = snapshot.assets.find((a) => a.id === clip.assetId)?.durationMs ?? clip.durationMs;
        const target = cursor + clip.durationMs;
        const nearest = beats.reduce((best, beat) => (Math.abs(beat - target) < Math.abs(best - target) ? beat : best), beats[0]);
        let duration = clip.durationMs;
        if (Math.abs(nearest - target) <= 1200) {
          duration = Math.max(700, nearest - cursor);
          offsetTotal += Math.abs(nearest - target);
          alignedCuts += 1;
        }
        const outMs = Math.min(sourceDuration, clip.inMs + duration);
        const next = { ...clip, durationMs: outMs - clip.inMs, outMs };
        cursor += next.durationMs;
        return next;
      });
      snapshot.timeline = { ...snapshot.timeline, clips, updatedAt: now() };
      touch();
      return {
        alignedCuts,
        totalCuts: clips.length,
        averageOffsetMs: alignedCuts ? Math.round(offsetTotal / alignedCuts) : 0,
        timeline: structuredClone(snapshot.timeline),
      } as BeatSyncResult as T;
    }
    case "attach_music": {
      const asset = snapshot.assets.find((a) => a.id === args.assetId && a.kind === "audio");
      if (!asset) throw new Error("音乐素材不存在");
      snapshot.timeline = {
        ...snapshot.timeline,
        musicAssetId: asset.id,
        musicVolume: (args.volume as number) ?? snapshot.timeline.musicVolume ?? 0.8,
        updatedAt: now(),
      };
      touch();
      return structuredClone(snapshot.timeline) as T;
    }
    default:
      throw { code: "not_implemented", message: `浏览器演示桥接未实现命令: ${cmd}` };
  }
}

export async function mockListen<T>(
  event: string,
  callback: (event: { payload: T }) => void,
): Promise<() => void> {
  const set = listeners.get(event) ?? new Set();
  set.add(callback as (event: { payload: unknown }) => void);
  listeners.set(event, set);
  return () => set.delete(callback as (event: { payload: unknown }) => void);
}

export { mockInvoke, secondProject };
