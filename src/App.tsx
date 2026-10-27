import { useEffect } from "react";
import {
  AudioLines,
  Clapperboard,
  Film,
  FolderOpen,
  LayoutDashboard,
  Loader2,
  Plus,
  Settings,
  Sparkles,
  Users,
  Layers,
} from "lucide-react";
import { useAppStore } from "@/store/useAppStore";
import type { Route } from "@/lib/types";
import { ProjectDashboard } from "@/screens/Dashboard";
import { ScriptStudio } from "@/screens/ScriptStudio";
import { CharacterStudio } from "@/screens/CharacterStudio";
import { ShotStudio } from "@/screens/ShotStudio";
import { EditorScreen } from "@/screens/Editor";
import { AssetLibrary } from "@/screens/AssetLibrary";
import { SettingsScreen } from "@/screens/SettingsScreen";
import { NewProjectDialog, GenerateScriptDialog } from "@/components/Dialogs";
import { ContextDrawer } from "@/components/ContextDrawer";
import { JobDock } from "@/components/JobDock";
import { Toasts } from "@/components/Toasts";
import { Button, IconButton, cx } from "@/components/ui";
import { isTauri } from "@/lib/bridge";

const SCREEN_LABEL: Record<Route, string> = {
  dashboard: "工作台",
  script: "剧本工坊",
  characters: "角色工坊",
  shots: "分镜与生成",
  editor: "剪辑台",
  assets: "素材库",
  settings: "设置",
};

const NAV = [
  { route: "dashboard" as Route, label: "概览", icon: LayoutDashboard, section: "工作台" },
  { route: "script" as Route, label: "剧本", icon: Sparkles, section: "创作" },
  { route: "characters" as Route, label: "角色", icon: Users, section: "创作" },
  { route: "shots" as Route, label: "分镜", icon: Clapperboard, section: "创作" },
  { route: "editor" as Route, label: "剪辑", icon: Film, section: "创作" },
  { route: "assets" as Route, label: "素材", icon: Layers, section: "资源" },
];

export default function App() {
  const init = useAppStore((s) => s.init);
  const initialized = useAppStore((s) => s.initialized);
  const fatalError = useAppStore((s) => s.fatalError);
  const loading = useAppStore((s) => s.loading);
  const route = useAppStore((s) => s.route);
  const snapshot = useAppStore((s) => s.snapshot);
  const settings = useAppStore((s) => s.settings);
  const system = useAppStore((s) => s.system);
  const activeJobs = snapshot?.jobs.filter((job) => job.status === "running" || job.status === "queued") ?? [];

  useEffect(() => {
    void init();
  }, [init]);

  useEffect(() => {
    document.title = snapshot ? `${snapshot.project.name} · 沙拉车` : "沙拉车 SaladCart";
  }, [snapshot]);

  if (!initialized) {
    return (
      <div className="splash">
        <div className="splash__mark">
          <img src="/brand/icon.png" alt="SaladCart" />
          <span className="splash__ring" />
        </div>
        <p className="eyebrow">SALADCART / AI COMIC DRAMA WORKSTATION</p>
        <h1>正在装车。</h1>
        <p>读取本机项目、供应商与素材……</p>
      </div>
    );
  }

  if (fatalError) {
    return (
      <div className="splash splash--error">
        <img src="/brand/icon.png" alt="" className="splash__icon" />
        <p className="eyebrow">STARTUP ERROR</p>
        <h1>工作站没能启动。</h1>
        <p>{fatalError}</p>
        <Button variant="primary" onClick={() => window.location.reload()}>
          重新加载
        </Button>
      </div>
    );
  }

  const needsProject = !["dashboard", "settings"].includes(route);
  const screen = needsProject && !snapshot ? (
    <div className="studio-page no-project">
      <div className="no-project__card panel">
        <img src="/brand/icon.png" alt="" />
        <p className="eyebrow">NO PROJECT OPEN</p>
        <h1>先打开一部漫剧。</h1>
        <p>项目包含自己的剧本、角色、镜头、素材与时间线。可以从概览打开已有项目，或新建一部。</p>
        <div>
          <Button variant="primary" icon={<Plus size={15} />} onClick={() => useAppStore.getState().setShowNewProject(true)}>
            新建项目
          </Button>
          <Button variant="ghost" onClick={() => useAppStore.getState().setRoute("dashboard")}>
            回到概览
          </Button>
        </div>
      </div>
    </div>
  ) : route === "dashboard" ? (
    <ProjectDashboard />
  ) : route === "settings" ? (
    <SettingsScreen />
  ) : route === "script" ? (
    <ScriptStudio />
  ) : route === "characters" ? (
    <CharacterStudio />
  ) : route === "shots" ? (
    <ShotStudio />
  ) : route === "editor" ? (
    <EditorScreen />
  ) : route === "assets" ? (
    <AssetLibrary />
  ) : null;

  return (
    <div className="app-shell">
      <Sidebar activeJobs={activeJobs.length} />
      <div className="app-main">
        <AppBar activeJobs={activeJobs.length} />
        {loading ? <div className="loading-bar"><span /></div> : null}
        <main className="app-content">{screen}</main>
      </div>
      <ContextDrawer />
      <JobDock />
      <NewProjectDialog />
      <GenerateScriptDialog />
      <Toasts />
    </div>
  );
}

function Sidebar({ activeJobs }: { activeJobs: number }) {
  const route = useAppStore((s) => s.route);
  const setRoute = useAppStore((s) => s.setRoute);
  const snapshot = useAppStore((s) => s.snapshot);
  const projects = useAppStore((s) => s.projects);
  const openProject = useAppStore((s) => s.openProject);
  const setNewProject = useAppStore((s) => s.setShowNewProject);
  const settings = useAppStore((s) => s.settings);
  const system = useAppStore((s) => s.system);
  const setJobDock = useAppStore((s) => s.setJobDockOpen);
  const sections = ["工作台", "创作", "资源"];
  const keyed = settings?.providers.filter((p) => p.apiKeySet).length ?? 0;

  return (
    <aside className="sidebar">
      <div className="sidebar__brand">
        <img src="/brand/icon.png" alt="SaladCart" />
        <div>
          <strong>沙拉车</strong>
          <span>SALADCART</span>
        </div>
        <em className={isTauri ? "sidebar__mode" : "sidebar__mode is-web"}>{isTauri ? "DESKTOP" : "PREVIEW"}</em>
      </div>

      <div className="project-switch">
        <p className="sidebar__label">当前项目</p>
        <div className="project-switch__row">
          <select
            className="project-switch__select"
            value={snapshot?.project.id ?? ""}
            onChange={(event) => {
              if (event.target.value) void openProject(event.target.value);
            }}
          >
            {!snapshot ? <option value="">选择一个项目</option> : null}
            {projects.map((project) => (
              <option key={project.id} value={project.id}>
                {project.name}
              </option>
            ))}
          </select>
          <button className="project-switch__add" onClick={() => setNewProject(true)} aria-label="新建项目" title="新建项目">
            <Plus size={15} />
          </button>
        </div>
        {snapshot ? (
          <div className="project-switch__meta">
            <span>{snapshot.project.aspectRatio}</span>
            <span>{snapshot.project.genre || "漫剧"}</span>
            <span>{snapshot.episodes.length} 集</span>
          </div>
        ) : (
          <div className="project-switch__meta"><span>没有打开的项目</span></div>
        )}
      </div>

      <nav className="sidebar__nav">
        {sections.map((section) => (
          <div className="sidebar__section" key={section}>
            <p className="sidebar__label">{section}</p>
            {NAV.filter((item) => item.section === section).map((item) => {
              const Icon = item.icon;
              const disabled = !["dashboard", "settings"].includes(item.route) && !snapshot;
              return (
                <button
                  key={item.route}
                  className={cx("sidebar__item", route === item.route && "is-active")}
                  onClick={() => setRoute(item.route)}
                  disabled={disabled}
                >
                  <Icon size={17} />
                  <span>{item.label}</span>
                  {item.route === "script" && snapshot?.episodes.length ? <em>{snapshot.episodes.length}</em> : null}
                  {item.route === "shots" && snapshot?.episodes.length ? <em>{snapshot.episodes.flatMap((e) => e.scenes).flatMap((s) => s.shots).length}</em> : null}
                  {item.route === "editor" && snapshot?.timeline.clips.length ? <em>{snapshot.timeline.clips.length}</em> : null}
                </button>
              );
            })}
          </div>
        ))}
      </nav>

      <div className="sidebar__bottom">
        <button className={cx("sidebar__item", route === "settings" && "is-active")} onClick={() => setRoute("settings")}>
          <Settings size={17} />
          <span>设置</span>
        </button>
        <div className="sidebar__status">
          <button onClick={() => setJobDock(true)}>
            <span className={cx("status-dot", activeJobs ? "is-live" : "is-good")} />
            <span>{activeJobs ? `${activeJobs} 个任务运行中` : "任务队列空闲"}</span>
          </button>
          <button onClick={() => setRoute("settings")}>
            <span className={cx("status-dot", system?.ffmpegAvailable ? "is-good" : "is-bad")} />
            <span>FFmpeg {system?.ffmpegAvailable ? "就绪" : "未安装"}</span>
          </button>
          <div className="sidebar__foot">
            <span>{keyed ? `${keyed} 个 Key` : "Mock 就绪"}</span>
            <em>v{system?.appVersion ?? "0.1.0"}</em>
          </div>
        </div>
      </div>
    </aside>
  );
}

function AppBar({ activeJobs }: { activeJobs: number }) {
  const route = useAppStore((s) => s.route);
  const snapshot = useAppStore((s) => s.snapshot);
  const setNewProject = useAppStore((s) => s.setShowNewProject);
  const setJobDock = useAppStore((s) => s.setJobDockOpen);
  const setRoute = useAppStore((s) => s.setRoute);
  return (
    <header className="app-bar">
      <div className="app-bar__crumbs">
        <span>沙拉车</span>
        <i>/</i>
        {snapshot && route !== "dashboard" && route !== "settings" ? (
          <>
            <button onClick={() => setRoute("dashboard")}>{snapshot.project.name}</button>
            <i>/</i>
          </>
        ) : null}
        <strong>{SCREEN_LABEL[route]}</strong>
      </div>
      <div className="app-bar__right">
        <button className={cx("job-trigger", activeJobs > 0 && "is-live")} onClick={() => setJobDock(true)}>
          {activeJobs ? <Loader2 size={14} className="spin" /> : <AudioLines size={14} />}
          <span>{activeJobs ? `${activeJobs} 个任务` : "任务"}</span>
        </button>
        <Button size="sm" variant="secondary" icon={<Plus size={14} />} onClick={() => setNewProject(true)}>
          新项目
        </Button>
        <IconButton label="设置" onClick={() => setRoute("settings")} active={route === "settings"}>
          <Settings size={17} />
        </IconButton>
      </div>
    </header>
  );
}
