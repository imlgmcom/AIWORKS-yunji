// 网页采集弹窗：粘贴单页 URL → 选择采集插件 → 抓取并按选择器提取 → 预览勾选 → 回填编辑表单
import { useEffect, useMemo, useRef, useState } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  Button,
  Input,
  Textarea,
  Spinner,
  Select,
  Checkbox,
  MessageBar,
  MessageBarBody,
  makeStyles,
  tokens,
} from '@fluentui/react-components';
import {
  ArrowDownloadRegular,
  GlobeRegular,
  FolderAddRegular,
  ArrowLeftRegular,
  ImageRegular,
  LinkRegular,
} from '@fluentui/react-icons';
import { open } from '@tauri-apps/plugin-dialog';
import { collectorApi } from '../lib/tauri';
import type { CollectorInfo } from '../lib/tauri';
import {
  extractPage,
  magnetName,
  magnetTrackerCount,
  parsePlugin,
  urlMatches,
} from '../lib/collector';
import type { CollectResult } from '../lib/collector';

export interface CollectApplyPayload {
  title: string;
  /** 游戏简介 → 写入文章正文（Markdown） */
  content: string;
  thumbnailUrl: string | null;
  imageUrls: string[];
  magnets: string[];
  sourceUrl: string | null;
  /** 抓图防盗链 Referer（页面最终 URL） */
  referer: string;
}

const useStyles = makeStyles({
  overlay: {
    position: 'fixed',
    inset: 0,
    background: 'rgba(0,0,0,0.35)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
    padding: '24px',
  },
  dialog: {
    background: tokens.colorNeutralBackground1,
    borderRadius: '10px',
    boxShadow: tokens.shadow64,
    width: '760px',
    maxWidth: '100%',
    maxHeight: '88vh',
    display: 'flex',
    flexDirection: 'column',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '14px 18px',
    borderBottom: `1px solid ${tokens.colorNeutralStroke2}`,
    fontWeight: 600,
    fontSize: '15px',
  },
  body: {
    padding: '16px 18px',
    overflowY: 'auto',
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
  },
  footer: {
    display: 'flex',
    justifyContent: 'flex-end',
    gap: '8px',
    padding: '12px 18px',
    borderTop: `1px solid ${tokens.colorNeutralStroke2}`,
  },
  row: { display: 'flex', gap: '8px', alignItems: 'center' },
  sectionLabel: {
    fontWeight: 600,
    fontSize: '13px',
    color: tokens.colorNeutralForeground2,
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    marginTop: '4px',
  },
  imgGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))',
    gap: '8px',
  },
  imgCell: {
    position: 'relative',
    borderRadius: '6px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    overflow: 'hidden',
    background: tokens.colorNeutralBackground2,
    aspectRatio: '16 / 9',
    cursor: 'pointer',
  },
  imgCellImg: { width: '100%', height: '100%', objectFit: 'cover' },
  imgCellCheck: { position: 'absolute', top: '4px', left: '4px', zIndex: 1 },
  imgCellFallback: {
    width: '100%',
    height: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: tokens.colorNeutralForeground3,
    fontSize: '11px',
    padding: '4px',
    textAlign: 'center',
  },
  magnetRow: {
    display: 'flex',
    gap: '8px',
    alignItems: 'flex-start',
    padding: '8px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    borderRadius: '6px',
  },
  coverBox: {
    display: 'flex',
    gap: '10px',
    alignItems: 'center',
  },
  coverImg: {
    width: '64px',
    height: '90px',
    objectFit: 'cover',
    borderRadius: '6px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    background: tokens.colorNeutralBackground2,
  },
  proxyBanner: {
    fontSize: '12px',
    padding: '6px 10px',
    borderRadius: '6px',
  },
  taskRow: { display: 'flex', flexDirection: 'column', gap: '3px', padding: '6px 2px' },
  taskLine: { display: 'flex', justifyContent: 'space-between', fontSize: '12px', gap: '8px' },
  barTrack: {
    height: '5px',
    borderRadius: '3px',
    background: tokens.colorNeutralBackground3,
    overflow: 'hidden',
  },
  barFill: { height: '100%', background: tokens.colorBrandBackground, transition: 'width .2s' },
});

type Step = 'input' | 'loading' | 'preview' | 'applying';

interface CollectorProgressEvent {
  kind: 'cover' | 'image';
  index: number;
  total_tasks: number;
  done: number;
  size?: number | null;
  round: number;
  proxy?: string | null;
}

interface TaskProgress {
  done: number;
  size?: number | null;
  round: number;
  speed: number; // bytes/s
}

export function CollectDialog({
  hasThumbnail,
  onClose,
  onApply,
}: {
  hasThumbnail: boolean;
  onClose: () => void;
  onApply: (payload: CollectApplyPayload) => Promise<void>;
}) {
  const s = useStyles();
  const [step, setStep] = useState<Step>('input');
  const [url, setUrl] = useState('');
  const [plugins, setPlugins] = useState<CollectorInfo[]>([]);
  const [selectedId, setSelectedId] = useState('');
  const [error, setError] = useState('');
  const [referer, setReferer] = useState('');

  const [extracted, setExtracted] = useState<CollectResult | null>(null);
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [useThumb, setUseThumb] = useState(!hasThumbnail);
  const [useSource, setUseSource] = useState(true);
  const [imgSet, setImgSet] = useState<Set<number>>(new Set());
  const [magSet, setMagSet] = useState<Set<number>>(new Set());

  // ---- 下载实时进度（collector-progress 事件）----
  const [proxy, setProxy] = useState<string | null | undefined>(undefined);
  const [progress, setProgress] = useState<Record<string, TaskProgress>>({});
  const speedRef = useRef<Map<string, { t: number; b: number }>>(new Map());

  useEffect(() => {
    collectorApi
      .list()
      .then((list) => {
        setPlugins(list);
        if (url && list.length > 0) {
          const m = list.find((p) => urlMatches(p.patterns, url));
          if (m) setSelectedId(m.id);
        }
        if (!selectedId) setSelectedId(list[0]?.id ?? '');
      })
      .catch((e: Error) => setError(e.message));
    // 仅初始化加载
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // URL 变化时自动匹配插件
  useEffect(() => {
    if (!url || plugins.length === 0) return;
    const m = plugins.find((p) => urlMatches(p.patterns, url));
    if (m) setSelectedId(m.id);
  }, [url, plugins]);

  // 回填阶段订阅后端下载进度
  useEffect(() => {
    if (step !== 'applying') return;
    let unlisten: UnlistenFn | undefined;
    let alive = true;
    listen<CollectorProgressEvent>('collector-progress', (ev) => {
      if (!alive) return;
      const p = ev.payload;
      if (p.proxy !== undefined && p.proxy !== null) setProxy(p.proxy);
      else if (p.proxy === null) setProxy(null);
      const key = `${p.kind}-${p.index}`;
      const now = performance.now();
      const prev = speedRef.current.get(key);
      let speed = 0;
      if (prev) {
        const dt = (now - prev.t) / 1000;
        if (dt > 0) speed = Math.max(0, (p.done - prev.b) / dt);
      }
      speedRef.current.set(key, { t: now, b: p.done });
      setProgress((m) => {
        const inst = speed;
        const smoothed = m[key] ? m[key].speed * 0.6 + inst * 0.4 : inst;
        return {
          ...m,
          [key]: {
            done: p.done,
            size: p.size ?? m[key]?.size,
            round: p.round,
            speed: smoothed,
          },
        };
      });
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      alive = false;
      unlisten?.();
    };
  }, [step]);

  const selectedPlugin = useMemo(
    () => plugins.find((p) => p.id === selectedId),
    [plugins, selectedId],
  );

  const start = async () => {
    setError('');
    const target = url.trim();
    if (!/^https?:\/\//i.test(target)) {
      setError('请输入 http/https 开头的网页链接');
      return;
    }
    if (!selectedPlugin) {
      setError('请选择采集插件');
      return;
    }
    setStep('loading');
    try {
      const page = await collectorApi.fetch(target);
      const plugin = parsePlugin(selectedPlugin);
      const result = extractPage(page.body, page.final_url, plugin);
      setReferer(page.final_url);
      setExtracted(result);
      setTitle(result.title);
      setDescription(result.description);
      setImgSet(new Set(result.images.map((_, i) => i)));
      setMagSet(new Set(result.magnets.map((_, i) => i)));
      setUseThumb(!hasThumbnail);
      setUseSource(true);
      if (!result.title && !result.thumbnail && result.images.length === 0 && result.magnets.length === 0) {
        setError('插件未从该页面提取到任何内容，可能页面结构已变化，请更换插件或检查链接');
        setStep('input');
        return;
      }
      setStep('preview');
    } catch (e) {
      setError(e instanceof Error ? e.message : '采集失败');
      setStep('input');
    }
  };

  const importPlugin = async () => {
    setError('');
    const picked = await open({
      multiple: false,
      filters: [{ name: '采集插件', extensions: ['json'] }],
    });
    if (typeof picked !== 'string') return;
    if (!window.confirm('导入第三方采集插件？插件只包含选择器规则、不执行代码，但仍建议只导入你信任的来源。')) return;
    try {
      const info = await collectorApi.importPlugin(picked);
      setPlugins((prev) => [info, ...prev.filter((p) => p.id !== info.id)]);
      setSelectedId(info.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : '导入失败');
    }
  };

  const toggle = (set: Set<number>, setter: (s: Set<number>) => void, i: number, checked: boolean) => {
    const next = new Set(set);
    if (checked) next.add(i);
    else next.delete(i);
    setter(next);
  };

  const confirm = async () => {
    if (!extracted) return;
    setProgress({});
    speedRef.current.clear();
    setProxy(undefined);
    setStep('applying');
    try {
      await onApply({
        title: title.trim(),
        content: description.trim(),
        thumbnailUrl: useThumb && extracted.thumbnail ? extracted.thumbnail : null,
        imageUrls: extracted.images.filter((_, i) => imgSet.has(i)),
        magnets: extracted.magnets.filter((_, i) => magSet.has(i)),
        sourceUrl: useSource ? (referer || url.trim()) : null,
        referer,
      });
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : '回填失败');
      setStep('preview');
    }
  };

  return (
    <div className={s.overlay} onClick={(e) => e.target === e.currentTarget && step !== 'applying' && onClose()}>
      <div className={s.dialog}>
        <div className={s.header}>
          <span style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <GlobeRegular /> 网页采集
          </span>
          {selectedPlugin && (
            <span style={{ fontWeight: 400, fontSize: '12px', color: tokens.colorNeutralForeground3 }}>
              {selectedPlugin.name}
            </span>
          )}
        </div>

        <div className={s.body}>
          {error && (
            <MessageBar intent="error">
              <MessageBarBody>{error}</MessageBarBody>
            </MessageBar>
          )}

          {(step === 'input' || step === 'loading') && (
            <>
              <Input
                placeholder="粘贴单个网页链接，如 https://fitgirl-repacks.site/forza-horizon-6/"
                value={url}
                onChange={(_, d) => setUrl(d.value)}
                contentBefore={<GlobeRegular />}
                disabled={step === 'loading'}
              />
              <div className={s.row}>
                <Select
                  style={{ flex: 1 }}
                  value={selectedId}
                  onChange={(_, d) => setSelectedId(d.value)}
                  disabled={step === 'loading' || plugins.length === 0}
                >
                  {plugins.length === 0 && <option value="">（无可用插件）</option>}
                  {plugins.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                      {urlMatches(p.patterns, url) ? '（匹配当前链接）' : ''}
                    </option>
                  ))}
                </Select>
                <Button icon={<FolderAddRegular />} onClick={importPlugin} disabled={step === 'loading'}>
                  导入插件
                </Button>
              </div>
              {selectedPlugin?.description && (
                <span style={{ fontSize: '12px', color: tokens.colorNeutralForeground3 }}>
                  {selectedPlugin.description}
                </span>
              )}
              {step === 'loading' && <Spinner labelPosition="below" label="正在抓取并解析页面…" />}
            </>
          )}

          {step === 'preview' && extracted && (
            <>
              <div>
                <div className={s.sectionLabel}>标题</div>
                <Input value={title} onChange={(_, d) => setTitle(d.value)} style={{ width: '100%' }} />
              </div>

              <div>
                <div className={s.sectionLabel}>游戏简介</div>
                <Textarea
                  value={description}
                  onChange={(_, d) => setDescription(d.value)}
                  rows={8}
                  resize="vertical"
                  style={{ width: '100%' }}
                />
              </div>

              {extracted.thumbnail && (
                <div>
                  <div className={s.sectionLabel}>
                    <Checkbox checked={useThumb} onChange={(_, d) => setUseThumb(d.checked === true)} label="封面图" />
                  </div>
                  <div className={s.coverBox} style={{ marginTop: '6px' }}>
                    <RemoteImg className={s.coverImg} src={extracted.thumbnail} />
                    <span style={{ fontSize: '12px', color: tokens.colorNeutralForeground3, wordBreak: 'break-all' }}>
                      {extracted.thumbnail}
                    </span>
                  </div>
                </div>
              )}

              {extracted.images.length > 0 && (
                <div>
                  <div className={s.sectionLabel}>
                    <ImageRegular /> 图集（{imgSet.size}/{extracted.images.length}，仅图片）
                    <Button
                      size="small"
                      appearance="subtle"
                      onClick={() =>
                        setImgSet(imgSet.size === extracted.images.length ? new Set() : new Set(extracted.images.map((_, i) => i)))
                      }
                    >
                      {imgSet.size === extracted.images.length ? '全不选' : '全选'}
                    </Button>
                  </div>
                  <div className={s.imgGrid}>
                    {extracted.images.map((u, i) => (
                      <div key={u} className={s.imgCell} onClick={() => toggle(imgSet, setImgSet, i, !imgSet.has(i))}>
                        <Checkbox
                          className={s.imgCellCheck}
                          checked={imgSet.has(i)}
                          onChange={(_, d) => toggle(imgSet, setImgSet, i, d.checked === true)}
                          onClick={(e) => e.stopPropagation()}
                        />
                        <RemoteImg className={s.imgCellImg} src={u} />
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {extracted.magnets.length > 0 && (
                <div>
                  <div className={s.sectionLabel}>
                    🧲 磁力链接（{magSet.size}/{extracted.magnets.length}，已按 hash 去重）
                    <Button
                      size="small"
                      appearance="subtle"
                      onClick={() =>
                        setMagSet(magSet.size === extracted.magnets.length ? new Set() : new Set(extracted.magnets.map((_, i) => i)))
                      }
                    >
                      {magSet.size === extracted.magnets.length ? '全不选' : '全选'}
                    </Button>
                  </div>
                  {extracted.magnets.map((m, i) => {
                    const dn = magnetName(m);
                    return (
                      <div key={m} className={s.magnetRow} style={{ marginTop: '6px' }}>
                        <Checkbox
                          checked={magSet.has(i)}
                          onChange={(_, d) => toggle(magSet, setMagSet, i, d.checked === true)}
                        />
                        <div style={{ fontSize: '12px', minWidth: 0 }}>
                          <div style={{ fontWeight: 600 }}>{dn || `磁力 #${i + 1}`}</div>
                          <div style={{ color: tokens.colorNeutralForeground3, wordBreak: 'break-all' }}>
                            {m.slice(0, 100)}
                            {m.length > 100 ? '…' : ''}
                          </div>
                          <div>{magnetTrackerCount(m)} 个 tracker</div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}

              <div className={s.sectionLabel}>
                <LinkRegular />
                <Checkbox
                  checked={useSource}
                  onChange={(_, d) => setUseSource(d.checked === true)}
                  label="把采集网址作为「链接」类型附件"
                />
              </div>
            </>
          )}

          {step === 'applying' && extracted && (
            <ProgressPanel
              proxy={proxy}
              rows={[
                ...(useThumb && extracted.thumbnail
                  ? [{ key: 'cover-0', label: '封面图' }]
                  : []),
                ...extracted.images
                  .map((_, i) => i)
                  .filter((i) => imgSet.has(i))
                  .map((_, order) => ({ key: `image-${order}`, label: `图集 ${order + 1}` })),
              ]}
              progress={progress}
            />
          )}
        </div>

        <div className={s.footer}>
          <Button appearance="subtle" onClick={onClose} disabled={step === 'applying' || step === 'loading'}>
            取消
          </Button>
          {step === 'preview' && (
            <>
              <Button icon={<ArrowLeftRegular />} onClick={() => setStep('input')}>
                上一步
              </Button>
              <Button
                appearance="primary"
                icon={<ArrowDownloadRegular />}
                onClick={confirm}
                disabled={
                  !title.trim() &&
                  !useThumb &&
                  imgSet.size === 0 &&
                  magSet.size === 0 &&
                  !useSource
                }
              >
                确认回填
              </Button>
            </>
          )}
          {step === 'input' && (
            <Button appearance="primary" icon={<ArrowDownloadRegular />} onClick={start}>
              开始采集
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

/** 远程图片预览：加载失败时显示兜底（部分站防盗链，实际下载由后端带 Referer 完成） */
function RemoteImg({ src, className }: { src: string; className: string }) {
  const [failed, setFailed] = useState(false);
  if (failed) {
    return (
      <div
        className={className}
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '11px',
          color: 'var(--colorNeutralForeground3)',
          padding: '4px',
          textAlign: 'center',
        }}
      >
        预览不可用
      </div>
    );
  }
  return (
    <img
      className={className}
      src={src}
      alt=""
      loading="lazy"
      referrerPolicy="no-referrer"
      onError={() => setFailed(true)}
    />
  );
}

function fmtBytes(n: number): string {
  if (n >= 1024 * 1024) return `${(n / 1024 / 1024).toFixed(2)}MB`;
  if (n >= 1024) return `${(n / 1024).toFixed(1)}KB`;
  return `${n}B`;
}

function ProgressPanel({
  proxy,
  rows,
  progress,
}: {
  proxy: string | null | undefined;
  rows: { key: string; label: string }[];
  progress: Record<string, TaskProgress>;
}) {
  const s = useStyles();
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', padding: '4px 0' }}>
      <div
        className={s.proxyBanner}
        style={{
          background: proxy === undefined ? 'var(--colorNeutralBackground3)' : proxy ? 'var(--colorPaletteGreenBackground2)' : 'var(--colorPaletteRedBackground2)',
          color: proxy === undefined ? 'var(--colorNeutralForeground3)' : undefined,
        }}
      >
        {proxy === undefined && '正在建立连接…'}
        {proxy === null && (
          <>
            ⚠ 未检测到系统代理，当前为<b>直连</b>。浏览器能秒开而这里很慢，通常是 Clash 只开了浏览器代理/已关闭
            ——请在 Clash 中打开「系统代理」后重试（TUN 模式可忽略此提示）
          </>
        )}
        {proxy && <>网络出口：系统代理 <b>{proxy}</b></>}
      </div>

      {rows.map(({ key, label }) => {
        const t = progress[key];
        const pct = t?.size ? Math.min(100, Math.round((t.done / t.size) * 100)) : undefined;
        return (
          <div key={key} className={s.taskRow}>
            <div className={s.taskLine}>
              <span>{label}</span>
              <span style={{ color: 'var(--colorNeutralForeground3)' }}>
                {!t
                  ? '排队中…'
                  : `${fmtBytes(t.done)}${t.size ? ` / ${fmtBytes(t.size)}` : ''}${
                      pct !== undefined ? ` · ${pct}%` : ''
                    } · ${fmtBytes(t.speed)}/s${t.round > 0 ? ` · 续传第 ${t.round + 1} 次` : ''}`}
              </span>
            </div>
            <div className={s.barTrack}>
              <div
                className={s.barFill}
                style={{ width: pct !== undefined ? `${pct}%` : t ? '100%' : '0%', opacity: pct === undefined ? 0.35 : 1 }}
              />
            </div>
          </div>
        );
      })}
      <span style={{ fontSize: '11px', color: 'var(--colorNeutralForeground3)' }}>
        单张图片 30 秒无数据才会超时并重试，慢而不断会持续下载，请勿关闭窗口
      </span>
    </div>
  );
}
