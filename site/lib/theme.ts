export const THEME_KEY = "sweeploom-theme";

/* Runs in <head> before first paint so the page never flashes the wrong
   theme. Mirrors the weavatrix.com bootstrap with SweepLoom's own key. */
export const THEME_BOOTSTRAP = `(function(){try{var k='${THEME_KEY}',c=localStorage.getItem(k);if(c!=='light'&&c!=='dark')c='auto';var m=window.matchMedia&&window.matchMedia('(prefers-color-scheme: light)').matches;var e=document.documentElement;e.setAttribute('data-theme',c);e.setAttribute('data-theme-resolved',c==='auto'?(m?'light':'dark'):c)}catch(e){}})()`;
