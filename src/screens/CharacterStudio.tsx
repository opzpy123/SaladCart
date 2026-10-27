import { useEffect, useState } from "react";
import {
  Check,
  Download,
  Image as ImageIcon,
  Layers,
  Lock,
  Plus,
  Sparkles,
  Trash2,
  UserRound,
  Wand2,
} from "lucide-react";
import { assetUrl } from "@/lib/bridge";
import { useAppStore } from "@/store/useAppStore";
import type { Character, CreateCharacterInput, ProjectReference } from "@/lib/types";
import {
  Button,
  EmptyState,
  Field,
  IconButton,
  InlineNotice,
  Modal,
  SectionHeader,
  StatusPill,
  Thumb,
  cx,
} from "@/components/ui";

export function CharacterStudio() {
  const snapshot = useAppStore((s) => s.snapshot);
  const currentId = useAppStore((s) => s.currentCharacterId);
  const select = useAppStore((s) => s.selectCharacter);
  const generate = useAppStore((s) => s.generateCharacter);
  const [addOpen, setAddOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const current = snapshot?.characters.find((c) => c.id === currentId) ?? snapshot?.characters[0] ?? null;

  if (!snapshot) return null;

  return (
    <div className="studio-page">
      <header className="workspace-head">
        <div>
          <p className="eyebrow">CHARACTERS / 角色工坊</p>
          <h1>把脸、衣服和气质锁住。</h1>
          <p className="workspace-head__sub">
            角色设定 + 定妆照 + 三视图会一直跟着项目走；生成任何镜头时自动注入。
          </p>
        </div>
        <div className="workspace-head__actions">
          <div className="head-stat">
            <span>角色</span>
            <strong>{snapshot.characters.length}</strong>
          </div>
          <div className="head-stat">
            <span>已定妆</span>
            <strong>{snapshot.characters.filter((c) => c.referenceAssetId).length}</strong>
          </div>
          <Button variant="secondary" icon={<Download size={15} />} onClick={() => setImportOpen(true)}>
            从其他项目导入
          </Button>
          <Button variant="primary" icon={<Plus size={15} />} onClick={() => setAddOpen(true)}>
            新建角色
          </Button>
        </div>
      </header>

      <div className="character-layout">
        <main className="panel character-board">
          <SectionHeader
            title="角色圣经"
            subtitle="点击角色卡片编辑。锁定特征会进入每一次生成上下文。"
            action={
              <span className="muted small">
                {snapshot.characters.filter((c) => c.lockedTraits.length).length} 个角色已锁定外观
              </span>
            }
          />
          {snapshot.characters.length ? (
            <div className="character-grid">
              {snapshot.characters.map((character, index) => {
                const reference = snapshot.assets.find((a) => a.id === character.referenceAssetId);
                const sheet = snapshot.assets.find((a) => a.id === character.sheetAssetId);
                const job = snapshot.jobs.find(
                  (j) =>
                    j.targetId === character.id &&
                    j.kind === "image" &&
                    (j.status === "running" || j.status === "queued"),
                );
                return (
                  <article
                    key={character.id}
                    className={cx("character-card", current?.id === character.id && "is-active")}
                    onClick={() => select(character.id)}
                  >
                    <div className="character-card__visual">
                      {reference ? (
                        <img src={assetUrl(reference.thumbPath ?? reference.absPath)} alt={character.name} />
                      ) : (
                        <div className="character-card__placeholder">
                          <UserRound size={40} />
                          <span>尚未定妆</span>
                        </div>
                      )}
                      <div className="character-card__top">
                        <span className="character-card__number">0{index + 1}</span>
                        <StatusPill status={character.status} />
                      </div>
                      {sheet ? <span className="character-card__badge"><Layers size={12} /> 三视图</span> : null}
                      {job ? (
                        <div className="character-card__job">
                          <Sparkles size={13} className="spin" /> 生成中 {job.progress}%
                        </div>
                      ) : null}
                    </div>
                    <div className="character-card__body">
                      <div className="character-card__name">
                        <div>
                          <strong>{character.name}</strong>
                          <small>{character.role || "角色"}</small>
                        </div>
                        <IconButton
                          label="生成定妆照"
                          onClick={(event) => {
                            event.stopPropagation();
                            void generate(character.id, "portrait");
                          }}
                        >
                          <Wand2 size={15} />
                        </IconButton>
                      </div>
                      <p>{character.appearance || character.description || "还没有外观描述。"}</p>
                      <div className="trait-chips">
                        {character.lockedTraits.slice(0, 4).map((trait) => (
                          <span key={trait}>{trait}</span>
                        ))}
                        {character.lockedTraits.length > 4 ? <em>+{character.lockedTraits.length - 4}</em> : null}
                      </div>
                    </div>
                  </article>
                );
              })}
            </div>
          ) : (
            <EmptyState
              icon={<UserRound size={26} />}
              title="先建一个主角"
              description="从剧本自动提取角色，或手动写下外观、服装、发型与标志物。"
              action={
                <div className="empty-actions">
                  <Button variant="primary" icon={<Plus size={15} />} onClick={() => setAddOpen(true)}>
                    新建角色
                  </Button>
                  <Button variant="ghost" onClick={() => useAppStore.getState().setShowScriptDialog(true)}>
                    从剧本生成
                  </Button>
                </div>
              }
            />
          )}
        </main>

        <aside className="panel character-inspector">
          {current ? (
            <CharacterEditor character={current} onImport={() => setImportOpen(true)} />
          ) : (
            <EmptyState
              compact
              icon={<UserRound size={20} />}
              title="选择一个角色"
              description="编辑角色外观并生成定妆照，后续镜头会自动引用。"
            />
          )}
        </aside>
      </div>

      <AddCharacterDialog open={addOpen} onClose={() => setAddOpen(false)} />
      <ImportCharacterDialog open={importOpen} onClose={() => setImportOpen(false)} />
    </div>
  );
}

function CharacterEditor({ character, onImport }: { character: Character; onImport: () => void }) {
  const snapshot = useAppStore((s) => s.snapshot);
  const save = useAppStore((s) => s.saveCharacter);
  const remove = useAppStore((s) => s.removeCharacter);
  const generate = useAppStore((s) => s.generateCharacter);
  const pushToast = useAppStore((s) => s.pushToast);
  const [name, setName] = useState(character.name);
  const [role, setRole] = useState(character.role);
  const [appearance, setAppearance] = useState(character.appearance);
  const [description, setDescription] = useState(character.description);
  const [personality, setPersonality] = useState(character.personality);
  const [voice, setVoice] = useState(character.voice);
  const [traits, setTraits] = useState(character.lockedTraits.join("\n"));
  const [seed, setSeed] = useState(character.seed ? String(character.seed) : "");

  useEffect(() => {
    setName(character.name);
    setRole(character.role);
    setAppearance(character.appearance);
    setDescription(character.description);
    setPersonality(character.personality);
    setVoice(character.voice);
    setTraits(character.lockedTraits.join("\n"));
    setSeed(character.seed ? String(character.seed) : "");
  }, [character]);

  if (!snapshot) return null;
  const reference = snapshot.assets.find((a) => a.id === character.referenceAssetId);
  const sheet = snapshot.assets.find((a) => a.id === character.sheetAssetId);
  const job = snapshot.jobs.find(
    (j) => j.targetId === character.id && (j.status === "running" || j.status === "queued"),
  );
  const castShots = snapshot.episodes
    .flatMap((e) => e.scenes.flatMap((s) => s.shots))
    .filter((s) => s.characterIds.includes(character.id)).length;

  const submit = () =>
    void save(character.id, {
      name,
      role,
      appearance,
      description,
      personality,
      voice,
      lockedTraits: traits
        .split(/\n|,|，/)
        .map((t) => t.trim())
        .filter(Boolean),
      seed: Number(seed) || 0,
    });

  return (
    <>
      <div className="character-inspector__head">
        <div>
          <p className="eyebrow">CHARACTER PROFILE</p>
          <h2>{character.name}</h2>
          <p>{castShots} 个镜头正在使用这个角色</p>
        </div>
        <StatusPill status={character.status} />
      </div>

      <div className="character-visuals">
        <figure>
          <Thumb src={reference ? assetUrl(reference.thumbPath ?? reference.absPath) : null} alt="定妆照" aspect="3 / 4" />
          <figcaption>定妆照</figcaption>
        </figure>
        <figure>
          <Thumb src={sheet ? assetUrl(sheet.thumbPath ?? sheet.absPath) : null} alt="三视图" aspect="3 / 4" />
          <figcaption>三视图</figcaption>
        </figure>
        {job ? (
          <div className="character-visuals__job">
            <Sparkles size={14} className="spin" />
            <strong>{job.progress}%</strong>
            <span>{job.stage}</span>
          </div>
        ) : null}
      </div>

      <div className="inspector-actions">
        <Button variant="primary" icon={<Wand2 size={15} />} onClick={() => void generate(character.id, "portrait")}>
          生成定妆照
        </Button>
        <Button variant="secondary" icon={<Layers size={15} />} onClick={() => void generate(character.id, "sheet")}>
          生成三视图
        </Button>
      </div>
      <div className="inspector-actions inspector-actions--review">
        <Button size="sm" variant="ghost" icon={<Download size={13} />} onClick={onImport}>
          导入其他角色
        </Button>
        <Button
          size="sm"
          variant="ghost"
          icon={<Trash2 size={13} />}
          onClick={() => {
            if (window.confirm(`删除角色「${character.name}」？`)) void remove(character.id);
          }}
        >
          删除
        </Button>
      </div>

      {!character.lockedTraits.length ? (
        <InlineNotice tone="warn">外观还没锁定。建议至少写 3 条不可变的特征，跨镜头一致性会明显更好。</InlineNotice>
      ) : null}

      <div className="inspector-section">
        <div className="form-grid form-grid--two">
          <Field label="角色名">
            <input className="input" value={name} onChange={(e) => setName(e.target.value)} />
          </Field>
          <Field label="定位">
            <input className="input" value={role} onChange={(e) => setRole(e.target.value)} />
          </Field>
        </div>
      </div>
      <div className="inspector-section">
        <Field label="外观（发型、五官、服装、标志物）">
          <textarea className="textarea" rows={4} value={appearance} onChange={(e) => setAppearance(e.target.value)} />
        </Field>
      </div>
      <div className="inspector-section">
        <Field label="人物小传">
          <textarea className="textarea" rows={3} value={description} onChange={(e) => setDescription(e.target.value)} />
        </Field>
      </div>
      <div className="inspector-section">
        <div className="form-grid form-grid--two">
          <Field label="性格">
            <input className="input" value={personality} onChange={(e) => setPersonality(e.target.value)} />
          </Field>
          <Field label="声音">
            <input className="input" value={voice} onChange={(e) => setVoice(e.target.value)} />
          </Field>
        </div>
      </div>
      <div className="inspector-section">
        <Field label="锁定特征" hint="每行一条。生成时作为硬约束注入。">
          <textarea
            className="textarea textarea--mono"
            rows={5}
            value={traits}
            onChange={(e) => setTraits(e.target.value)}
            placeholder={"黑色及肩短发\n左侧银灰挑染\n深灰高领风衣"}
          />
        </Field>
        <div className="trait-chips trait-chips--editor">
          {traits
            .split(/\n|,|，/)
            .map((t) => t.trim())
            .filter(Boolean)
            .map((t) => (
              <span key={t}><Lock size={10} /> {t}</span>
            ))}
        </div>
      </div>
      <div className="inspector-section">
        <Field label="Seed" hint="相同 seed + 相同参考图有利于稳定抽卡">
          <input className="input" value={seed} onChange={(e) => setSeed(e.target.value)} placeholder="可选" />
        </Field>
      </div>

      <div className="inspector-foot inspector-foot--sticky">
        <button className="save-hint">
          <Check size={13} /> 保存后立即进入所有镜头的上下文
        </button>
        <Button variant="primary" onClick={submit}>
          保存角色
        </Button>
      </div>
    </>
  );
}

function AddCharacterDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const add = useAppStore((s) => s.addCharacter);
  const pushToast = useAppStore((s) => s.pushToast);
  const [name, setName] = useState("");
  const [role, setRole] = useState("");
  const [appearance, setAppearance] = useState("");
  const [description, setDescription] = useState("");
  const [traits, setTraits] = useState("");

  useEffect(() => {
    if (open) {
      setName("");
      setRole("");
      setAppearance("");
      setDescription("");
      setTraits("");
    }
  }, [open]);

  const submit = async () => {
    if (!name.trim()) {
      pushToast("error", "角色需要一个名字");
      return;
    }
    const input: CreateCharacterInput = {
      name: name.trim(),
      role,
      description,
      appearance,
      personality: "",
      voice: "",
      lockedTraits: traits.split(/\n|,|，/).map((t) => t.trim()).filter(Boolean),
    };
    await add(input);
    onClose();
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="新建角色"
      subtitle="写下不能被生成器改掉的特征，才能在几十个镜头里保持同一个人。"
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>取消</Button>
          <Button variant="primary" icon={<Plus size={15} />} onClick={submit}>创建角色</Button>
        </>
      }
    >
      <div className="form-grid form-grid--two">
        <Field label="角色名">
          <input className="input" value={name} onChange={(e) => setName(e.target.value)} placeholder="例如：沈知遥" />
        </Field>
        <Field label="定位">
          <input className="input" value={role} onChange={(e) => setRole(e.target.value)} placeholder="主角 / 对手 / 伙伴" />
        </Field>
      </div>
      <Field label="外观">
        <textarea className="textarea" rows={3} value={appearance} onChange={(e) => setAppearance(e.target.value)} placeholder="冷白皮，黑色及肩短发，左侧银灰挑染，深灰高领风衣…" />
      </Field>
      <Field label="人物小传">
        <textarea className="textarea" rows={3} value={description} onChange={(e) => setDescription(e.target.value)} />
      </Field>
      <Field label="锁定特征" hint="每行一条">
        <textarea className="textarea textarea--mono" rows={4} value={traits} onChange={(e) => setTraits(e.target.value)} />
      </Field>
    </Modal>
  );
}

function ImportCharacterDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const loadReferences = useAppStore((s) => s.loadReferences);
  const importCharacter = useAppStore((s) => s.importCharacter);
  const [references, setReferences] = useState<ProjectReference[]>([]);
  const [loading, setLoading] = useState(false);
  const [selected, setSelected] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setLoading(true);
    void loadReferences()
      .then((refs) => setReferences(refs.filter((ref) => ref.characters.length)))
      .finally(() => setLoading(false));
  }, [open, loadReferences]);

  const submit = async () => {
    if (!selected) return;
    const [projectId, characterId] = selected.split("::");
    await importCharacter(projectId, characterId);
    onClose();
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="从其他项目导入角色"
      subtitle="导入时会复制角色设定与参考图，不会在两个项目间建立实时依赖。"
      wide
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>取消</Button>
          <Button variant="primary" icon={<Download size={15} />} disabled={!selected} onClick={submit}>
            导入并复制素材
          </Button>
        </>
      }
    >
      {loading ? (
        <div className="muted">正在扫描项目…</div>
      ) : references.length ? (
        <div className="import-list">
          {references.map((ref) => (
            <div key={ref.projectId} className="import-project">
              <div className="import-project__head">
                <strong>{ref.projectName}</strong>
                <small>{ref.styleBible.visualStyle}</small>
              </div>
              <div className="import-characters">
                {ref.characters.map((character) => {
                  const value = `${ref.projectId}::${character.id}`;
                  return (
                    <button
                      key={character.id}
                      className={cx("import-character", selected === value && "is-active")}
                      onClick={() => setSelected(value)}
                    >
                      <UserRound size={16} />
                      <span>
                        <strong>{character.name}</strong>
                        <small>{character.role || "角色"}</small>
                      </span>
                      {selected === value ? <Check size={14} /> : null}
                    </button>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <EmptyState
          compact
          icon={<Download size={20} />}
          title="没有可导入的角色"
          description="创建另一个项目并生成角色后，就能在这里复制使用。"
        />
      )}
    </Modal>
  );
}
