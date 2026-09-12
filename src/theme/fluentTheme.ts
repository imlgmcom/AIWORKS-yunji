// Fluent UI v2 主题配置
import { createLightTheme, createDarkTheme, type Theme } from '@fluentui/react-components';

// 品牌色：靛蓝（slate/indigo）— 完整 20 阶色板
const brandColors = {
  10: '#020617',
  20: '#0f172a',
  30: '#1e293b',
  40: '#1e293b',
  50: '#334155',
  60: '#334155',
  70: '#475569',
  80: '#475569',
  90: '#64748b',
  100: '#64748b',
  110: '#94a3b8',
  120: '#94a3b8',
  130: '#cbd5e1',
  140: '#cbd5e1',
  150: '#e2e8f0',
  160: '#e2e8f0',
  170: '#f1f5f9',
  180: '#f8fafc',
  190: '#f8fafc',
  200: '#ffffff',
};

export const yunjiLightTheme: Theme = createLightTheme(brandColors);
export const yunjiDarkTheme: Theme = createDarkTheme(brandColors);
