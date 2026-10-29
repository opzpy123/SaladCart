import { useEffect, useMemo, useState } from "react";
import {
  Check,
  Cpu,
  ExternalLink,
  FolderOpen,
  HardDrive,
  KeyRound,
  Loader2,
  RefreshCw,
  Server,
  ShieldCheck,
  Terminal,
  X,
} from "lucide-react";
import { invoke } from "@/lib/bridge";
import { useAppStore } from "@/store/useAppStore";
import type { ProviderProfile, ProviderTestResult } from "@/lib/types";
import {
  Button,
  Field,
  InlineNotice,
  SectionHeader,
  Segmented,
  StatusPill,
  cx,
} from "@/components/ui";

type Kind = "llm" | "image" | "video" | "music";

const KIND_LABEL: Record<Kind, string> = {
  llm: "编剧 / LLM",
  image: "文生图",
  video: "视频生成",
  music: "音乐",
};

export function SettingsScreen() {
  const settings = useAppStore((s) => s.settings);
  const snapshot = useAppStore((s) => s.snapshot);
  const updateProject = useAppStore((s) => s.updateProject);
  const system = useAppStore((s) => s.system);
  const updatePreferences = useAppStore((s) => s.updatePreferences);
  const refreshSystem = useAppStore((s) => s.refreshSystem);
  const refreshSettings = useAppStore((s) => s.refreshSettings);
  const pushToast = useAppStore((s) => s.pushToast);
  const [kind, setKind] = useState<Kind>("llm");
  const [defaults, setDefaults] = useState({ llm: "", image: "", video: "", concurrency: 2 });
  const [ffmpegPath, setFfmpegPath] = useState("");
  const [budgetCap, setBudgetCap] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (snapshot) setBudgetCap(snapshot.project.budgetCap ? String(snapshot.project.budgetCap) : "");
  }, [snapshot?.project.id, snapshot?.project.budgetCap]);

  useEffect(() => {
    if (!settings) return;
    setDefaults({
      llm: settings.defaults.llm,
      image: settings.defaults.image,
      video: settings.defaults.video,
      concurrency: settings.concurrency,
    });
    setFfmpegPath(settings.ffmpegPath ?? system?.ffmpegPath ?? "");
  }, [settings, system?.ffmpegPath]);

  const providers = useMemo(
    () => settings?.providers.filter((p) => p.kind === kind) ?? [],
    [settings, kind],
  );
  const committed = useMemo(
    () =>
      (snapshot?.jobs ?? []).reduce(
        (sum, job) =>
          sum + (job.status === "queued" || job.status === "running" ? job.costEstimate : job.costActual),
        0,
      ),
    [snapshot?.jobs],
  );

  if (!settings) return null;

  const saveDefaults = async () => {
    setSaving(true);
    await updatePreferences({
      defaults: {
        llm: defaults.llm,
        image: defaults.image,
        video: defaults.video,
      },
      concurrency: defaults.concurrency,
    });
    setSaving(false);
  };

  const saveFfmpeg = async () => {
    setSaving(true);
    await updatePreferences({ ffmpegPath: ffmpegPath.trim(), clearFfmpegPath: !ffmpegPath.trim() });
    await refreshSystem();
    await refreshSettings();
    setSaving(false);
    pushToast("success", "FFmpeg 路径已更新");
  };

  const saveBudget = async () => {
    const value = Math.max(0, Number(budgetCap) || 0);
    setSaving(true);
    await updateProject({ budgetCap: value });
    setSaving(false);
    setBudgetCap(value ? String(value) : "");
    pushToast("success", value ? `项目预算已设为 ¥${value.toFixed(2)}` : "项目预算已关闭");
  };

  const openFolder = async () => {
    try {
      await invoke("open_storage_folder");
    } catch (error) {
      pushToast("info", "桌面版可以直接打开文件夹", String(error));
    }
  };

  return (
    <div className="studio-page settings-page">
      <header className="workspace-head">
        <div>
          <p className="eyebrow">SETTINGS / 工作站设置</p>
          <h1>所有创作，都留在本机。</h1>
          <p className="workspace-head__sub">
            API Key 存入系统钥匙串；项目、素材、时间线与成片保存在本地项目目录。没有分析埋点。
          </p>
        </div>
        <div className="workspace-head__actions">
          <StatusPill
            status={system?.ffmpegAvailable ? "ready" : "failed"}
            label={system?.ffmpegAvailable ? "FFmpeg 就绪" : "FFmpeg 未安装"}
          />
          <Button
            variant="secondary"
            icon={<RefreshCw size={15} />}
            onClick={() => {
              void refreshSystem();
              void refreshSettings();
            }}
          >
            重新检测
          </Button>
        </div>
      </header>

      <div className="settings-layout">
        <main className="settings-main">
          <section className="panel settings-section">
            <SectionHeader
              title="AI 供应商"
              subtitle="模型、接口与密钥按工作流分工。默认供应商用于新任务，单次任务也可覆盖。"
              action={
                <Segmented
                  size="sm"
                  value={kind}
                  onChange={(value) => setKind(value)}
                  options={(Object.keys(KIND_LABEL) as Kind[]).map((value) => ({
                    value,
                    label: KIND_LABEL[value],
                  }))}
                />
              }
            />
            <div className="provider-list">
              {providers.map((provider) => (
                <ProviderCard key={provider.id} provider={provider} />
              ))}
            </div>
            {kind === "music" ? (
              <InlineNotice tone="info">
                曲库搜索使用 Jamendo Client ID；AI 配乐可接 Stability Audio / ElevenLabs / 火山引擎纯音乐。Mock 会本地合成 100 BPM
                的原创打击乐，用来验证卡点与导出。
              </InlineNotice>
            ) : null}
          </section>

          {snapshot ? (
            <section className="panel settings-section">
              <SectionHeader
                title="项目预算"
                subtitle="0 表示不限额。已占用包含排队/运行中的估算与已完成任务的实际成本；超额生成会在入队时被拦截。"
              />
              <div className="settings-form-grid">
                <Field label="预算上限（¥）">
                  <input
                    className="input"
                    type="number"
                    min="0"
                    step="0.01"
                    value={budgetCap}
                    onChange={(event) => setBudgetCap(event.target.value)}
                    placeholder="0 = 不限额"
                  />
                </Field>
                <Field label="当前项目已占用（¥）">
                  <input className="input" readOnly value={committed.toFixed(2)} />
                </Field>
              </div>
              <div className="settings-actions">
                <Button variant="secondary" loading={saving} onClick={() => void saveBudget()}>
                  保存预算
                </Button>
              </div>
            </section>
          ) : null}

          <section className="panel settings-section">
            <SectionHeader title="默认路由" subtitle="决定没有特别指定时，剧本、画面和视频走哪个供应商。" />
            <div className="settings-form-grid">
              <Field label="默认编剧">
                <select className="input" value={defaults.llm} onChange={(e) => setDefaults({ ...defaults, llm: e.target.value })}>
                  {settings.providers.filter((p) => p.kind === "llm" && p.enabled).map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.label}{p.id.startsWith("mock-") ? " · 离线" : p.apiKeySet ? " · 已配置 Key" : ""}
                    </option>
                  ))}
                </select>
              </Field>
              <Field label="默认图像">
                <select className="input" value={defaults.image} onChange={(e) => setDefaults({ ...defaults, image: e.target.value })}>
                  {settings.providers.filter((p) => p.kind === "image" && p.enabled).map((p) => (
                    <option key={p.id} value={p.id}>{p.label}</option>
                  ))}
                </select>
              </Field>
              <Field label="默认视频">
                <select className="input" value={defaults.video} onChange={(e) => setDefaults({ ...defaults, video: e.target.value })}>
                  {settings.providers.filter((p) => p.kind === "video" && p.enabled).map((p) => (
                    <option key={p.id} value={p.id}>{p.label}</option>
                  ))}
                </select>
              </Field>
              <Field label="任务并发" hint="同时向供应商发起的任务数，过高会触发限流">
                <select
                  className="input"
                  value={defaults.concurrency}
                  onChange={(e) => setDefaults({ ...defaults, concurrency: Number(e.target.value) })}
                >
                  {[1, 2, 3, 4].map((n) => (
                    <option key={n} value={n}>{n} 个并行</option>
                  ))}
                </select>
              </Field>
            </div>
            <div className="settings-actions">
              <Button variant="primary" loading={saving} icon={<Check size={15} />} onClick={saveDefaults}>
                保存默认路由
              </Button>
            </div>
          </section>

          <section className="panel settings-section">
            <SectionHeader title="FFmpeg 与本地媒体" subtitle="生成视频的探测、缩略图、字幕烧录与 MP4 导出都依赖本机 FFmpeg。" />
            <div className="ffmpeg-status">
              <div className={cx("ffmpeg-status__icon", system?.ffmpegAvailable ? "is-good" : "is-bad")}>
                <Terminal size={19} />
              </div>
              <div>
                <strong>{system?.ffmpegAvailable ? "FFmpeg 已就绪" : "未找到 FFmpeg"}</strong>
                <small>{system?.ffmpegVersion ?? "安装后点击重新检测，或手动指定可执行文件路径。"}</small>
                {system?.ffmpegPath ? <code>{system.ffmpegPath}</code> : null}
              </div>
            </div>
            <div className="ffmpeg-form">
              <Field label="FFmpeg 路径" hint="留空时自动从 PATH、Homebrew 或常见安装目录查找">
                <input
                  className="input"
                  value={ffmpegPath}
                  onChange={(e) => setFfmpegPath(e.target.value)}
                  placeholder="/opt/homebrew/bin/ffmpeg"
                />
              </Field>
              <Button variant="secondary" loading={saving} onClick={saveFfmpeg}>
                保存路径
              </Button>
            </div>
            {!system?.ffmpegAvailable ? (
              <InlineNotice tone="warn">
                macOS 可安装：<code>brew install ffmpeg</code>；Windows 可安装：<code>winget install Gyan.FFmpeg</code>。
                未安装时仍可使用 Mock 图片和剧本，但无法生成视频或导出。
              </InlineNotice>
            ) : null}
          </section>

          <section className="panel settings-section">
            <SectionHeader title="项目与存储" subtitle="每个项目独立一个 SQLite 与素材目录，可以整个文件夹迁移。" />
            <div className="storage-card">
              <HardDrive size={20} />
              <div>
                <strong>{system?.dataDir ?? settings.storageDir ?? "本地应用数据目录"}</strong>
                <small>{system?.projectCount ?? 0} 个项目 · {system?.platform ?? ""}</small>
              </div>
              <Button size="sm" variant="secondary" icon={<FolderOpen size={14} />} onClick={() => void openFolder()}>
                打开文件夹
              </Button>
            </div>
            <div className="storage-rules">
              <div><ShieldCheck size={15} /> API Key 只进系统钥匙串，不写进项目</div>
              <div><HardDrive size={15} /> 项目移除时移入回收区，不直接删除</div>
              <div><Cpu size={15} /> 媒体处理由 Rust + FFmpeg 完成，不占用 WebView 主线程</div>
            </div>
          </section>
        </main>

        <aside className="settings-side">
          <section className="panel settings-about">
            <img src="/brand/icon.png" alt="SaladCart" />
            <p className="eyebrow">SALADCART DESKTOP</p>
            <h2>一台为 AI 漫剧准备的工作站。</h2>
            <p>
              剧本、角色、分镜、视频、配乐和剪辑共享同一个项目图。上下文自动编译、角色锁定、成本预估、任务队列与
              素材溯源都在本机完成。
            </p>
            <div className="about-stats">
              <div><span>应用版本</span><strong>{system?.appVersion ?? "0.4.3"}</strong></div>
              <div><span>运行平台</span><strong>{system?.platform ?? "macOS / Windows"}</strong></div>
              <div><span>项目数</span><strong>{system?.projectCount ?? 0}</strong></div>
            </div>
            <div className="about-note">
              <p><strong>原生架构</strong> Tauri 2 + Rust + 系统 WebView，macOS 与 Windows 共用一套代码与项目格式。</p>
              <p><strong>隐私</strong> 无遥测、无云端账户。只有你主动点击生成时，编译后的上下文才会发送给所选供应商。</p>
            </div>
          </section>
          <section className="panel settings-shortcuts">
            <SectionHeader title="工作流约定" />
            <ol>
              <li><span>01</span> 先锁定角色与风格，再批量生成画面。</li>
              <li><span>02</span> 分镜视频生成前检查上下文与成本。</li>
              <li><span>03</span> 选中要用的 take，标记通过后再装配。</li>
              <li><span>04</span> 配乐先分析 BPM，再一键卡点、导出。</li>
            </ol>
          </section>
        </aside>
      </div>
    </div>
  );
}

function ProviderCard({ provider }: { provider: ProviderProfile }) {
  const updateProvider = useAppStore((s) => s.updateProvider);
  const saveSecret = useAppStore((s) => s.saveProviderSecret);
  const clearSecret = useAppStore((s) => s.clearProviderSecret);
  const testProvider = useAppStore((s) => s.testProvider);
  const [base, setBase] = useState(provider.apiBase);
  const [model, setModel] = useState(provider.defaultModel);
  const [groupId, setGroupId] = useState(String(provider.options.groupId ?? ""));
  const [secret, setSecret] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<ProviderTestResult | null>(null);
  const [savingKey, setSavingKey] = useState(false);
  const local = provider.id.startsWith("mock-") || provider.vendor === "ollama" || provider.id === "comfyui-image";

  useEffect(() => {
    setBase(provider.apiBase);
    setModel(provider.defaultModel);
    setGroupId(String(provider.options.groupId ?? ""));
  }, [provider]);

  const saveConfig = async () => {
    await updateProvider(provider.id, {
      apiBase: base,
      defaultModel: model,
      models: [model],
      options: provider.id === "minimax-video" ? { ...provider.options, groupId } : provider.options,
    });
  };

  const saveKey = async () => {
    if (!secret.trim()) return;
    setSavingKey(true);
    await saveSecret(provider.id, secret.trim());
    setSecret("");
    setSavingKey(false);
  };

  const test = async () => {
    setTesting(true);
    try {
      const result = await testProvider(provider.id);
      setTestResult(result);
    } finally {
      setTesting(false);
    }
  };

  return (
    <article className={cx("provider-card", !provider.enabled && "is-disabled")}>
      <div className="provider-card__head">
        <div className="provider-card__avatar">
          {provider.kind === "music" ? "♪" : provider.kind === "video" ? <Server size={18} /> : provider.kind === "image" ? "◈" : "✎"}
        </div>
        <div className="provider-card__title">
          <strong>{provider.label}</strong>
          <small>{provider.id}</small>
        </div>
        <label className="toggle">
          <input
            type="checkbox"
            checked={provider.enabled}
            onChange={(event) => void updateProvider(provider.id, { enabled: event.target.checked })}
          />
          <span />
        </label>
      </div>

      <div className="provider-card__body">
        <div className="provider-card__row">
          <span className="provider-label">接口</span>
          <input className="input input--mono" value={base} onChange={(e) => setBase(e.target.value)} />
        </div>
        <div className="provider-card__row">
          <span className="provider-label">默认模型</span>
          <input className="input input--mono" value={model} onChange={(e) => setModel(e.target.value)} />
        </div>
        {provider.id === "minimax-video" ? (
          <div className="provider-card__row">
            <span className="provider-label">Group ID</span>
            <input className="input input--mono" value={groupId} onChange={(e) => setGroupId(e.target.value)} placeholder="MiniMax GroupId" />
          </div>
        ) : null}
        {!local ? (
          <div className="provider-card__key">
            <div className="provider-card__key-head">
              <span>
                <KeyRound size={13} /> {provider.id === "volc-music" ? "AccessKey:SecretKey" : "API Key"}
              </span>
              {provider.apiKeySet ? (
                <em className="key-state key-state--set"><Check size={11} /> 已存入钥匙串</em>
              ) : (
                <em className="key-state">未配置</em>
              )}
            </div>
            <div className="provider-card__key-row">
              <input
                className="input input--mono"
                type="password"
                value={secret}
                onChange={(e) => setSecret(e.target.value)}
                placeholder={
                  provider.apiKeySet
                    ? "••••••••  输入新 Key 可替换"
                    : provider.id === "volc-music"
                      ? "AccessKey:SecretKey（IAM 访问密钥，不是方舟 API Key）"
                      : provider.id === "kling-video"
                        ? "API Key 或 AccessKey:SecretKey"
                        : provider.envVar || "粘贴 API Key"
                }
              />
              <Button size="sm" variant="secondary" loading={savingKey} onClick={saveKey} disabled={!secret.trim()}>
                保存
              </Button>
              {provider.apiKeySet ? (
                <Button size="sm" variant="ghost" icon={<X size={13} />} onClick={() => void clearSecret(provider.id)}>
                  清除
                </Button>
              ) : null}
            </div>
          </div>
        ) : (
          <div className="provider-card__local">
            <ShieldCheck size={14} /> 无需 API Key，运行在你的机器上或使用离线 Mock。
          </div>
        )}
      </div>

      <div className="provider-card__foot">
        <div className="provider-card__test">
          {testing ? (
            <span><Loader2 size={13} className="spin" /> 检测中…</span>
          ) : testResult ? (
            <span className={testResult.ok ? "is-good" : "is-bad"}>
              {testResult.ok ? <Check size={13} /> : <X size={13} />} {testResult.message}
            </span>
          ) : (
            <span className="muted">Key 只写入系统钥匙串，界面永远不回显。</span>
          )}
        </div>
        <div className="provider-card__actions">
          <Button size="sm" variant="ghost" onClick={test} disabled={testing}>
            检测
          </Button>
          <Button size="sm" variant="secondary" onClick={() => void saveConfig()}>
            保存配置
          </Button>
        </div>
      </div>
    </article>
  );
}
