(function () {
  "use strict";

  function normalizeTail(path) {
    if (!path) return "";
    var tail = path;
    if (tail.startsWith("/")) tail = tail.slice(1);
    if (tail === "index.html" || tail === "search-index.json") return "";
    tail = tail.replace(/(^|\/)index\.html$/, "$1");
    if (tail.endsWith("search-index.json")) return "";
    if (tail.length > 0 && !tail.endsWith("/")) tail += "/";
    return tail;
  }

  function currentVersionAndTail(pathname) {
    var m = pathname.match(/^\/website\/documentation\/(v[^\/]+)\/?(.*)$/);
    if (!m) return { version: null, tail: "" };
    return { version: m[1], tail: normalizeTail(m[2]) };
  }

  fetch('/website/documentation/versions.json')
    .then(function (res) { return res.json(); })
    .then(function (data) {
      var selectors = document.querySelectorAll('[data-docs-version-select]');
      if (!selectors.length) return;

      var state = currentVersionAndTail(window.location.pathname);
      var selected = state.version || data.latest;

      selectors.forEach(function (select) {
        data.versions.forEach(function (item) {
          var opt = document.createElement('option');
          opt.value = item.id;
          opt.textContent = item.label;
          if (item.id === selected) opt.selected = true;
          select.appendChild(opt);
        });

        select.addEventListener('change', function () {
          var next = select.value;
          if (!next) return;

          if (state.version) {
            var target = '/website/documentation/' + next + '/';
            if (state.tail) target += state.tail;
            target += 'index.html';
            window.location.href = target;
            return;
          }

          window.location.href = '/website/documentation/' + next + '/index.html';
        });
      });
    })
    .catch(function () {
      /* no-op */
    });
})();
