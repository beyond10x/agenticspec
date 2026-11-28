"use strict";
(function () {
  var root = document.documentElement;
  var choices = document.querySelectorAll("[data-theme-choice]");
  function theme(t) {
    if (t === "system") root.removeAttribute("data-theme");
    else root.setAttribute("data-theme", t);
    choices.forEach(function (b) {
      b.setAttribute("aria-pressed", String(b.getAttribute("data-theme-choice") === t));
    });
  }
  choices.forEach(function (b) {
    b.addEventListener("click", function () { theme(b.getAttribute("data-theme-choice")); });
  });
  var asked = new URLSearchParams(location.search).get("theme");
  if (asked === "light" || asked === "dark") theme(asked);

  var svg = document.querySelector("svg.graph");
  var nodes = [];
  var edges = [];
  var details = document.querySelectorAll(".detail[data-detail]");
  var hint = document.getElementById("pin-hint");
  var picks = document.querySelectorAll("button.pick[data-pick]");
  var pinned = "";
  function collect() {
    nodes = svg ? svg.querySelectorAll(".node[data-part]") : [];
    edges = svg ? svg.querySelectorAll(".edge") : [];
  }
  collect();

  function show(part) {
    var found = false;
    details.forEach(function (d) {
      var on = d.getAttribute("data-detail") === part;
      d.hidden = !on;
      found = found || on;
    });
    if (!found) {
      details.forEach(function (d) { d.hidden = d.getAttribute("data-detail") !== ""; });
    }
    if (svg) svg.classList.toggle("focus", part !== "");
    edges.forEach(function (e) {
      var hot = part !== "" && (e.getAttribute("data-from") === part || e.getAttribute("data-to") === part || e.getAttribute("data-over") === part);
      e.classList.toggle("hot", hot);
    });
  }
  function marks() {
    nodes.forEach(function (n) {
      var button = n.querySelector(":scope > [role=button]");
      if (button && !n.hasAttribute("data-open")) button.setAttribute("aria-pressed", String(n.getAttribute("data-part") === pinned));
    });
    picks.forEach(function (b) {
      b.setAttribute("aria-pressed", String(b.getAttribute("data-pick") === pinned));
    });
    if (hint) hint.textContent = pinned ? "selected · Esc clears" : "hover or select";
    show(pinned);
  }
  function pin(part) {
    pinned = pinned === part ? "" : part;
    marks();
  }
  function activate(el, act) {
    el.addEventListener("click", function (ev) { ev.stopPropagation(); act(); });
    el.addEventListener("keydown", function (ev) {
      if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); ev.stopPropagation(); act(); }
    });
  }
  // The handlers of the graph shown; a state of the graph swapped in gets its own once.
  function bind() {
    if (!svg || svg.__bound) return;
    svg.__bound = true;
    svg.querySelectorAll(".node[data-part]").forEach(function (n) {
      var part = n.getAttribute("data-part");
      var opens = n.getAttribute("data-open");
      n.addEventListener("mouseenter", function () { show(part); });
      n.addEventListener("mouseleave", function () { show(pinned); });
      // The button of a part is its card's background, so focus arrives from inside the group.
      n.addEventListener("focusin", function (ev) { ev.stopPropagation(); show(part); });
      n.addEventListener("focusout", function (ev) { ev.stopPropagation(); show(pinned); });
      // The card of a collapsed region opens it; any other card is selected.
      activate(n, opens ? function () { go(opens, '[data-close="' + CSS.escape(opens) + '"] [role=button]'); } : function () { pin(part); });
    });
    svg.querySelectorAll("[data-close]").forEach(function (c) {
      var id = c.getAttribute("data-close");
      activate(c, function () { go(up[id] || "", '.node[data-open="' + CSS.escape(id) + '"] > [role=button]'); });
    });
  }
  document.addEventListener("keydown", function (ev) {
    if (ev.key === "Escape" && pinned) pin(pinned);
  });

  // The states of a graph with collapsed regions: the page as served, and one per region with
  // that region open, each laid out in full in a template beside the graph. Opening a region
  // swaps its state in; its control swaps in the state it was opened from.
  var view = document.querySelector(".graph-view");
  var states = {"": svg};
  var up = {};
  if (view) {
    view.querySelectorAll("template[data-state]").forEach(function (t) {
      up[t.getAttribute("data-state")] = t.getAttribute("data-up") || "";
    });
  }
  function go(id, focus) {
    var next = states[id];
    if (!next) {
      var t = view.querySelector('template[data-state="' + CSS.escape(id) + '"]');
      var s = t && t.content.querySelector("svg.graph");
      if (!s) return;
      next = states[id] = document.importNode(s, true);
    }
    svg.replaceWith(next);
    svg = next;
    svg.style.width = "";
    svg.style.height = "";
    natural = svg.viewBox && svg.viewBox.baseVal;
    collect();
    bind();
    if (pinned && !svg.querySelector('.node[data-part="' + CSS.escape(pinned) + '"]')) pinned = "";
    marks();
    if (initial === "fit") opening();
    else setZoom(zoom);
    var el = focus && svg.querySelector(focus);
    if (el) el.focus();
  }
  bind();

  // The graph opens whole: larger than its view (wider than the card or taller than the view's
  // highest), it opens at Fit; else at its natural size. It pans by drag, wheel and arrow keys
  // while it overflows (the view is then a focusable scroll region), and the zoom tools show:
  // out, in, fit, actual size. Until a zoom tool is used, a resized view fits again.
  var tools = document.querySelector(".graph-tools");
  var level = tools && tools.querySelector(".zoom-level");
  var natural = svg && svg.viewBox && svg.viewBox.baseVal;
  var zoom = 1;
  var initial = view ? view.getAttribute("data-initial") : null;
  // The room of the view: its width with any scroll bar, and its highest height.
  function room() {
    var h = parseFloat(getComputedStyle(view).maxHeight);
    return {w: view.offsetWidth - 2, h: (isFinite(h) ? h : Infinity) - 2};
  }
  function larger() {
    var r = room();
    return r.w > 0 && (natural.width > r.w || natural.height > r.h);
  }
  function fitZoom() {
    var r = room();
    if (!natural || !natural.width || !natural.height || r.w <= 0 || r.h <= 0) return 1;
    return Math.min(1, r.w / natural.width, r.h / natural.height);
  }
  function measureView() {
    if (!view || !natural) return;
    if (tools) tools.hidden = !larger() && zoom === 1;
    var scrolls = view.scrollWidth > view.clientWidth + 1 || view.scrollHeight > view.clientHeight + 1;
    view.classList.toggle("pannable", scrolls);
    if (scrolls) {
      view.setAttribute("tabindex", "0");
      view.setAttribute("role", "region");
      view.setAttribute("aria-label", "Graph: drag, scroll or use the arrow keys to move around");
    } else {
      view.removeAttribute("tabindex");
      view.removeAttribute("role");
      view.removeAttribute("aria-label");
    }
  }
  function setZoom(z) {
    if (!view || !natural) return;
    var lo = Math.min(fitZoom(), 0.25);
    z = Math.max(lo, Math.min(2, z));
    var cx = (view.scrollLeft + view.clientWidth / 2) / zoom;
    var cy = (view.scrollTop + view.clientHeight / 2) / zoom;
    zoom = z;
    svg.style.width = natural.width * z + "px";
    svg.style.height = natural.height * z + "px";
    view.scrollLeft = cx * z - view.clientWidth / 2;
    view.scrollTop = cy * z - view.clientHeight / 2;
    if (level) level.textContent = Math.round(z * 100) + " %";
    measureView();
  }
  if (tools) {
    tools.querySelectorAll("button[data-zoom]").forEach(function (b) {
      b.addEventListener("click", function () {
        var what = b.getAttribute("data-zoom");
        initial = null;
        if (what === "in") setZoom(zoom * 1.25);
        else if (what === "out") setZoom(zoom / 1.25);
        else if (what === "fit") setZoom(fitZoom());
        else setZoom(1);
      });
    });
  }
  function opening() {
    if (initial === "fit" && natural && larger()) setZoom(fitZoom());
    else if (initial === "fit" && zoom !== 1) setZoom(1);
    else measureView();
  }
  if (view) {
    opening();
    if (typeof ResizeObserver !== "undefined") new ResizeObserver(opening).observe(view);
    var drag = null;
    var moved = false;
    view.addEventListener("pointerdown", function (ev) {
      if (ev.pointerType !== "mouse" || ev.button !== 0 || !view.classList.contains("pannable")) return;
      drag = {x: ev.clientX, y: ev.clientY, left: view.scrollLeft, top: view.scrollTop};
      moved = false;
    });
    view.addEventListener("pointermove", function (ev) {
      if (!drag) return;
      var dx = ev.clientX - drag.x, dy = ev.clientY - drag.y;
      if (!moved && Math.abs(dx) + Math.abs(dy) < 5) return;
      if (!moved) {
        moved = true;
        view.classList.add("dragging");
        view.setPointerCapture(ev.pointerId);
      }
      view.scrollLeft = drag.left - dx;
      view.scrollTop = drag.top - dy;
    });
    function stop() {
      drag = null;
      view.classList.remove("dragging");
    }
    view.addEventListener("pointerup", stop);
    view.addEventListener("pointercancel", stop);
    // A drag ends without selecting the part it started on.
    view.addEventListener("click", function (ev) {
      if (moved) { ev.stopPropagation(); moved = false; }
    }, true);
  }
  function reveal(part) {
    var node = svg && svg.querySelector('.node[data-part="' + CSS.escape(part) + '"]');
    if (!node || !view) return;
    var v = view.getBoundingClientRect(), r = node.getBoundingClientRect();
    if (r.left < v.left || r.right > v.right) view.scrollLeft += r.left - v.left - (v.width - r.width) / 2;
    if (r.top < v.top || r.bottom > v.bottom) view.scrollTop += r.top - v.top - (v.height - r.height) / 2;
  }
  picks.forEach(function (b) {
    b.addEventListener("click", function () {
      var part = b.getAttribute("data-pick");
      pin(part);
      if (pinned) reveal(part);
    });
  });

  // The links table, filtered by the part a link starts at, ends at or passes over.
  var filter = document.querySelector("select[data-link-filter]");
  if (filter) {
    var rows = document.querySelectorAll(".links-table tbody tr");
    var count = document.querySelector("[data-link-count]");
    filter.addEventListener("change", function () {
      var part = filter.value, shown = 0;
      rows.forEach(function (r) {
        var on = !part || r.getAttribute("data-from") === part || r.getAttribute("data-to") === part || r.getAttribute("data-over") === part;
        r.hidden = !on;
        if (on) shown++;
      });
      if (count) count.textContent = part ? shown + " of " + rows.length + " links" : rows.length + " links";
    });
  }

  // A table wider (or, in a scrolling box, taller) than its box scrolls: while it does, its box is a focusable region
  // named for its card, so a keyboard can reach it and scroll it.
  document.querySelectorAll(".table-wrap").forEach(function (w) {
    var head = w.parentElement && w.parentElement.querySelector("h2");
    var label = head ? head.textContent : "Table";
    function measure() {
      if (w.scrollWidth > w.clientWidth + 1 || w.scrollHeight > w.clientHeight + 1) {
        w.setAttribute("tabindex", "0");
        w.setAttribute("role", "region");
        w.setAttribute("aria-label", label);
      } else {
        w.removeAttribute("tabindex");
        w.removeAttribute("role");
        w.removeAttribute("aria-label");
      }
    }
    measure();
    if (typeof ResizeObserver !== "undefined") new ResizeObserver(measure).observe(w);
  });

  // Landscape: the pair of places shown.
  document.querySelectorAll("select[data-pick=pair]").forEach(function (s) {
    var card = s.closest(".card");
    function pick() {
      card.querySelectorAll("figure.pair[data-pair]").forEach(function (f) {
        f.hidden = f.getAttribute("data-pair") !== s.value;
      });
    }
    s.addEventListener("change", pick);
    pick();
  });

  // Trade-offs: every position and every Pareto front is in the page; a choice of KPIs shows one.
  var ax = document.querySelector("select[data-axis=x]");
  var ay = document.querySelector("select[data-axis=y]");
  var scatter = document.querySelector("svg.scatter");
  if (ax && ay && scatter) {
    var label = document.getElementById("to-t");
    var plot = function () {
      var x = ax.value, y = ay.value, front = null;
      scatter.querySelectorAll(".ax-x").forEach(function (g) { g.classList.toggle("off", g.getAttribute("data-kpi") !== x); });
      scatter.querySelectorAll(".ax-y").forEach(function (g) { g.classList.toggle("off", g.getAttribute("data-kpi") !== y); });
      scatter.querySelectorAll("path.front").forEach(function (p) {
        var on = p.getAttribute("data-pair") === x + "|" + y;
        p.classList.toggle("off", !on);
        if (on) front = p;
      });
      var ids = front ? front.getAttribute("data-front").split(" ") : [];
      if (label) label.textContent = front ? front.getAttribute("data-label") : "Trade-offs: " + x + " against itself";
      scatter.querySelectorAll("g.sp").forEach(function (g) {
        var px = g.getAttribute("data-x-" + x), py = g.getAttribute("data-y-" + y);
        var ok = px !== null && py !== null;
        g.classList.toggle("off", !ok);
        if (ok) g.setAttribute("transform", "translate(" + px + "," + py + ")");
        g.classList.toggle("front-pt", ids.indexOf(g.getAttribute("data-v")) >= 0);
      });
    };
    ax.addEventListener("change", plot);
    ay.addEventListener("change", plot);
    plot();
  }
})();
