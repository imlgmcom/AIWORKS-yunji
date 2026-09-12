// 认证状态（Zustand）
import { create } from 'zustand';
import type { UserInfo } from '../types/models';
import { authApi } from '../lib/tauri';

interface AuthStore {
  user: UserInfo | null;
  loading: boolean;
  init: () => Promise<void>;
  setUser: (user: UserInfo | null) => void;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

export const useAuthStore = create<AuthStore>((set) => ({
  user: null,
  loading: true,

  init: async () => {
    try {
      const user = await authApi.me();
      set({ user, loading: false });
    } catch {
      set({ user: null, loading: false });
    }
  },

  setUser: (user) => set({ user }),

  login: async (username, password) => {
    const user = await authApi.login(username, password);
    set({ user });
  },

  logout: async () => {
    await authApi.logout();
    set({ user: null });
  },
}));
