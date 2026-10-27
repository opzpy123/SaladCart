import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import {
  AlertTriangle,
  Check,
  Copy,
  Film,
  Image as ImageIcon,
  Layers,
  Sparkles,
  X,
} from "lucide-react";
import { assetUrl, formatCurrency, formatDuration } from "@/lib/bridge";
import { useAppStore } from "@/store/useAppStore";
import { Button, EmptyState, IconButton, InlineNotice, Skeleton, cx } from "./ui";

export function ContextDrawer() {
  const open = useAppStore((s) => s.contextOpen);
  const loading = useAppStore((s) => s.contextLoading);
  const preview = useAppStore((s) => s.contextPreview);
  const close = useAppStore((s) => s.closeContext);
  const generateShot = useAppStore((s) => s.generateShot);
  const pushToast = useAppStore((s) => s.pushToast);
  const [tab, setTab] = useState<"prompt" | "refs" | "checks">("prompt");

  useEffect(() => {
    if (open) setTab("prompt");
  }, [open]);

  if (!open) return null;

  const copy = async () => {
    if (!preview) return;
    try {
      await navigator.clipboard.writeText(
        `${preview.systemPrompt}\n\n${preview.userPrompt}\n\nNEGATIVE:\n${preview.negativePrompt}`,
      );
      pushToast("success", "提示词已复制");
    } catch {
      pushToast("error", "复制失败", "请手动选择文本复制");
    }
  };

  const generate = async (kind: "image" | "video") => {
    if (!preview || preview.targetType !== "shot") return;
    await generateShot(preview.targetId, kind);
    close();
  };

  return createPortal(
    <div className="drawer-backdrop" onMouseDown={(event) => event.target === event.currentTarget && close()}>
      <aside className="drawer">
        <header className="drawer__head">
          <div>
            <p className="eyebrow">CONTEXT COMPILER</p>
            <h2>{preview?.title ?? "正在编译上下文"}</h2>
            <p className="drawer__sub">
              {preview ? `${preview.providerLabel} · ${preview.model}` : "读取剧本、角色锁定与连续性…"}
            </p>
          </div>
          <IconButton label="关闭" onClick={close}>
            <X size={18} />
          </IconButton>
        </header>

        {loading || !preview ? (
          <div className="drawer__body">
            <Skeleton lines={4} />
            <Skeleton lines={7} />
          </div>
        ) : (
          <>
            <div className="context-summary">
              <div>
                <span>预估成本</span>
                <strong>{formatCurrency(preview.estimatedCost)}</strong>
              </div>
              <div>
                <span>画幅</span>
                <strong>{preview.aspectRatio}</strong>
              </div>
              <div>
                <span>时长</span>
                <strong>{preview.durationMs ? formatDuration(preview.durationMs) : "静态"}</strong>
              </div>
              <div>
                <span>参考图</span>
                <strong>{preview.characterRefs.length}</strong>
              </div>
            </div>

            <div className="drawer-tabs">
              <button className={cx(tab === "prompt" && "is-active")} onClick={() => setTab("prompt")}>
                <Sparkles size={14} /> 最终提示词
              </button>
              <button className={cx(tab === "refs" && "is-active")} onClick={() => setTab("refs")}>
                <ImageIcon size={14} /> 参考与连续性
              </button>
              <button className={cx(tab === "checks" && "is-active")} onClick={() => setTab("checks")}>
                <AlertTriangle size={14} /> 检查项
                {preview.warnings.length ? <em>{preview.warnings.length}</em> : null}
              </button>
            </div>

            <div className="drawer__body">
              {tab === "prompt" ? (
                <>
                  <div className="context-note">
                    这就是即将发送给供应商的内容。项目里的剧本、角色锁定与连续性已自动拼入，不需要手工复制。
                  </div>
                  <section className="prompt-card">
                    <div className="prompt-card__head">
                      <span>系统指令</span>
                      <Button size="sm" variant="ghost" icon={<Copy size={13} />} onClick={copy}>
                        复制全部
                      </Button>
                    </div>
                    <p>{preview.systemPrompt}</p>
                  </section>
                  <section className="prompt-card">
                    <div className="prompt-card__head">
                      <span>生成指令</span>
                      <em>≈ {preview.tokenEstimate} tokens</em>
                    </div>
                    <pre>{preview.userPrompt}</pre>
                  </section>
                  <section className="prompt-card prompt-card--negative">
                    <div className="prompt-card__head">
                      <span>负面提示词</span>
                    </div>
                    <pre>{preview.negativePrompt || "（未设置）"}</pre>
                  </section>
                </>
              ) : null}

              {tab === "refs" ? (
                <>
                  <div className="context-note">
                    <Layers size={15} /> 角色参考图会按顺序附加；视频会优先使用上一镜头的尾帧作为首帧。
                  </div>
                  {preview.characterRefs.length ? (
                    <div className="ref-grid">
                      {preview.characterRefs.map((ref) => (
                        <figure key={`${ref.id}-${ref.role}`} className="ref-card">
                          <img src={assetUrl(ref.absPath)} alt={ref.name} />
                          <figcaption>
                            <strong>{ref.name}</strong>
                            <span>{roleLabel(ref.role)}</span>
                          </figcaption>
                        </figure>
                      ))}
                    </div>
                  ) : (
                    <EmptyState
                      compact
                      icon={<ImageIcon size={20} />}
                      title="没有参考图"
                      description="先在角色工坊生成定妆照，视频生成时会自动附加。"
                    />
                  )}
                  {preview.continuityNote ? (
                    <InlineNotice tone="good">{preview.continuityNote}</InlineNotice>
                  ) : null}
                  {preview.notes.map((note) => (
                    <InlineNotice key={note} tone="info">
                      {note}
                    </InlineNotice>
                  ))}
                </>
              ) : null}

              {tab === "checks" ? (
                <>
                  {preview.warnings.length ? (
                    preview.warnings.map((warning) => (
                      <InlineNotice key={warning} tone="warn">
                        {warning}
                      </InlineNotice>
                    ))
                  ) : (
                    <InlineNotice tone="good">
                      <Check size={14} /> 角色、连续性与画幅检查通过。
                    </InlineNotice>
                  )}
                  {preview.notes.map((note) => (
                    <InlineNotice key={note} tone="info">
                      {note}
                    </InlineNotice>
                  ))}
                </>
              ) : null}
            </div>

            <footer className="drawer__foot">
              <span>确认无误后再生成，避免为无效上下文付费。</span>
              <div>
                <Button variant="secondary" icon={<ImageIcon size={15} />} onClick={() => generate("image")}>
                  生成画面
                </Button>
                <Button variant="primary" icon={<Film size={15} />} onClick={() => generate("video")}>
                  生成视频
                </Button>
              </div>
            </footer>
          </>
        )}
      </aside>
    </div>,
    document.body,
  );
}

function roleLabel(role: string): string {
  switch (role) {
    case "character_reference":
      return "角色定妆照";
    case "character_sheet":
      return "角色三视图";
    case "existing_reference":
      return "已有参考";
    case "user_reference":
      return "手动参考";
    default:
      return role;
  }
}
