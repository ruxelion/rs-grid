(function () {
  function update() {
    var nav = document.querySelector('.rp-nav');
    if (!nav) return;
    var isHome = /^\/(fr\/?)?$/.test(location.pathname);
    if (isHome) {
      nav.classList.toggle('rp-nav--scrolled', window.scrollY > 50);
    } else {
      nav.classList.add('rp-nav--scrolled');
    }
  }

  function bind() {
    update();
    window.addEventListener('scroll', update, { passive: true });
    window.addEventListener('popstate', function () { setTimeout(update, 0); });
    var origPush = history.pushState;
    history.pushState = function () {
      origPush.apply(history, arguments);
      setTimeout(update, 0);
    };
    var origReplace = history.replaceState;
    history.replaceState = function () {
      origReplace.apply(history, arguments);
      setTimeout(update, 0);
    };
  }

  // Wait for .rp-nav to appear in the DOM
  var observer = new MutationObserver(function () {
    if (document.querySelector('.rp-nav')) {
      observer.disconnect();
      bind();
    }
  });
  observer.observe(document.documentElement, { childList: true, subtree: true });
})();
