if (typeof window !== 'undefined') {
  function update() {
    const nav = document.querySelector('.rp-nav');
    if (!nav) return;
    const isHome = /^\/(fr\/?)?$/.test(location.pathname);
    if (isHome) {
      nav.classList.toggle('rp-nav--scrolled', window.scrollY > 50);
    } else {
      nav.classList.add('rp-nav--scrolled');
    }
  }

  const observer = new MutationObserver(() => {
    if (document.querySelector('.rp-nav')) {
      observer.disconnect();
      update();
      window.addEventListener('scroll', update, { passive: true });
      window.addEventListener('popstate', () => setTimeout(update, 0));
      const origPush = history.pushState.bind(history);
      history.pushState = function (...args: Parameters<typeof origPush>) {
        origPush(...args);
        setTimeout(update, 0);
      };
    }
  });
  observer.observe(document.documentElement, { childList: true, subtree: true });
}
