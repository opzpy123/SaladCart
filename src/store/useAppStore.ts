import { create } from "zustand";
import { formatError, invoke, listen } from "@/lib/bridge";
import type {
  BeatAnalysis,
  BeatSyncResult,
  Character,
  ContextPreview,
  CreateCharacterInput,
  CreateProjectInput,
  ExportOptions,
  GenerateScriptRequest,
  GenerationJob,
  GenerationRequest,
  MusicGenerationRequest,
  MusicSearchResult,
  MusicTrack,
  PreferencesPatch,
  ProjectReference,
  ProjectSnapshot,
  ProjectSummary,
  ProviderPatch,
  ProviderTestResult,
  Route,
  Scene,
  Settings,
  Shot,
  SystemStatus,
  Timeline,
  TimelinePatch,
  Toast,
  UpdateCharacterInput,
  UpdateProjectInput,
  UpdateSceneInput,
  UpdateShotInput,
} from "@/lib/types";

let initStarted = false;
let refreshTimer: ReturnType<typeof setTimeout> | null = null;
let toastCounter = 0;

interface AppStore {
  initialized: boolean;
  loading: boolean;
  fatalError: string | null;
  settings: Settings | null;
  system: SystemStatus | null;
  projects: ProjectSummary[];
  snapshot: ProjectSnapshot | null;
  route: Route;
  currentSceneId: string | null;
  currentShotId: string | null;
  currentCharacterId: string | null;
  selectedShotIds: string[];
  showNewProject: boolean;
  showScriptDialog: boolean;
  jobDockOpen: boolean;
  contextOpen: boolean;
  contextLoading: boolean;
  contextPreview: ContextPreview | null;
  musicTracks: MusicTrack[];
  musicSearching: boolean;
  musicAnalysis: BeatAnalysis | null;
  musicGenerating: boolean;
  toasts: Toast[];

  init: () => Promise<void>;
  refreshProjects: () => Promise<void>;
  refreshSnapshot: (silent?: boolean) => Promise<void>;
  openProject: (id: string) => Promise<void>;
  createProject: (input: CreateProjectInput, generateAfter: boolean, episodes?: number) => Promise<void>;
  createDemo: () => Promise<void>;
  archiveProject: (id: string) => Promise<void>;
  updateProject: (input: UpdateProjectInput) => Promise<void>;
  setRoute: (route: Route) => void;
  selectScene: (sceneId: string) => void;
  selectShot: (shotId: string) => void;
  selectCharacter: (characterId: string) => void;
  toggleShotSelection: (shotId: string) => void;
  setShotSelection: (ids: string[]) => void;
  setShowNewProject: (open: boolean) => void;
  setShowScriptDialog: (open: boolean) => void;
  setJobDockOpen: (open: boolean) => void;
  closeContext: () => void;
  pushToast: (kind: Toast["kind"], title: string, message?: string) => void;
  removeToast: (id: string) => void;

  generateScript: (overrides?: Partial<GenerateScriptRequest>) => Promise<void>;
  generateCharacter: (characterId: string, variant: "portrait" | "sheet") => Promise<void>;
  generateVoice: (shotId: string) => Promise<void>;
  generateVoices: (shotIds: string[]) => Promise<void>;
  generateShot: (shotId: string, kind: "image" | "video") => Promise<void>;
  batchGenerate: (shotIds: string[], kind: "image" | "video", useContinuity?: boolean) => Promise<void>;
  openShotContext: (shotId: string, kind: "image" | "video") => Promise<void>;
  saveShot: (shotId: string, patch: UpdateShotInput) => Promise<void>;
  saveScene: (sceneId: string, patch: UpdateSceneInput) => Promise<void>;
  saveCharacter: (characterId: string, patch: UpdateCharacterInput) => Promise<void>;
  addCharacter: (input: CreateCharacterInput) => Promise<void>;
  removeCharacter: (characterId: string) => Promise<void>;
  importCharacter: (sourceProjectId: string, characterId: string) => Promise<void>;
  importStyle: (sourceProjectId: string) => Promise<void>;
  setShotStatus: (shotId: string, status: string) => Promise<void>;

  loadReferences: () => Promise<ProjectReference[]>;

  autoAssemble: () => Promise<void>;
  updateTimeline: (patch: TimelinePatch) => Promise<void>;
  exportTimeline: (options: ExportOptions) => Promise<void>;
  exportProvenance: () => Promise<void>;
  searchMusic: (query: string) => Promise<void>;
  importMusicTrack: (track: MusicTrack) => Promise<void>;
  generateMusic: (request: MusicGenerationRequest) => Promise<void>;
  analyzeBeats: () => Promise<void>;
  autoBeatSync: () => Promise<void>;
  attachMusic: (assetId: string, volume?: number) => Promise<void>;

  refreshSettings: () => Promise<void>;
  updatePreferences: (patch: PreferencesPatch) => Promise<void>;
  updateProvider: (providerId: string, patch: ProviderPatch) => Promise<void>;
  saveProviderSecret: (providerId: string, secret: string) => Promise<void>;
  clearProviderSecret: (providerId: string) => Promise<void>;
  testProvider: (providerId: string) => Promise<ProviderTestResult>;
  refreshSystem: () => Promise<void>;

  upsertJob: (job: GenerationJob) => void;
  handleJob: (job: GenerationJob) => void;
}

function firstScene(snapshot: ProjectSnapshot | null): Scene | null {
  return snapshot?.episodes[0]?.scenes[0] ?? null;
}

function updateShotInSnapshot(snapshot: ProjectSnapshot, shot: Shot): ProjectSnapshot {
  return {
    ...snapshot,
    episodes: snapshot.episodes.map((episode) => ({
      ...episode,
      scenes: episode.scenes.map((scene) => ({
        ...scene,
        shots: scene.shots.map((s) => (s.id === shot.id ? shot : s)),
      })),
    })),
  };
}

function updateSceneInSnapshot(snapshot: ProjectSnapshot, scene: Scene): ProjectSnapshot {
  return {
    ...snapshot,
    episodes: snapshot.episodes.map((episode) => ({
      ...episode,
      scenes: episode.scenes.map((s) => (s.id === scene.id ? scene : s)),
    })),
  };
}

function updateCharacterInSnapshot(snapshot: ProjectSnapshot, character: Character): ProjectSnapshot {
  return {
    ...snapshot,
    characters: snapshot.characters.map((c) => (c.id === character.id ? character : c)),
  };
}

function shotsOfScene(snapshot: ProjectSnapshot | null, sceneId: string | null): Shot[] {
  if (!snapshot || !sceneId) return [];
  for (const episode of snapshot.episodes) {
    const scene = episode.scenes.find((s) => s.id === sceneId);
    if (scene) return scene.shots;
  }
  return [];
}

function allShots(snapshot: ProjectSnapshot | null): Shot[] {
  if (!snapshot) return [];
  return snapshot.episodes.flatMap((episode) => episode.scenes.flatMap((scene) => scene.shots));
}

function sceneOfShot(snapshot: ProjectSnapshot | null, shotId: string | null): Scene | null {
  if (!snapshot || !shotId) return null;
  for (const episode of snapshot.episodes) {
    for (const scene of episode.scenes) {
      if (scene.shots.some((s) => s.id === shotId)) return scene;
    }
  }
  return null;
}

export const useAppStore = create<AppStore>()((set, get) => {
  const pushToast = (kind: Toast["kind"], title: string, message?: string) => {
    const toast: Toast = { id: `toast-${++toastCounter}`, kind, title, message };
    set((state) => ({ toasts: [...state.toasts, toast].slice(-4) }));
    setTimeout(() => get().removeToast(toast.id), kind === "error" ? 7000 : 4200);
  };

  const scheduleRefresh = () => {
    if (refreshTimer) clearTimeout(refreshTimer);
    refreshTimer = setTimeout(() => {
      refreshTimer = null;
      void get().refreshSnapshot(true);
    }, 550);
  };

  const shotRequest = (
    shotId: string,
    kind: "image" | "video",
    useContinuity?: boolean,
  ): GenerationRequest => {
    const snapshot = get().snapshot;
    const shot = allShots(snapshot).find((s) => s.id === shotId);
    const settings = get().settings;
    return {
      kind,
      targetType: "shot",
      targetId: shotId,
      providerId: kind === "video" ? settings?.defaults.video : settings?.defaults.image,
      model: null,
      options: {
        durationMs: shot?.durationMs ?? 5000,
        aspectRatio: snapshot?.project.aspectRatio ?? "9:16",
        seed: shot?.seed ?? null,
        variant: null,
        extraPrompt: "",
        useContinuity: useContinuity ?? Boolean(shot?.continuityFromShotId),
        referenceAssetIds: [],
      },
    };
  };

  return {
    initialized: false,
    loading: false,
    fatalError: null,
    settings: null,
    system: null,
    projects: [],
    snapshot: null,
    route: "dashboard",
    currentSceneId: null,
    currentShotId: null,
    currentCharacterId: null,
    selectedShotIds: [],
    showNewProject: false,
    showScriptDialog: false,
    jobDockOpen: false,
    contextOpen: false,
    contextLoading: false,
    contextPreview: null,
    musicTracks: [],
    musicSearching: false,
    musicAnalysis: null,
    musicGenerating: false,
    toasts: [],

    init: async () => {
      if (initStarted) return;
      initStarted = true;
      set({ loading: true });
      try {
        const [settings, system, projects] = await Promise.all([
          invoke<Settings>("get_settings"),
          invoke<SystemStatus>("get_system_status"),
          invoke<ProjectSummary[]>("list_projects"),
        ]);
        let snapshot: ProjectSnapshot | null = null;
        if (projects.length) {
          try {
            snapshot = await invoke<ProjectSnapshot>("open_project", { projectId: projects[0].id });
          } catch {
            snapshot = null;
          }
        }
        const scene = firstScene(snapshot);
        set({
          settings,
          system,
          projects,
          snapshot,
          currentSceneId: scene?.id ?? null,
          currentShotId: scene?.shots[0]?.id ?? null,
          currentCharacterId: snapshot?.characters[0]?.id ?? null,
          loading: false,
          initialized: true,
        });
        void listen<GenerationJob>("job:update", ({ payload }) => get().handleJob(payload)).catch(
          (error) => console.warn("job listener failed", error),
        );
        void listen<{ projectId: string }>("project:changed", () => {
          if (get().snapshot) scheduleRefresh();
        }).catch((error) => console.warn("project listener failed", error));
      } catch (error) {
        set({
          loading: false,
          initialized: true,
          fatalError: formatError(error),
        });
      }
    },

    refreshProjects: async () => {
      try {
        set({ projects: await invoke<ProjectSummary[]>("list_projects") });
      } catch (error) {
        pushToast("error", "无法读取项目列表", formatError(error));
      }
    },

    refreshSnapshot: async (silent = false) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        if (!silent) set({ loading: true });
        const next = await invoke<ProjectSnapshot>("open_project", { projectId: snapshot.project.id });
        const scene = firstScene(next);
        const currentSceneId = next.episodes.some((e) => e.scenes.some((s) => s.id === get().currentSceneId))
          ? get().currentSceneId
          : scene?.id ?? null;
        const sceneObj =
          next.episodes.flatMap((e) => e.scenes).find((s) => s.id === currentSceneId) ?? scene;
        const currentShotId = sceneObj?.shots.some((s) => s.id === get().currentShotId)
          ? get().currentShotId
          : sceneObj?.shots[0]?.id ?? null;
        const currentCharacterId = next.characters.some((c) => c.id === get().currentCharacterId)
          ? get().currentCharacterId
          : next.characters[0]?.id ?? null;
        set({
          snapshot: next,
          currentSceneId,
          currentShotId,
          currentCharacterId,
          loading: false,
        });
      } catch (error) {
        set({ loading: false });
        if (!silent) pushToast("error", "无法刷新项目", formatError(error));
      }
    },

    openProject: async (id) => {
      set({ loading: true });
      try {
        const snapshot = await invoke<ProjectSnapshot>("open_project", { projectId: id });
        const scene = firstScene(snapshot);
        set({
          snapshot,
          route: "script",
          currentSceneId: scene?.id ?? null,
          currentShotId: scene?.shots[0]?.id ?? null,
          currentCharacterId: snapshot.characters[0]?.id ?? null,
          selectedShotIds: [],
          loading: false,
          musicAnalysis: null,
        });
      } catch (error) {
        set({ loading: false });
        pushToast("error", "无法打开项目", formatError(error));
      }
    },

    createProject: async (input, generateAfter, episodes = 1) => {
      set({ loading: true });
      try {
        const snapshot = await invoke<ProjectSnapshot>("create_project", { input });
        const scene = firstScene(snapshot);
        set({
          snapshot,
          route: "script",
          showNewProject: false,
          currentSceneId: scene?.id ?? null,
          currentShotId: scene?.shots[0]?.id ?? null,
          currentCharacterId: null,
          loading: false,
        });
        await get().refreshProjects();
        if (generateAfter) {
          await get().generateScript({
            premise: input.premise,
            genre: input.genre,
            tone: input.tone,
            language: input.language,
            styleNotes: "",
            replaceExisting: false,
            episodes,
          });
        }
      } catch (error) {
        set({ loading: false });
        pushToast("error", "创建项目失败", formatError(error));
      }
    },

    createDemo: async () => {
      set({ loading: true });
      try {
        const snapshot = await invoke<ProjectSnapshot>("create_demo_project");
        const scene = firstScene(snapshot);
        set({
          snapshot,
          route: "script",
          showNewProject: false,
          currentSceneId: scene?.id ?? null,
          currentShotId: scene?.shots[0]?.id ?? null,
          currentCharacterId: snapshot.characters[0]?.id ?? null,
          selectedShotIds: [],
          loading: false,
        });
        await get().refreshProjects();
        pushToast("success", "演示项目已就绪", "剧本、角色、分镜、配乐都已装车");
      } catch (error) {
        set({ loading: false });
        pushToast("error", "无法载入演示项目", formatError(error));
      }
    },

    archiveProject: async (id) => {
      try {
        await invoke("archive_project", { projectId: id });
        await get().refreshProjects();
        if (get().snapshot?.project.id === id) {
          set({
            snapshot: null,
            route: "dashboard",
            currentSceneId: null,
            currentShotId: null,
            currentCharacterId: null,
          });
        }
        pushToast("success", "项目已移入回收区");
      } catch (error) {
        pushToast("error", "无法归档项目", formatError(error));
      }
    },

    updateProject: async (input) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const project = await invoke<ProjectSnapshot["project"]>("update_project", {
          projectId: snapshot.project.id,
          input,
        });
        set({ snapshot: { ...snapshot, project } });
        void get().refreshProjects();
      } catch (error) {
        pushToast("error", "保存项目失败", formatError(error));
      }
    },

    setRoute: (route) => set({ route }),

    selectScene: (sceneId) => {
      const shots = shotsOfScene(get().snapshot, sceneId);
      set({ currentSceneId: sceneId, currentShotId: shots[0]?.id ?? null });
    },

    selectShot: (shotId) => {
      const scene = sceneOfShot(get().snapshot, shotId);
      set({ currentShotId: shotId, currentSceneId: scene?.id ?? get().currentSceneId });
    },

    selectCharacter: (characterId) => set({ currentCharacterId: characterId }),

    toggleShotSelection: (shotId) =>
      set((state) => ({
        selectedShotIds: state.selectedShotIds.includes(shotId)
          ? state.selectedShotIds.filter((id) => id !== shotId)
          : [...state.selectedShotIds, shotId],
      })),

    setShotSelection: (ids) => set({ selectedShotIds: ids }),
    setShowNewProject: (open) => set({ showNewProject: open }),
    setShowScriptDialog: (open) => set({ showScriptDialog: open }),
    setJobDockOpen: (open) => set({ jobDockOpen: open }),
    closeContext: () => set({ contextOpen: false, contextPreview: null }),
    pushToast,
    removeToast: (id) => set((state) => ({ toasts: state.toasts.filter((t) => t.id !== id) })),

    generateScript: async (overrides = {}) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      const request: GenerateScriptRequest = {
        premise: snapshot.project.premise,
        genre: snapshot.project.genre,
        tone: snapshot.project.tone,
        language: snapshot.project.language,
        episodes: 1,
        scenesPerEpisode: 2,
        shotsPerScene: 3,
        styleNotes: "",
        providerId: get().settings?.defaults.llm ?? null,
        model: null,
        replaceExisting: snapshot.episodes.length > 0,
        seed: null,
        ...overrides,
      };
      try {
        const job = await invoke<GenerationJob>("generate_script", {
          projectId: snapshot.project.id,
          request,
        });
        get().upsertJob(job);
        set({ showScriptDialog: false });
        pushToast("info", "剧本生成已入队", `${job.providerLabel} · ${job.model}`);
      } catch (error) {
        pushToast("error", "无法启动剧本生成", formatError(error));
      }
    },

    generateCharacter: async (characterId, variant) => {
      const snapshot = get().snapshot;
      const character = snapshot?.characters.find((c) => c.id === characterId);
      if (!snapshot || !character) return;
      const request: GenerationRequest = {
        kind: "image",
        targetType: "character",
        targetId: characterId,
        providerId: get().settings?.defaults.image,
        model: null,
        options: {
          aspectRatio: snapshot.project.aspectRatio,
          variant,
          seed: character.seed,
          extraPrompt: "",
          useContinuity: false,
          referenceAssetIds: [],
        },
      };
      try {
        const job = await invoke<GenerationJob>("generate_media", {
          projectId: snapshot.project.id,
          request,
        });
        get().upsertJob(job);
        pushToast("info", variant === "sheet" ? "三视图生成已入队" : "定妆照生成已入队", character.name);
      } catch (error) {
        pushToast("error", "无法启动角色生成", formatError(error));
      }
    },

    generateVoice: async (shotId) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const asset = await invoke<ProjectSnapshot["assets"][number]>("generate_shot_voice", {
          projectId: snapshot.project.id,
          shotId,
          voice: null,
        });
        set((state) => {
          if (!state.snapshot) return {};
          return {
            snapshot: {
              ...state.snapshot,
              assets: [asset, ...state.snapshot.assets.filter((item) => item.id !== asset.id)],
              episodes: state.snapshot.episodes.map((episode) => ({
                ...episode,
                scenes: episode.scenes.map((scene) => ({
                  ...scene,
                  shots: scene.shots.map((shot) =>
                    shot.id === shotId ? { ...shot, voiceAssetId: asset.id } : shot,
                  ),
                })),
              })),
            },
          };
        });
        pushToast("success", "配音已生成", asset.name);
      } catch (error) {
        pushToast("error", "配音生成失败", formatError(error));
      }
    },

    generateVoices: async (shotIds) => {
      const snapshot = get().snapshot;
      if (!snapshot || !shotIds.length) return;
      pushToast("info", `开始生成 ${shotIds.length} 条配音`, "使用系统语音，逐条写入项目");
      let done = 0;
      let failed = 0;
      for (const shotId of shotIds) {
        try {
          await invoke("generate_shot_voice", {
            projectId: snapshot.project.id,
            shotId,
            voice: null,
          });
          done += 1;
        } catch {
          failed += 1;
        }
      }
      await get().refreshSnapshot(true);
      if (done) {
        pushToast("success", `${done} 条配音已生成`, failed ? `${failed} 条失败，可单独重试` : "导出时自动混入");
      }
      if (failed && !done) {
        pushToast("error", "配音生成失败", "请检查系统语音是否可用");
      }
    },

    generateShot: async (shotId, kind) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      const shot = allShots(snapshot).find((s) => s.id === shotId);
      try {
        const job = await invoke<GenerationJob>("generate_media", {
          projectId: snapshot.project.id,
          request: shotRequest(shotId, kind),
        });
        get().upsertJob(job);
        pushToast("info", kind === "video" ? "镜头视频已入队" : "画面生成已入队", shot?.title);
        set({ jobDockOpen: true });
      } catch (error) {
        pushToast("error", "无法启动生成", formatError(error));
      }
    },

    batchGenerate: async (shotIds, kind, useContinuity = true) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      let queued = 0;
      for (const shotId of shotIds) {
        try {
          const job = await invoke<GenerationJob>("generate_media", {
            projectId: snapshot.project.id,
            request: shotRequest(shotId, kind, useContinuity),
          });
          get().upsertJob(job);
          queued += 1;
        } catch (error) {
          pushToast("error", "批量任务失败", formatError(error));
        }
      }
      if (queued) {
        pushToast("success", `已加入 ${queued} 个${kind === "video" ? "视频" : "画面"}任务`, "任务会按并发上限自动执行");
        set({ jobDockOpen: true, selectedShotIds: [] });
      }
    },

    openShotContext: async (shotId, kind) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      set({ contextOpen: true, contextLoading: true, contextPreview: null });
      try {
        const preview = await invoke<ContextPreview>("get_context_preview", {
          projectId: snapshot.project.id,
          request: shotRequest(shotId, kind),
        });
        set({ contextPreview: preview, contextLoading: false });
      } catch (error) {
        set({ contextLoading: false, contextOpen: false });
        pushToast("error", "无法编译上下文", formatError(error));
      }
    },

    saveShot: async (shotId, patch) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const shot = await invoke<Shot>("update_shot", {
          projectId: snapshot.project.id,
          shotId,
          input: patch,
        });
        set((state) => ({ snapshot: state.snapshot ? updateShotInSnapshot(state.snapshot, shot) : null }));
      } catch (error) {
        pushToast("error", "保存镜头失败", formatError(error));
      }
    },

    saveScene: async (sceneId, patch) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const scene = await invoke<Scene>("update_scene", {
          projectId: snapshot.project.id,
          sceneId,
          input: patch,
        });
        set((state) => ({ snapshot: state.snapshot ? updateSceneInSnapshot(state.snapshot, scene) : null }));
      } catch (error) {
        pushToast("error", "保存场景失败", formatError(error));
      }
    },

    saveCharacter: async (characterId, patch) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const character = await invoke<Character>("update_character", {
          projectId: snapshot.project.id,
          characterId,
          input: patch,
        });
        set((state) => ({
          snapshot: state.snapshot ? updateCharacterInSnapshot(state.snapshot, character) : null,
        }));
        pushToast("success", "角色设定已保存");
      } catch (error) {
        pushToast("error", "保存角色失败", formatError(error));
      }
    },

    addCharacter: async (input) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const character = await invoke<Character>("create_character", {
          projectId: snapshot.project.id,
          input,
        });
        set((state) => ({
          snapshot: state.snapshot
            ? {
                ...state.snapshot,
                characters: [...state.snapshot.characters, character],
              }
            : null,
          currentCharacterId: character.id,
        }));
        pushToast("success", "角色已创建", character.name);
      } catch (error) {
        pushToast("error", "创建角色失败", formatError(error));
      }
    },

    removeCharacter: async (characterId) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        await invoke("delete_character", { projectId: snapshot.project.id, characterId });
        const characters = snapshot.characters.filter((c) => c.id !== characterId);
        set({
          snapshot: { ...snapshot, characters },
          currentCharacterId: characters[0]?.id ?? null,
        });
        pushToast("success", "角色已删除");
      } catch (error) {
        pushToast("error", "删除角色失败", formatError(error));
      }
    },

    importCharacter: async (sourceProjectId, characterId) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const character = await invoke<Character>("import_character", {
          projectId: snapshot.project.id,
          sourceProjectId,
          characterId,
        });
        await get().refreshSnapshot(true);
        set({ currentCharacterId: character.id });
        pushToast("success", "角色已导入本项目", character.name);
      } catch (error) {
        pushToast("error", "导入角色失败", formatError(error));
      }
    },

    importStyle: async (sourceProjectId) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const project = await invoke<ProjectSnapshot["project"]>("import_project_style", {
          projectId: snapshot.project.id,
          sourceProjectId,
        });
        set((state) => ({
          snapshot: state.snapshot ? { ...state.snapshot, project } : null,
        }));
        await get().refreshProjects();
        pushToast("success", "项目风格已复制", "之后生成的镜头会自动使用新风格");
      } catch (error) {
        pushToast("error", "复制项目风格失败", formatError(error));
      }
    },

    setShotStatus: async (shotId, status) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const shot = await invoke<Shot>("set_shot_status", {
          projectId: snapshot.project.id,
          shotId,
          status,
        });
        set((state) => ({ snapshot: state.snapshot ? updateShotInSnapshot(state.snapshot, shot) : null }));
      } catch (error) {
        pushToast("error", "更新镜头状态失败", formatError(error));
      }
    },

    loadReferences: async () => {
      const snapshot = get().snapshot;
      if (!snapshot) return [];
      try {
        return await invoke<ProjectReference[]>("list_project_references", {
          projectId: snapshot.project.id,
        });
      } catch (error) {
        pushToast("error", "无法读取其他项目", formatError(error));
        return [];
      }
    },

    autoAssemble: async () => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const timeline = await invoke<Timeline>("auto_assemble_timeline", {
          projectId: snapshot.project.id,
        });
        set((state) => ({
          snapshot: state.snapshot ? { ...state.snapshot, timeline } : null,
          route: "editor",
        }));
        pushToast("success", "时间线已自动装配", `${timeline.clips.length} 个镜头`);
      } catch (error) {
        pushToast("error", "无法装配时间线", formatError(error));
      }
    },

    updateTimeline: async (patch) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const timeline = await invoke<Timeline>("update_timeline", {
          projectId: snapshot.project.id,
          patch,
        });
        set((state) => ({
          snapshot: state.snapshot ? { ...state.snapshot, timeline } : null,
        }));
      } catch (error) {
        pushToast("error", "保存时间线失败", formatError(error));
      }
    },

    exportTimeline: async (options) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const job = await invoke<GenerationJob>("export_timeline", {
          projectId: snapshot.project.id,
          options,
        });
        get().upsertJob(job);
        set({ jobDockOpen: true });
        pushToast("info", "导出已入队", "FFmpeg 将在后台渲染 MP4");
      } catch (error) {
        pushToast("error", "导出失败", formatError(error));
      }
    },

    exportProvenance: async () => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const path = await invoke<string>("export_provenance", {
          projectId: snapshot.project.id,
        });
        pushToast("success", "溯源清单已导出", path);
      } catch (error) {
        pushToast("error", "导出溯源清单失败", formatError(error));
      }
    },

    searchMusic: async (query) => {
      set({ musicSearching: true });
      try {
        const result = await invoke<MusicSearchResult>("search_music", {
          providerId: "jamendo-music",
          query,
          limit: 8,
        });
        set({ musicTracks: result.tracks, musicSearching: false });
        if (result.error) pushToast("error", "音乐搜索失败", result.error);
        else if (!result.tracks.length) pushToast("info", "没有找到匹配的音乐", "换个关键词，或直接在下方用 AI 生成");
      } catch (error) {
        set({ musicSearching: false });
        pushToast("error", "音乐供应商不可用", formatError(error));
      }
    },

    importMusicTrack: async (track) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const asset = await invoke<ProjectSnapshot["assets"][number]>("import_music_url", {
          projectId: snapshot.project.id,
          track,
        });
        set((state) => ({
          snapshot: state.snapshot ? { ...state.snapshot, assets: [asset, ...state.snapshot.assets] } : null,
        }));
        await get().attachMusic(asset.id);
        pushToast("success", "配乐已导入并应用", track.title);
      } catch (error) {
        pushToast("error", "导入配乐失败", formatError(error));
      }
    },

    generateMusic: async (request) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      set({ musicGenerating: true });
      try {
        const job = await invoke<GenerationJob>("generate_music", {
          projectId: snapshot.project.id,
          request: { ...request, providerId: request.providerId ?? "mock-music" },
        });
        get().upsertJob(job);
        set({ jobDockOpen: true, musicGenerating: false });
        pushToast("info", "AI 配乐已入队", request.prompt || "原创配乐");
      } catch (error) {
        set({ musicGenerating: false });
        pushToast("error", "无法生成配乐", formatError(error));
      }
    },

    analyzeBeats: async () => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const analysis = await invoke<BeatAnalysis>("analyze_music_beats", {
          projectId: snapshot.project.id,
        });
        set({ musicAnalysis: analysis });
        await get().refreshSnapshot(true);
        pushToast("success", `检测到 ${analysis.bpm} BPM`, `${Math.round(analysis.confidence * 100)}% 置信度`);
      } catch (error) {
        pushToast("error", "节拍分析失败", formatError(error));
      }
    },

    autoBeatSync: async () => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const result = await invoke<BeatSyncResult>("auto_beat_sync", {
          projectId: snapshot.project.id,
        });
        set((state) => ({
          snapshot: state.snapshot ? { ...state.snapshot, timeline: result.timeline } : null,
        }));
        pushToast("success", `已对齐 ${result.alignedCuts}/${result.totalCuts} 个切点`, `平均偏移 ${result.averageOffsetMs}ms`);
      } catch (error) {
        pushToast("error", "自动卡点失败", formatError(error));
      }
    },

    attachMusic: async (assetId, volume) => {
      const snapshot = get().snapshot;
      if (!snapshot) return;
      try {
        const timeline = await invoke<Timeline>("attach_music", {
          projectId: snapshot.project.id,
          assetId,
          volume: volume ?? snapshot.timeline.musicVolume ?? 0.8,
        });
        set((state) => ({
          snapshot: state.snapshot ? { ...state.snapshot, timeline } : null,
          musicAnalysis: null,
        }));
        pushToast("success", "已应用到时间线");
      } catch (error) {
        pushToast("error", "应用配乐失败", formatError(error));
      }
    },

    refreshSettings: async () => {
      try {
        set({ settings: await invoke<Settings>("get_settings") });
      } catch (error) {
        pushToast("error", "无法读取设置", formatError(error));
      }
    },

    updatePreferences: async (patch) => {
      try {
        const settings = await invoke<Settings>("update_preferences", { patch });
        set({ settings });
        pushToast("success", "偏好设置已保存");
      } catch (error) {
        pushToast("error", "保存设置失败", formatError(error));
      }
    },

    updateProvider: async (providerId, patch) => {
      try {
        const settings = await invoke<Settings>("update_provider", { providerId, patch });
        set({ settings });
      } catch (error) {
        pushToast("error", "更新供应商失败", formatError(error));
      }
    },

    saveProviderSecret: async (providerId, secret) => {
      try {
        const settings = await invoke<Settings>("set_provider_secret", { providerId, secret });
        set({ settings });
        pushToast("success", "API Key 已存入系统钥匙串");
      } catch (error) {
        pushToast("error", "保存 API Key 失败", formatError(error));
      }
    },

    clearProviderSecret: async (providerId) => {
      try {
        const settings = await invoke<Settings>("clear_provider_secret", { providerId });
        set({ settings });
        pushToast("success", "API Key 已移除");
      } catch (error) {
        pushToast("error", "移除 API Key 失败", formatError(error));
      }
    },

    testProvider: async (providerId) => {
      return invoke<ProviderTestResult>("test_provider", { providerId });
    },

    refreshSystem: async () => {
      try {
        set({ system: await invoke<SystemStatus>("get_system_status") });
      } catch (error) {
        pushToast("error", "无法读取系统状态", formatError(error));
      }
    },

    upsertJob: (job) => {
      set((state) => {
        if (!state.snapshot) return {};
        const jobs = [job, ...state.snapshot.jobs.filter((j) => j.id !== job.id)].slice(0, 80);
        return { snapshot: { ...state.snapshot, jobs } };
      });
    },

    handleJob: (job) => {
      const snapshot = get().snapshot;
      if (!snapshot || job.projectId !== snapshot.project.id) return;
      get().upsertJob(job);
      if (job.status === "succeeded") {
        if (job.kind === "export") {
          pushToast("success", "成片已导出", String(job.result?.path ?? ""));
        } else if (job.kind === "script") {
          pushToast("success", "剧本已写进项目", "分集、场景和分镜已同步");
        } else if (job.kind === "music") {
          set({ musicAnalysis: null });
          pushToast("success", "AI 配乐已完成", "可在剪辑台试听与卡点");
        } else {
          pushToast("success", job.kind === "video" ? "镜头成片已完成" : "素材已生成", job.requestSummary);
        }
        scheduleRefresh();
        void get().refreshProjects();
      } else if (job.status === "failed") {
        pushToast("error", "生成失败", job.error ?? "供应商没有返回结果");
      }
    },
  };
});

export const selectCurrentShot = (state: AppStore): Shot | null =>
  allShots(state.snapshot).find((shot) => shot.id === state.currentShotId) ?? null;

export const selectCurrentScene = (state: AppStore): Scene | null =>
  sceneOfShot(state.snapshot, state.currentShotId) ??
  state.snapshot?.episodes.flatMap((e) => e.scenes).find((s) => s.id === state.currentSceneId) ??
  null;

export const selectAllShots = allShots;
export const selectShotsOfScene = shotsOfScene;
