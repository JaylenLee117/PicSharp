import { createContext, useContext, useEffect, useRef, useState } from 'react';

export enum Theme {
  Dark = 'dark',
  Light = 'light',
  System = 'system',
}

type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
};

type ThemeProviderState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
};

const initialState: ThemeProviderState = {
  theme: Theme.System,
  setTheme: () => null,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);
const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

export function ThemeProvider({
  children,
  defaultTheme = Theme.System,
  storageKey = 'app-theme',
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(
    () => (localStorage.getItem(storageKey) as Theme) || defaultTheme,
  );
  const themeRef = useRef<Theme>(theme);

  function setThemeStyle(newTheme: Theme.Dark | Theme.Light) {
    const root = window.document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(newTheme);
  }

  function toggleTheme(newTheme: Theme) {
    if (newTheme === Theme.System) {
      const systemTheme = mediaQuery.matches ? Theme.Dark : Theme.Light;
      setThemeStyle(systemTheme);
    } else {
      setThemeStyle(newTheme);
    }
    localStorage.setItem(storageKey, newTheme);
    setTheme(newTheme);
    themeRef.current = newTheme;
  }

  useEffect(() => {
    function handleThemeChange(event: MediaQueryListEvent) {
      if (themeRef.current !== Theme.System) return;
      if (event.matches) {
        setThemeStyle(Theme.Dark);
      } else {
        setThemeStyle(Theme.Light);
      }
    }
    mediaQuery.addEventListener('change', handleThemeChange);
    const currentTheme = localStorage.getItem(storageKey) as Theme;
    if (currentTheme === Theme.System) {
      setThemeStyle(mediaQuery.matches ? Theme.Dark : Theme.Light);
    } else {
      setThemeStyle(currentTheme);
    }
    return () => {
      mediaQuery.removeEventListener('change', handleThemeChange);
    };
  }, []);

  const value = {
    theme,
    setTheme: toggleTheme,
  };

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext);

  if (context === undefined) throw new Error('useTheme must be used within a ThemeProvider');

  return context;
};
