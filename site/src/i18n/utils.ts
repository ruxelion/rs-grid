import {
  translations,
  defaultLang,
  type Lang,
} from './translations';

export function getLangFromUrl(url: URL): Lang {
  const [, lang] = url.pathname.split('/');
  if (lang in translations) return lang as Lang;
  return defaultLang;
}

export function t(lang: Lang, key: string): string {
  return translations[lang][key] ?? translations[defaultLang][key] ?? key;
}

/** Build a localized path. EN has no prefix, FR gets /fr. */
export function localePath(lang: Lang, path: string): string {
  const clean = path.startsWith('/') ? path : `/${path}`;
  if (lang === defaultLang) return clean;
  const full = `/${lang}${clean}`;
  // Remove trailing slash (except bare "/")
  return full.endsWith('/') && full.length > 1 ? full.slice(0, -1) : full;
}

/** Get the alternate language. */
export function alternateLang(lang: Lang): Lang {
  return lang === 'en' ? 'fr' : 'en';
}

/** Build path to switch language on the current page. */
export function switchLangPath(url: URL): string {
  const lang = getLangFromUrl(url);
  const alt = alternateLang(lang);
  if (lang === defaultLang) {
    // EN -> FR: add /fr prefix
    const full = `/${alt}${url.pathname}`;
    return full.endsWith('/') && full.length > 1 ? full.slice(0, -1) : full;
  }
  // FR -> EN: remove /fr prefix
  const pathWithoutLang = url.pathname.replace(`/${lang}`, '') || '/';
  return pathWithoutLang;
}
