// 合集详情页：缩略图 + 标题/元信息 + Markdown 正文 + 合集内文章列表
import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  Button,
  Spinner,
  Badge,
  MessageBar,
  MessageBarBody,
  Input,
  Label,
  Textarea,
  Dropdown,
  Option,
  Dialog,
  DialogSurface,
  DialogBody,
  DialogTitle,
  DialogContent,
  DialogActions,
  makeStyles,
  tokens,
} from '@fluentui/react-components';
import {
  ArrowLeftRegular,
  EditRegular,
  ArrowUploadRegular,
  DeleteRegular,
  DocumentRegular,
} from '@fluentui/react-icons';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { collectionApi, imageApi, resourceApi, assetUrl } from '../lib/tauri';
import type { Collection, Resource } from '../types/models';
import { usePerms } from '../hooks/usePerms';
import { MarkdownView } from '../components/MarkdownView';
import { MarkdownEditor } from '../components/MarkdownEditor';
import { formatTime, relativeTime } from '../lib/format';

const useStyles = makeStyles({
  container: { display: 'flex', flexDirection: 'column', gap: '24px' },
  header: {
    display: 'flex',
    gap: '20px',
    alignItems: 'flex-start',
    '@media (max-width: 640px)': {
      flexDirection: 'column',
    },
  },
  thumb: {
    width: '212px',
    height: '150px',
    objectFit: 'cover',
    borderRadius: '10px',
    backgroundColor: tokens.colorNeutralBackground3,
    flexShrink: 0,
  },
  headerMain: { flex: 1, minWidth: 0 },
  title: { fontSize: '26px', fontWeight: 700, margin: 0, lineHeight: 1.3 },
  meta: { display: 'flex', gap: '10px', alignItems: 'center', flexWrap: 'wrap', fontSize: '13px', color: tokens.colorNeutralForeground3 },
  descBlock: {
    margin: 0,
    fontSize: '14px',
    lineHeight: 1.7,
    color: tokens.colorNeutralForeground2,
    fontStyle: 'italic',
    position: 'relative',
    paddingLeft: '18px',
    '&::before': {
      content: '"“"',
      position: 'absolute',
      left: 0,
      top: '-2px',
      fontSize: '22px',
      color: tokens.colorBrandForeground1,
      fontStyle: 'normal',
      lineHeight: 1,
    },
  },
  content: {
    padding: '20px 24px',
    borderRadius: '12px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    backgroundColor: tokens.colorNeutralBackground1,
    lineHeight: 1.8,
    fontSize: '15px',
    '& img': { maxWidth: '100%', borderRadius: '8px' },
    '& pre': { overflowX: 'auto', background: tokens.colorNeutralBackground2, padding: '12px', borderRadius: '8px' },
  },
  resList: { display: 'flex', flexDirection: 'column', gap: '10px' },
  resCard: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '14px 16px',
    borderRadius: '10px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    backgroundColor: tokens.colorNeutralBackground1,
    cursor: 'pointer',
    transition: 'background 0.15s',
    '&:hover': { backgroundColor: tokens.colorNeutralBackground1Hover },
  },
  resTitle: { fontSize: '15px', fontWeight: 600, flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  resDesc: { fontSize: '12px', color: tokens.colorNeutralForeground3, marginTop: '2px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
});

export function CollectionDetailPage() {
  const styles = useStyles();
  const { cid } = useParams<{ cid: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const canEdit = usePerms().can('collection_own');
  const cidNum = Number(cid);

  const [editOpen, setEditOpen] = useState(false);
  const [editTitle, setEditTitle] = useState('');
  const [editDesc, setEditDesc] = useState('');
  const [editVisibility, setEditVisibility] = useState('private');
  const [editContent, setEditContent] = useState('');
  const [editThumb, setEditThumb] = useState('');
  const [saving, setSaving] = useState(false);

  const { data: collection, isLoading } = useQuery<Collection>({
    queryKey: ['collection', cidNum],
    queryFn: () => collectionApi.get(cidNum),
    enabled: !!cid && !Number.isNaN(cidNum),
  });

  const { data: resData } = useQuery<{ items: Resource[] }>({
    queryKey: ['collection-resources', cidNum],
    queryFn: () => resourceApi.list({ page: 1, page_size: 1000, collection_id: cidNum }),
    enabled: !!cid && !Number.isNaN(cidNum),
  });

  const openEdit = () => {
    if (!collection) return;
    setEditTitle(collection.title);
    setEditDesc(collection.description);
    setEditVisibility(collection.visibility);
    setEditContent(collection.content || '');
    setEditThumb(collection.thumbnail_path);
    setEditOpen(true);
  };

  const uploadThumb = async () => {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp'] }],
    });
    const filePath = typeof selected === 'string' ? selected : null;
    if (!filePath || !collection) return;
    const result = await imageApi.uploadCollectionImage(collection.id, filePath);
    setEditThumb(result.file_path);
  };

  const saveEdit = async () => {
    if (!collection) return;
    setSaving(true);
    try {
      await collectionApi.update(collection.id, {
        title: editTitle.trim(),
        description: editDesc.trim(),
        visibility: editVisibility,
        access_password: collection.access_password,
        thumbnail_path: editThumb,
        content: editContent,
      });
      queryClient.invalidateQueries({ queryKey: ['collection', cidNum] });
      queryClient.invalidateQueries({ queryKey: ['collections'] });
      setEditOpen(false);
    } catch (e: any) {
      alert(e.message || '保存失败');
    } finally {
      setSaving(false);
    }
  };

  if (isLoading) return <Spinner label="加载中..." />;
  if (!collection) {
    return (
      <MessageBar intent="warning">
        <MessageBarBody>合集不存在或已被删除</MessageBarBody>
      </MessageBar>
    );
  }

  const resources = resData?.items ?? [];

  return (
    <div className={styles.container}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Button appearance="subtle" icon={<ArrowLeftRegular />} onClick={() => navigate('/collections')}>
          返回合集
        </Button>
        {canEdit && (
          <Button appearance="outline" icon={<EditRegular />} onClick={openEdit}>
            编辑合集
          </Button>
        )}
      </div>

      {/* 标题 + 缩略图（左右布局） */}
      <div className={styles.header}>
        {collection.thumbnail_path && (
          <img
            className={styles.thumb}
            src={assetUrl(collection.thumbnail_path)}
            alt={collection.title}
            loading="lazy"
          />
        )}
        <div className={styles.headerMain}>
          <h1 className={styles.title}>{collection.title}</h1>
          <div className={styles.meta}>
            <span title={formatTime(collection.created_at)}>创建于 {relativeTime(collection.created_at)}</span>
            <span title={formatTime(collection.updated_at)}>更新于 {relativeTime(collection.updated_at)}</span>
            <Badge appearance="outline">{collection.visibility === 'public' ? '公开' : '私有'}</Badge>
            <Badge appearance="filled" color="informative">{collection.resource_count ?? resources.length} 篇文章</Badge>
          </div>
          {collection.description && (
            <p className={styles.descBlock} style={{ marginTop: '12px' }}>
              {collection.description.replace(/\s*\n+\s*/g, ' ').trim()}
            </p>
          )}
        </div>
      </div>

      {collection.content && (
        <MarkdownView content={collection.content} className={styles.content} />
      )}

      {/* 合集内文章列表 */}
      <div>
        <h3 style={{ fontSize: '16px', fontWeight: 600, margin: '0 0 12px' }}>
          <DocumentRegular style={{ marginRight: '6px', verticalAlign: 'middle' }} />
          合集文章（{resources.length}）
        </h3>
        {resources.length === 0 ? (
          <div style={{ color: tokens.colorNeutralForeground3, fontSize: '13px', padding: '20px', textAlign: 'center' }}>
            暂无文章
          </div>
        ) : (
          <div className={styles.resList}>
            {resources.map((r) => (
              <div key={r.id} className={styles.resCard} onClick={() => navigate(`/detail/${r.id}`)}>
                {r.thumbnail_path ? (
                  <img
                    src={assetUrl(r.thumbnail_path)}
                    alt=""
                    style={{ width: '64px', height: '48px', objectFit: 'cover', borderRadius: '6px', flexShrink: 0 }}
                    loading="lazy"
                  />
                ) : (
                  <div style={{ width: '64px', height: '48px', borderRadius: '6px', backgroundColor: tokens.colorNeutralBackground3, flexShrink: 0 }} />
                )}
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div className={styles.resTitle}>{r.title}</div>
                  {r.description && <div className={styles.resDesc}>{r.description}</div>}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 编辑弹窗 */}
      <Dialog open={editOpen} onOpenChange={(_, d) => setEditOpen(d.open)}>
        <DialogSurface style={{ maxWidth: '720px' }}>
          <DialogBody>
            <DialogTitle>编辑合集</DialogTitle>
            <DialogContent>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                <div>
                  <Label htmlFor="col-title">名称 *</Label>
                  <Input id="col-title" value={editTitle} onChange={(_, d) => setEditTitle(d.value)} />
                </div>
                <div>
                  <Label htmlFor="col-desc">描述</Label>
                  <Textarea id="col-desc" value={editDesc} onChange={(_, d) => setEditDesc(d.value)} rows={2} />
                </div>
                <div>
                  <Label>可见性</Label>
                  <Dropdown
                    value={editVisibility === 'public' ? '公开' : '私有'}
                    selectedOptions={[editVisibility]}
                    onOptionSelect={(_, d) => d.optionValue && setEditVisibility(d.optionValue)}
                  >
                    <Option value="private" text="私有">私有</Option>
                    <Option value="public" text="公开">公开</Option>
                  </Dropdown>
                </div>
                <div>
                  <Label>缩略图</Label>
                  <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
                    {editThumb ? (
                      <img src={assetUrl(editThumb)} alt="" style={{ width: '120px', height: '80px', objectFit: 'cover', borderRadius: '6px' }} />
                    ) : (
                      <div style={{ width: '120px', height: '80px', borderRadius: '6px', backgroundColor: tokens.colorNeutralBackground3 }} />
                    )}
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                      <Button appearance="outline" size="small" icon={<ArrowUploadRegular />} onClick={uploadThumb}>上传</Button>
                      {editThumb && (
                        <Button appearance="subtle" size="small" icon={<DeleteRegular />} onClick={() => setEditThumb('')}>清除</Button>
                      )}
                    </div>
                  </div>
                </div>
                <div>
                  <Label>合集正文（Markdown）</Label>
                  <MarkdownEditor value={editContent} onChange={setEditContent} height={320} placeholder="在此输入合集正文..." />
                </div>
              </div>
            </DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={() => setEditOpen(false)}>取消</Button>
              <Button appearance="primary" onClick={saveEdit} disabled={saving || !editTitle.trim()}>
                {saving ? '保存中...' : '保存'}
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </div>
  );
}
