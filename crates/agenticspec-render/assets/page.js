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
  var nodes = document.querySelectorAll("svg.graph .node[data-part]");
  var edges = document.querySelectorAll("svg.graph .edge");
  var details = document.querySelectorAll(".detail[data-detail]");
  var hint = document.getElementById("pin-hint");
  var pinned = "";

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
  function pin(part) {
    pinned = pinned === part ? "" : part;
    nodes.forEach(function (n) {
      var button = n.querySelector(":scope > [role=button]");
      if (button) button.setAttribute("aria-pressed", String(n.getAttribute("data-part") === pinned));
    });
    if (hint) hint.textContent = pinned ? "selected · Esc clears" : "hover or select";
    show(pinned);
  }
  nodes.forEach(function (n) {
    var part = n.getAttribute("data-part");
    n.addEventListener("mouseenter", function () { show(part); });
    n.addEventListener("mouseleave", function () { show(pinned); });
    // The button of a part is its card's background, so focus arrives from inside the group.
    n.addEventListener("focusin", function (ev) { ev.stopPropagation(); show(part); });
    n.addEventListener("focusout", function (ev) { ev.stopPropagation(); show(pinned); });
    n.addEventListener("click", function (ev) { ev.stopPropagation(); pin(part); });
    n.addEventListener("keydown", function (ev) {
      if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); ev.stopPropagation(); pin(part); }
    });
  });
  document.addEventListener("keydown", function (ev) {
    if (ev.key === "Escape" && pinned) pin(pinned);
  });

  // A table wider than its card scrolls sideways: while it does, its box is a focusable region
  // named for its card, so a keyboard can reach it and scroll it.
  document.querySelectorAll(".table-wrap").forEach(function (w) {
    var head = w.parentElement && w.parentElement.querySelector("h2");
    var label = head ? head.textContent : "Table";
    function measure() {
      if (w.scrollWidth > w.clientWidth + 1) {
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
