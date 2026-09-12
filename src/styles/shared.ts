// 共享样式：所有页面通用的 makeStyles 定义，避免各页面重复定义相同样式
import { makeStyles, tokens } from '@fluentui/react-components';

export const useSharedStyles = makeStyles({
  // 页面容器（所有路由页最外层）
  pageContainer: {
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  // 页面标题栏（标题 + 右侧操作按钮）
  pageHeader: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: '12px',
  },
  // 表单字段容器
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    marginBottom: '12px',
  },
  // 瀑布流容器（CSS 多列布局）
  masonry: {
    columnCount: 'auto',
    columnWidth: '240px',
    columnGap: '12px',
  },
  // 瀑布流卡片
  masonryCard: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px',
    padding: '8px',
    borderRadius: '8px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    backgroundColor: tokens.colorNeutralBackground1,
    cursor: 'pointer',
    position: 'relative',
    width: '100%',
    marginBottom: '12px',
    breakInside: 'avoid',
    '&:hover': {
      border: `1px solid ${tokens.colorBrandStroke1}`,
    },
  },
  // 卡片缩略图（按原始比例显示）
  thumb: {
    width: '100%',
    height: 'auto',
    display: 'block',
    borderRadius: '6px',
    backgroundColor: tokens.colorNeutralBackground3,
  },
  // 无缩略图时的占位
  thumbPlaceholder: {
    width: '100%',
    aspectRatio: '4 / 3',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: '6px',
    backgroundColor: tokens.colorNeutralBackground3,
    fontSize: '32px',
    fontWeight: 700,
    color: tokens.colorNeutralForeground3,
  },
  // 卡片正文区
  cardBody: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    padding: '0 4px',
    minWidth: 0,
  },
  // 卡片标题
  cardTitle: {
    fontSize: '15px',
    fontWeight: 600,
    whiteSpace: 'nowrap',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
  },
  // 卡片描述（两行截断）
  cardDesc: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
    display: '-webkit-box',
    WebkitLineClamp: 2,
    WebkitBoxOrient: 'vertical',
    overflow: 'hidden',
  },
  // 卡片底部元信息
  cardMeta: {
    display: 'flex',
    gap: '6px',
    alignItems: 'center',
    fontSize: '11px',
    color: tokens.colorNeutralForeground3,
    flexWrap: 'wrap',
  },
  // 卡片标签行（独立于元信息行，避免与附件类型图标混淆）
  cardTags: {
    display: 'flex',
    gap: '4px',
    alignItems: 'center',
    flexWrap: 'wrap',
    marginTop: '2px',
  },
  // 卡片操作按钮区
  cardActions: {
    display: 'flex',
    gap: '4px',
    padding: '0 4px 4px',
  },
  // 空状态
  emptyState: {
    padding: '48px 16px',
    textAlign: 'center',
  },
  // 管理页 grid 卡片容器（合集/标签）
  gridList: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(240px, 1fr))',
    gap: '12px',
  },
  // 管理页卡片（合集/标签）
  gridCard: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '12px 16px',
    borderRadius: '8px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    backgroundColor: tokens.colorNeutralBackground1,
    position: 'relative',
    cursor: 'pointer',
    '&:hover': {
      border: `1px solid ${tokens.colorBrandStroke1}`,
    },
  },
  // grid 卡片信息区
  gridCardInfo: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
  },
  // grid 卡片操作按钮区
  gridCardActions: {
    display: 'flex',
    gap: '4px',
  },

  // === 四种视图模式 ===

  // 16:9 竖向卡片网格（固定 3 列，窄屏 2 列）
  gridV: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
    gap: '12px',
  },
  gridVCard: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
    padding: '10px',
    borderRadius: '8px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    backgroundColor: tokens.colorNeutralBackground1,
    cursor: 'pointer',
    position: 'relative',
    overflow: 'hidden',
    '&:hover': {
      border: `1px solid ${tokens.colorBrandStroke1}`,
    },
  },
  gridVThumb: {
    width: '100%',
    aspectRatio: '16 / 9',
    objectFit: 'cover',
    borderRadius: '6px',
    backgroundColor: tokens.colorNeutralBackground3,
  },

  // 自适应横向卡片（1 行 2~4 个）
  gridH: {
    display: 'grid',
    // min(45%, 320px): 45% 保证至少 2 列, 320px 上限保证最多约 4 列
    gridTemplateColumns: 'repeat(auto-fill, minmax(min(45%, 320px), 1fr))',
    gap: '10px',
  },
  gridHCard: {
    display: 'flex',
    gap: '8px',
    padding: '8px',
    borderRadius: '8px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    backgroundColor: tokens.colorNeutralBackground1,
    cursor: 'pointer',
    position: 'relative',
    overflow: 'hidden',
    '&:hover': {
      border: `1px solid ${tokens.colorBrandStroke1}`,
    },
  },
  gridHThumb: {
    width: '80px',
    height: '60px',
    objectFit: 'cover',
    borderRadius: '6px',
    flexShrink: 0,
    backgroundColor: tokens.colorNeutralBackground3,
  },
  gridHThumbPlaceholder: {
    width: '80px',
    height: '60px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: '6px',
    flexShrink: 0,
    backgroundColor: tokens.colorNeutralBackground3,
    fontSize: '24px',
    fontWeight: 700,
    color: tokens.colorNeutralForeground3,
  },
  gridHBody: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
    minWidth: 0,
    flex: 1,
  },

  // 表格列表
  tableList: {
    display: 'flex',
    flexDirection: 'column',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    borderRadius: '8px',
    overflow: 'hidden',
  },
  tableRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '8px 12px',
    cursor: 'pointer',
    position: 'relative',
    '&:hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
    },
    '&:not(:last-child)': {
      borderBottom: `1px solid ${tokens.colorNeutralStroke2}`,
    },
  },
  tableThumb: {
    width: '56px',
    height: '56px',
    objectFit: 'cover',
    borderRadius: '4px',
    flexShrink: 0,
    backgroundColor: tokens.colorNeutralBackground3,
  },
  tableThumbPlaceholder: {
    width: '56px',
    height: '56px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: '4px',
    flexShrink: 0,
    backgroundColor: tokens.colorNeutralBackground3,
    fontSize: '20px',
    fontWeight: 700,
    color: tokens.colorNeutralForeground3,
  },
  tableBody: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
    flex: 1,
    minWidth: 0,
  },

  // 视图切换器（浮动底栏右侧）
  viewSwitcher: {
    position: 'fixed',
    right: '24px',
    bottom: '20px',
    display: 'flex',
    gap: '4px',
    padding: '4px',
    borderRadius: '10px',
    backgroundColor: tokens.colorNeutralBackground1,
    boxShadow: '0 2px 12px rgba(0,0,0,0.12)',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    zIndex: 200,
    alignItems: 'center',
  },
  viewSwitcherBtn: {
    minWidth: '36px',
    height: '36px',
    borderRadius: '6px',
  },
  viewSwitcherActive: {
    backgroundColor: tokens.colorBrandBackground2,
    color: tokens.colorBrandForeground1,
  },
  viewSwitcherDivider: {
    width: '1px',
    height: '24px',
    backgroundColor: tokens.colorNeutralStroke2,
    margin: '0 2px',
  },
});
