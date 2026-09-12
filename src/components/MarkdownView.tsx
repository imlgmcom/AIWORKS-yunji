// Markdown 渲染器（基于 Vditor.md2html）
import { useEffect, useState } from 'react';
import Vditor from 'vditor';
import 'vditor/dist/index.css';
import { rewriteAssetHtml, torrentApi } from '../lib/tauri';

interface MarkdownViewProps {
  content: string;
  className?: string;
}

export function MarkdownView({ content, className }: MarkdownViewProps) {
  const [html, setHtml] = useState('');

  useEffect(() => {
    let cancelled = false;
    const md = content || '';
    if (!md.trim()) {
      setHtml('');
      return;
    }
    Vditor.md2html(md, {
      mode: 'light',
      hljs: { style: 'github', lineNumber: false },
      math: { engine: 'KaTeX' },
      cdn: '/vditor',
    }).then((result) => {
      if (!cancelled) setHtml(rewriteAssetHtml(result, 'view'));
    });
    return () => { cancelled = true; };
  }, [content]);

  // 正文中的本地附件链接：用系统默认软件打开（而非让 webview 导航）
  const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const anchor = (e.target as HTMLElement).closest<HTMLAnchorElement>('a[data-asset]');
    if (!anchor) return;
    e.preventDefault();
    const rel = anchor.dataset.asset;
    if (rel) {
      torrentApi.openFile(rel).catch((err) => {
        console.error('打开附件失败:', err);
      });
    }
  };

  if (!html) return null;

  return (
    <div
      className={className}
      onClick={handleClick}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
