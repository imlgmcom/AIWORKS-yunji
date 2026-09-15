// 元素进入视口检测（用于无限滚动的底部哨兵）
import { useEffect, useRef, useState } from 'react';

export function useInView<T extends HTMLElement = HTMLDivElement>(
  options?: IntersectionObserverInit & { enabled?: boolean },
) {
  const ref = useRef<T | null>(null);
  const [inView, setInView] = useState(false);
  const enabled = options?.enabled ?? true;

  // 不传依赖数组：每次渲染后重新观察，确保哨兵晚挂载（数据加载后）也能绑定
  useEffect(() => {
    const el = ref.current;
    if (!el || !enabled) {
      setInView(false);
      return;
    }
    const observer = new IntersectionObserver(
      ([entry]) => setInView(entry.isIntersecting),
      {
        root: options?.root ?? null,
        rootMargin: options?.rootMargin ?? '300px',
        threshold: options?.threshold ?? 0,
      },
    );
    observer.observe(el);
    return () => observer.disconnect();
  });

  return { ref, inView };
}
