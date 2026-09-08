(function () {
  "use strict";

  function currentVersion() {
    var match = window.location.pathname.match(/^\/website\/documentation\/(v[^/]+)\//);
    return match ? match[1] : null;
  }

  function normalize(text) {
    return (text || "").toLowerCase();
  }

  function escapeHtml(text) {
    return (text || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function tokenize(query) {
    return normalize(query)
      .split(/\s+/)
      .map(function (part) { return part.trim(); })
      .filter(function (part) { return part.length >= 2; });
  }

  function snippet(text, term) {
    var source = text.replace(/\s+/g, " ").trim();
    var lower = normalize(source);
    var index = lower.indexOf(term);
    if (index < 0) return escapeHtml(source.slice(0, 180));

    var start = Math.max(0, index - 70);
    var end = Math.min(source.length, index + term.length + 110);
    var cut = source.slice(start, end);
    var safe = escapeHtml(cut);
    var re = new RegExp("(" + term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + ")", "ig");
    safe = safe.replace(re, "<mark>$1</mark>");
    return (start > 0 ? "..." : "") + safe + (end < source.length ? "..." : "");
  }

  function score(entry, terms) {
    var title = normalize(entry.title);
    var summary = normalize(entry.summary);
    var text = normalize(entry.text);
    var total = 0;

    for (var i = 0; i < terms.length; i++) {
      var term = terms[i];
      if (!text.includes(term) && !title.includes(term) && !summary.includes(term)) return 0;
      if (title.includes(term)) total += 20;
      if (summary.includes(term)) total += 8;
      total += Math.min(10, (text.split(term).length - 1));
    }

    return total;
  }

  function renderResults(target, meta, entries, query) {
    var terms = tokenize(query);
    if (!terms.length) {
      target.innerHTML = "";
      meta.textContent = "Type at least two characters.";
      return;
    }

    var results = entries
      .map(function (entry) {
        return { entry: entry, score: score(entry, terms) };
      })
      .filter(function (item) { return item.score > 0; })
      .sort(function (a, b) { return b.score - a.score || a.entry.title.localeCompare(b.entry.title); })
      .slice(0, 12);

    if (!results.length) {
      target.innerHTML = "";
      meta.textContent = "No matches.";
      return;
    }

    meta.textContent = results.length + " match" + (results.length === 1 ? "" : "es") + ".";
    target.innerHTML = results.map(function (item) {
      var entry = item.entry;
      return '<a class="docs-search-result" href="' + escapeHtml(entry.url) + '">' +
        '<span class="docs-search-section">' + escapeHtml(entry.section) + '</span>' +
        '<strong>' + escapeHtml(entry.title) + '</strong>' +
        '<span>' + escapeHtml(entry.summary) + '</span>' +
        '<small>' + snippet(entry.text, terms[0]) + '</small>' +
        '</a>';
    }).join("");
  }

  var roots = document.querySelectorAll("[data-docs-search]");
  if (!roots.length) return;

  function loadIndex(version) {
    return fetch("/website/documentation/" + version + "/search-index.json").then(function (res) {
      return res.json();
    });
  }

  var version = currentVersion();
  var indexPromise = version
    ? loadIndex(version)
    : fetch("/website/documentation/versions.json")
      .then(function (res) { return res.json(); })
      .then(function (data) { return loadIndex(data.latest); });

  indexPromise
    .then(function (data) {
      roots.forEach(function (root) {
        var input = root.querySelector("[data-docs-search-input]");
        var meta = root.querySelector("[data-docs-search-meta]");
        var results = root.querySelector("[data-docs-search-results]");
        if (!input || !meta || !results) return;

        input.addEventListener("input", function () {
          renderResults(results, meta, data.entries || [], input.value);
        });
      });
    })
    .catch(function () {
      roots.forEach(function (root) {
        var meta = root.querySelector("[data-docs-search-meta]");
        if (meta) meta.textContent = "Search index unavailable.";
      });
    });
})();
