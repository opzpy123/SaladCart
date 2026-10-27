import { useMemo, useRef, useState } from "react";
import {
  AudioLines,
  Check,
  Clock3,
  Copy,
  Film,
  FolderOpen,
  Image as ImageIcon,
  Layers,
  Music,
  Search,
  Upload,
} from "lucide-react";
import {
  assetUrl,
  formatDuration,
  formatError,
  invoke,
  pickAssetPaths,
  relativeTime,
} from "@/lib/bridge";
import { useAppStore } from "@/store/useAppStore";
import type { Asset } from "@/lib/types";
import {
  Button,
  EmptyState,
  IconButton,
  Modal,
  Segmented,
  StatusPill,
  Thumb,
  cx,
} from "@/components/ui";

type Filter = "all" | "image" | "video" | "audio";

export function AssetLibrary() {
  const snapshot = useAppStore((s) => s.snapshot);
  const attachMusic = useAppStore((s) => s.attachMusic);
  const setRoute = useAppStore((s) => s.setRoute);
  const pushToast = useAppStore((s) => s.pushToast);
  const refreshSnapshot = useAppStore((s) => s.refreshSnapshot);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [filter, setFilter] = useState<Filter>("all");
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<Asset | null>(null);
  const [busy, setBusy] = useState(false);
  const [importing, setImporting] = useState(false);

  const assets = useMemo(() => {
    const list = snapshot?.assets ?? [];
    return list.filter(
      (asset) =>
        (filter === "all" || asset.kind === filter) &&
        (!query || asset.name.toLowerCase().includes(query.toLowerCase())),
    );
  }, [snapshot?.assets, filter, query]);

  if (!snapshot) return null;

  const kindFor = (name: string): "image" | "video" | "audio" => {
    const ext = name.split(".").pop()?.toLowerCase() ?? "";
    if (["mp4", "mov", "webm", "m4v"].includes(ext)) return "video";
    if (["mp3", "wav", "m4a", "aac", "flac"].includes(ext)) return "audio";
    return "image";
  };

  const importBatch = async (items: Array<{ sourcePath: string; kind: string; name: string }>) => {
    if (!items.length) return;
    setImporting(true);
    let done = 0;
    for (const item of items) {
      try {
        await invoke<Asset>("import_asset_path", {
          projectId: snapshot.project.id,
          input: item,
        });
        done += 1;
      } catch (error) {
        pushToast("error", "导入失败", `${item.name}：${formatError(error)}`);
      }
    }
    if (done) {
      await refreshSnapshot(true);
      pushToast("success", `已导入 ${done} 个素材`, "文件已复制进当前项目");
    }
    setImporting(false);
  };

  const startImport = async () => {
    const paths = await pickAssetPaths();
    if (paths === null) {
      fileInputRef.current?.click();
      return;
    }
    await importBatch(
      paths.map((sourcePath) => ({
        sourcePath,
        kind: kindFor(sourcePath),
        name: sourcePath.split(/[\\/]/).pop() ?? "导入素材",
      })),
    );
  };

  const onBrowserFiles = async (files: FileList | null) => {
    if (!files?.length) return;
    await importBatch(
      Array.from(files).map((file) => ({
        sourcePath: file.name,
        kind: kindFor(file.name),
        name: file.name,
      })),
    );
  };

  const approve = async (asset: Asset) => {
    setBusy(true);
    try {
      const updated = await invoke<Asset>("set_asset_status", {
        projectId: snapshot.project.id,
        assetId: asset.id,
        status: asset.status === "approved" ? "ready" : "approved",
      });
      useAppStore.setState((state) => ({
        snapshot: state.snapshot
          ? { ...state.snapshot, assets: state.snapshot.assets.map((a) => (a.id === updated.id ? updated : a)) }
          : null,
      }));
      setSelected(updated);
    } catch (error) {
      pushToast("error", "更新素材状态失败", String(error));
    } finally {
      setBusy(false);
    }
  };

  const applyAsMusic = async (asset: Asset) => {
    await attachMusic(asset.id);
    setSelected(null);
    setRoute("editor");
  };

  return (
    <div className="studio-page">
      <input
        ref={fileInputRef}
        type="file"
        multiple
        hidden
        accept=".png,.jpg,.jpeg,.webp,.mp4,.mov,.webm,.mp3,.wav,.m4a,.aac,.flac"
        onChange={(event) => {
          void onBrowserFiles(event.target.files);
          event.target.value = "";
        }}
      />
      <header className="workspace-head">
        <div>
          <p className="eyebrow">ASSETS / 素材库</p>
          <h1>每次抽卡，都有出处。</h1>
          <p className="workspace-head__sub">
            图片、视频、配乐统一进项目库，保留供应商、模型、seed、版本与状态；不再在文件夹里猜最新版。
          </p>
        </div>
        <div className="workspace-head__actions">
          <Button
            variant="secondary"
            icon={<Upload size={15} />}
            loading={importing}
            onClick={() => void startImport()}
          >
            导入素材
          </Button>
          <div className="head-stat"><span>图片</span><strong>{snapshot.assets.filter((a) => a.kind === "image").length}</strong></div>
          <div className="head-stat"><span>视频</span><strong>{snapshot.assets.filter((a) => a.kind === "video").length}</strong></div>
          <div className="head-stat"><span>音频</span><strong>{snapshot.assets.filter((a) => a.kind === "audio").length}</strong></div>
        </div>
      </header>

      <div className="asset-toolbar">
        <Segmented
          size="sm"
          value={filter}
          onChange={(value) => setFilter(value)}
          options={[
            { value: "all", label: "全部" },
            { value: "image", label: "图片" },
            { value: "video", label: "视频" },
            { value: "audio", label: "音频" },
          ]}
        />
        <div className="asset-search">
          <Search size={15} />
          <input
            className="input"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="按名称搜索素材"
          />
        </div>
      </div>

      {assets.length ? (
        <div className="asset-grid">
          {assets.map((asset) => (
            <button key={asset.id} className={cx("asset-card", `asset-card--${asset.kind}`)} onClick={() => setSelected(asset)}>
              <div className="asset-card__visual">
                {asset.kind === "audio" ? (
                  <div className="asset-audio">
                    <span className="asset-audio__wave">
                      {Array.from({ length: 42 }).map((_, i) => (
                        <i key={i} style={{ height: `${20 + Math.abs(Math.sin(i * 0.6)) * 65}%` }} />
                      ))}
                    </span>
                    <Music size={18} />
                  </div>
                ) : asset.thumbPath || asset.kind === "image" ? (
                  <img src={assetUrl(asset.thumbPath ?? asset.absPath)} alt={asset.name} loading="lazy" />
                ) : (
                  <div className="asset-card__fallback"><Film size={22} /></div>
                )}
                <span className="asset-kind">
                  {asset.kind === "image" ? <ImageIcon size={12} /> : asset.kind === "video" ? <Film size={12} /> : <AudioLines size={12} />}
                  {asset.kind === "image" ? "图片" : asset.kind === "video" ? "视频" : "音频"}
                </span>
                {asset.version > 1 ? <span className="asset-version">v{asset.version}</span> : null}
              </div>
              <div className="asset-card__body">
                <strong>{asset.name}</strong>
                <span>
                  {asset.providerId ?? asset.source} · {relativeTime(asset.createdAt)}
                </span>
                <div className="asset-card__foot">
                  <StatusPill status={asset.status} />
                  {asset.durationMs ? <em>{formatDuration(asset.durationMs)}</em> : null}
                </div>
              </div>
            </button>
          ))}
        </div>
      ) : (
        <EmptyState
          icon={<Layers size={26} />}
          title="素材库还是空的"
          description="生成角色、画面、视频或配乐后，素材会自动进入项目库并附带完整来源。"
          action={<Button variant="secondary" onClick={() => setRoute("shots")}>去分镜工坊</Button>}
        />
      )}

      <Modal
        open={Boolean(selected)}
        onClose={() => setSelected(null)}
        title={selected?.name ?? "素材详情"}
        subtitle={selected ? `${selected.kind.toUpperCase()} · ${selected.source} · ${relativeTime(selected.createdAt)}` : undefined}
        wide
        footer={
          selected ? (
            <>
              <Button
                variant="secondary"
                icon={<FolderOpen size={14} />}
                onClick={async () => {
                  await navigator.clipboard.writeText(selected.absPath).catch(() => undefined);
                  pushToast("success", "素材路径已复制");
                }}
              >
                复制路径
              </Button>
              {selected.kind === "audio" ? (
                <Button variant="secondary" icon={<Music size={14} />} onClick={() => void applyAsMusic(selected)}>
                  作为配乐应用
                </Button>
              ) : null}
              <Button variant="primary" icon={<Check size={14} />} loading={busy} onClick={() => void approve(selected)}>
                {selected.status === "approved" ? "取消通过" : "标记通过"}
              </Button>
            </>
          ) : null
        }
      >
        {selected ? (
          <div className="asset-detail">
            <div className="asset-detail__preview">
              {selected.kind === "image" ? (
                <img src={assetUrl(selected.absPath)} alt={selected.name} />
              ) : selected.kind === "video" ? (
                <video src={assetUrl(selected.absPath)} poster={assetUrl(selected.thumbPath)} controls />
              ) : (
                <div className="asset-detail__audio">
                  <AudioLines size={30} />
                  <audio src={assetUrl(selected.absPath)} controls />
                </div>
              )}
            </div>
            <div className="asset-detail__meta">
              <div className="meta-row"><span>状态</span><StatusPill status={selected.status} /></div>
              <div className="meta-row"><span>供应商</span><strong>{selected.providerId ?? "—"}</strong></div>
              <div className="meta-row"><span>模型</span><strong>{selected.model ?? "—"}</strong></div>
              <div className="meta-row"><span>Seed</span><strong>{selected.seed ?? "—"}</strong></div>
              <div className="meta-row"><span>版本</span><strong>v{selected.version}{selected.parentAssetId ? " · 有上一版" : ""}</strong></div>
              <div className="meta-row"><span>尺寸</span><strong>{selected.width ?? "—"}×{selected.height ?? "—"}</strong></div>
              <div className="meta-row"><span>时长</span><strong>{formatDuration(selected.durationMs)}</strong></div>
              <div className="meta-row"><span>路径</span><code>{selected.relPath}</code></div>
              {selected.prompt ? (
                <div className="asset-prompt">
                  <span>生成提示词</span>
                  <pre>{selected.prompt}</pre>
                </div>
              ) : null}
            </div>
          </div>
        ) : null}
      </Modal>
    </div>
  );
}
