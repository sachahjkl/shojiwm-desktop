// The site's behaviour: the Aero / Kernel switch, the lightbox, the gallery's filters, the
// documentation's table of contents and the changelog page. Every part works on the HTML as it is: without this
// script, a screenshot is a link to its file and every gallery picture shows.

"use strict";

/* ----- the menu on a narrow screen -------------------------------------------------------- */

// The three bars open and close the menu; a choice in it, Escape or a press outside closes it.
function setUpMenu() {
  const top = document.querySelector(".top");
  const button = top?.querySelector(".nav-toggle");
  if (!top || !button) {
    return;
  }
  const set = (open) => {
    top.toggleAttribute("data-open", open);
    button.setAttribute("aria-expanded", String(open));
  };
  button.hidden = false;
  button.addEventListener("click", () => set(!top.hasAttribute("data-open")));
  top.querySelectorAll(".nav a").forEach((a) => a.addEventListener("click", () => set(false)));
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && top.hasAttribute("data-open")) {
      set(false);
      button.focus();
    }
  });
  document.addEventListener("click", (e) => {
    if (!top.contains(e.target)) {
      set(false);
    }
  });
}

/* ----- the look: Aero or Kernel ------------------------------------------------------------- */

// The head sets data-look before the first paint (see the inline script of each page).
const LOOKS = ["kernel", "aero"];

// Kernel is the default theme, so the default look.
function currentLook() {
  return document.documentElement.dataset.look === "aero" ? "aero" : "kernel";
}

// Swaps every picture that has a Kernel version (data-kernel) to the look chosen.
function applyLook(look) {
  document.documentElement.dataset.look = look;
  try {
    localStorage.setItem("look", look);
  } catch {
    // Storage refused: the choice lasts for this page only.
  }
  for (const img of document.querySelectorAll("img[data-kernel]")) {
    const src = look === "kernel" ? img.dataset.kernel : img.dataset.aero;
    img.setAttribute("src", src);
    const link = img.closest("a");
    if (link) {
      link.setAttribute("href", src);
    }
  }
  for (const button of document.querySelectorAll(".look-switch button")) {
    button.setAttribute("aria-pressed", String(button.dataset.look === look));
  }
  for (const label of document.querySelectorAll("[data-look-label]")) {
    label.textContent = look === "kernel" ? "Kernel" : "Aero";
  }
}

function setUpLook() {
  const switches = document.querySelectorAll(".look-switch");
  for (const group of switches) {
    group.hidden = false;
    group.addEventListener("click", (event) => {
      const button = event.target.closest("button[data-look]");
      if (button && LOOKS.includes(button.dataset.look)) {
        applyLook(button.dataset.look);
      }
    });
  }
  applyLook(currentLook());
}

/* ----- the lightbox ------------------------------------------------------------------------- */

const ICONS = {
  close: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>',
  prev: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 5l-7 7 7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>',
  next: '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 5l7 7-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" fill="none"/></svg>',
};

// The distance in pixels a finger must travel sideways to change the picture.
const SWIPE_DISTANCE = 50;

function createLightbox() {
  const dialog = document.createElement("dialog");
  dialog.className = "lightbox";
  dialog.setAttribute("aria-modal", "true");
  dialog.setAttribute("aria-labelledby", "lb-caption");
  dialog.innerHTML = `
    <div class="lb-bar">
      <span class="lb-counter" aria-live="polite"></span>
      <a class="lb-file" href="">Open the file</a>
      <button type="button" class="lb-close" aria-label="Close">${ICONS.close}</button>
    </div>
    <div class="lb-stage">
      <img alt="" decoding="async">
      <button type="button" class="lb-prev" aria-label="Previous picture">${ICONS.prev}</button>
      <button type="button" class="lb-next" aria-label="Next picture">${ICONS.next}</button>
    </div>
    <p class="lb-caption" id="lb-caption"></p>`;
  document.body.append(dialog);

  const stage = dialog.querySelector(".lb-stage");
  const image = dialog.querySelector(".lb-stage img");
  const caption = dialog.querySelector(".lb-caption");
  const counter = dialog.querySelector(".lb-counter");
  const file = dialog.querySelector(".lb-file");
  const prevButton = dialog.querySelector(".lb-prev");
  const nextButton = dialog.querySelector(".lb-next");

  let links = [];
  let index = 0;
  let opener = null;

  // The pictures that the reader can see now: a filtered-out gallery card does not count.
  function visibleLinks() {
    return [...document.querySelectorAll("a[data-lightbox]")].filter(
      (a) => a.getClientRects().length > 0,
    );
  }

  function captionOf(link) {
    const figure = link.closest("figure");
    const text = figure?.querySelector("figcaption")?.textContent.trim();
    return text || link.querySelector("img")?.alt || "";
  }

  function preload(i) {
    const link = links[(i + links.length) % links.length];
    if (link) {
      const img = new Image();
      img.src = link.href;
    }
  }

  function show(i) {
    index = (i + links.length) % links.length;
    const link = links[index];
    const thumbnail = link.querySelector("img");
    stage.classList.add("loading");
    image.onload = () => stage.classList.remove("loading");
    image.onerror = () => stage.classList.remove("loading");
    image.src = link.href;
    image.alt = thumbnail?.alt || "";
    image.classList.toggle("details", link.hasAttribute("data-pixelated"));
    caption.textContent = captionOf(link);
    counter.textContent = `${index + 1} / ${links.length}`;
    file.href = link.href;
    const alone = links.length < 2;
    prevButton.hidden = alone;
    nextButton.hidden = alone;
    if (!alone) {
      preload(index + 1);
      preload(index - 1);
    }
  }

  function open(link) {
    links = visibleLinks();
    const start = links.indexOf(link);
    if (start < 0) {
      return false;
    }
    opener = link;
    document.documentElement.classList.add("lightbox-open");
    show(start);
    dialog.showModal();
    dialog.querySelector(".lb-close").focus();
    return true;
  }

  function close() {
    if (dialog.open) {
      dialog.close();
    }
  }

  dialog.addEventListener("close", () => {
    document.documentElement.classList.remove("lightbox-open");
    image.removeAttribute("src");
    opener?.focus();
    opener = null;
  });

  // Escape: the browser sends "cancel" and closes; the "close" handler restores the page.
  dialog.addEventListener("keydown", (event) => {
    if (event.key === "ArrowRight") {
      event.preventDefault();
      show(index + 1);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      show(index - 1);
    } else if (event.key === "Home") {
      event.preventDefault();
      show(0);
    } else if (event.key === "End") {
      event.preventDefault();
      show(links.length - 1);
    } else if (event.key === "Tab") {
      // Keep the focus inside the dialog.
      const focusable = [...dialog.querySelectorAll("a[href], button:not([hidden])")];
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  });

  dialog.querySelector(".lb-close").addEventListener("click", close);
  prevButton.addEventListener("click", () => show(index - 1));
  nextButton.addEventListener("click", () => show(index + 1));

  // A click on the dark area around the picture closes the lightbox.
  stage.addEventListener("click", (event) => {
    if (event.target === stage) {
      close();
    }
  });

  // Swipe on a touch screen.
  let touchStart = null;
  stage.addEventListener("pointerdown", (event) => {
    if (event.pointerType === "touch") {
      touchStart = { x: event.clientX, y: event.clientY };
    }
  });
  stage.addEventListener("pointerup", (event) => {
    if (!touchStart || event.pointerType !== "touch") {
      return;
    }
    const dx = event.clientX - touchStart.x;
    const dy = event.clientY - touchStart.y;
    touchStart = null;
    if (Math.abs(dx) > SWIPE_DISTANCE && Math.abs(dx) > Math.abs(dy)) {
      show(dx < 0 ? index + 1 : index - 1);
    }
  });
  stage.addEventListener("pointercancel", () => {
    touchStart = null;
  });

  document.addEventListener("click", (event) => {
    const link = event.target.closest("a[data-lightbox]");
    // A click with a modifier opens the file the browser's way (a new tab, a download).
    if (!link || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
      return;
    }
    if (open(link)) {
      event.preventDefault();
    }
  });
}

/* ----- the gallery's filters ---------------------------------------------------------------- */

function setUpGallery() {
  const filters = document.querySelector(".filters");
  if (!filters) {
    return;
  }
  filters.hidden = false;
  const cards = [...document.querySelectorAll(".gallery .shot")];
  const sections = [...document.querySelectorAll(".gallery section")];
  const count = filters.querySelector(".count");
  const params = new URLSearchParams(location.search);
  const state = {
    theme: params.get("theme") || "both",
    area: params.get("area") || "all",
  };

  function apply() {
    let shown = 0;
    for (const card of cards) {
      const theme = card.dataset.theme;
      const area = card.dataset.area;
      const visible =
        (state.theme === "both" || theme === state.theme) &&
        (state.area === "all" || area === state.area);
      card.hidden = !visible;
      if (visible) {
        shown += 1;
      }
    }
    for (const section of sections) {
      section.hidden = !section.querySelector(".shot:not([hidden])");
    }
    for (const button of filters.querySelectorAll("button")) {
      const on = state[button.dataset.filter] === button.dataset.value;
      button.setAttribute("aria-pressed", String(on));
    }
    count.textContent = `${shown} ${shown === 1 ? "picture" : "pictures"}`;
    const query = new URLSearchParams();
    if (state.theme !== "both") {
      query.set("theme", state.theme);
    }
    if (state.area !== "all") {
      query.set("area", state.area);
    }
    const search = query.toString();
    history.replaceState(null, "", search ? `?${search}` : location.pathname);
  }

  filters.addEventListener("click", (event) => {
    const button = event.target.closest("button[data-filter]");
    if (button) {
      state[button.dataset.filter] = button.dataset.value;
      apply();
    }
  });
  apply();
}

/* ----- the documentation's table of contents ------------------------------------------------ */

// Marks the entry of the section in view.
function setUpToc() {
  const toc = document.querySelector(".toc");
  if (!toc || !("IntersectionObserver" in window)) {
    return;
  }
  const entries = new Map();
  for (const link of toc.querySelectorAll('a[href^="#"]')) {
    const target = document.getElementById(link.hash.slice(1));
    if (target) {
      entries.set(target, link);
    }
  }
  let current = null;
  const observer = new IntersectionObserver(
    (seen) => {
      for (const entry of seen) {
        if (entry.isIntersecting) {
          current?.removeAttribute("aria-current");
          current = entries.get(entry.target);
          current?.setAttribute("aria-current", "true");
        }
      }
    },
    { rootMargin: "-15% 0px -75% 0px" },
  );
  for (const target of entries.keys()) {
    observer.observe(target);
  }
}

/* ----- the changelog ------------------------------------------------------------------------ */

// The changelog page fills its container from changelog.md, the CHANGELOG.md of the repository
// that the Pages workflow copies beside the site. The renderer reads only the Markdown that the
// file uses (Keep a Changelog) and builds every node with createElement and textContent: the
// file's text never goes through innerHTML.

const CHANGELOG_ON_GITHUB = "https://github.com/sachahjkl/shojiwm-desktop/blob/main/CHANGELOG.md";

// The sections of Keep a Changelog; each one gets its own tag colour.
const CHANGE_KINDS = ["added", "changed", "deprecated", "removed", "fixed", "security"];

// Only http, https and relative addresses become links.
function safeHref(url) {
  if (/^https?:\/\//i.test(url)) {
    return url;
  }
  if (/^[a-z][a-z0-9+.-]*:/i.test(url) || url.startsWith("//")) {
    return null;
  }
  return url;
}

// Appends the inline Markdown of text to parent: **bold**, `code` and [text](url).
function appendInline(parent, text) {
  const pattern = /`([^`]+)`|\*\*(.+?)\*\*|\[([^\]]+)\]\(([^)\s]+)\)/g;
  let last = 0;
  for (const match of text.matchAll(pattern)) {
    if (match.index > last) {
      parent.append(text.slice(last, match.index));
    }
    if (match[1] !== undefined) {
      const code = document.createElement("code");
      code.textContent = match[1];
      parent.append(code);
    } else if (match[2] !== undefined) {
      const strong = document.createElement("strong");
      appendInline(strong, match[2]);
      parent.append(strong);
    } else {
      const href = safeHref(match[4]);
      if (href) {
        const link = document.createElement("a");
        link.href = href;
        appendInline(link, match[3]);
        parent.append(link);
      } else {
        appendInline(parent, match[3]);
      }
    }
    last = match.index + match[0].length;
  }
  if (last < text.length) {
    parent.append(text.slice(last));
  }
}

// "## [0.1.0] - 2026-11-11" or "## [Unreleased]": the section of one version.
function versionSection(title) {
  const match = /^\[([^\]]+)\](?:\s+-\s+(.+))?$/.exec(title);
  const name = match ? match[1] : title;
  const date = match?.[2]?.trim();
  const unreleased = name.toLowerCase() === "unreleased";
  const section = document.createElement("section");
  section.className = "cl-version";
  section.id = unreleased ? "unreleased" : `v${name.replace(/[^\w.-]/g, "-")}`;
  const heading = document.createElement("h2");
  const label = document.createElement("a");
  label.className = "cl-name";
  label.href = `#${section.id}`;
  label.textContent = unreleased ? "Unreleased" : name;
  heading.append(label);
  if (date) {
    const time = document.createElement(/^\d{4}-\d{2}-\d{2}$/.test(date) ? "time" : "span");
    time.className = "cl-date";
    if (time.tagName === "TIME") {
      time.dateTime = date;
    }
    time.textContent = date;
    heading.append(" ", time);
  }
  section.append(heading);
  return section;
}

function renderChangelog(markdown) {
  const fragment = document.createDocumentFragment();
  let parent = fragment; // the version section, once the first "##" is read
  let list = null;
  let item = null; // { node, text } of the list item being read
  let paragraph = null; // the lines of the paragraph being read

  function flushItem() {
    if (item) {
      appendInline(item.node, item.text);
      item = null;
    }
  }
  function flushParagraph() {
    if (paragraph) {
      const p = document.createElement("p");
      appendInline(p, paragraph.join(" "));
      parent.append(p);
      paragraph = null;
    }
  }
  function flushAll() {
    flushItem();
    flushParagraph();
    list = null;
  }

  for (const raw of markdown.split(/\r?\n/)) {
    const line = raw.trimEnd();
    if (/^#\s/.test(line)) {
      flushAll(); // the page has its own title
    } else if (/^##\s/.test(line)) {
      flushAll();
      parent = versionSection(line.slice(3).trim());
      fragment.append(parent);
    } else if (/^###\s/.test(line)) {
      flushAll();
      const title = line.slice(4).trim();
      const heading = document.createElement("h3");
      const kind = title.toLowerCase();
      heading.className = CHANGE_KINDS.includes(kind) ? `cl-kind cl-${kind}` : "cl-kind";
      heading.textContent = title;
      parent.append(heading);
    } else if (/^\[[^\]]+\]:\s/.test(line)) {
      flushAll(); // a link reference (the versions' compare links): not shown
    } else if (/^[-*]\s/.test(line)) {
      flushItem();
      flushParagraph();
      if (!list) {
        list = document.createElement("ul");
        list.className = "cl-list";
        parent.append(list);
      }
      item = { node: document.createElement("li"), text: line.slice(2).trim() };
      list.append(item.node);
    } else if (line.trim() === "") {
      flushItem();
      flushParagraph();
    } else if (item && /^\s/.test(line)) {
      item.text += ` ${line.trim()}`;
    } else {
      flushItem();
      list = null;
      if (!paragraph) {
        paragraph = [];
      }
      paragraph.push(line.trim());
    }
  }
  flushAll();
  return fragment;
}

function changelogError(container) {
  const p = document.createElement("p");
  p.className = "cl-error";
  p.append("The changelog could not be loaded. Read ");
  const link = document.createElement("a");
  link.href = CHANGELOG_ON_GITHUB;
  link.textContent = "CHANGELOG.md on GitHub";
  p.append(link, ".");
  container.replaceChildren(p);
}

async function setUpChangelog() {
  const container = document.querySelector("[data-changelog]");
  if (!container) {
    return;
  }
  container.setAttribute("aria-busy", "true");
  try {
    const response = await fetch(container.dataset.changelog, { cache: "no-cache" });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    container.replaceChildren(renderChangelog(await response.text()));
    // The address can name a version (#v0.1.0): scroll to it now that it exists.
    if (location.hash) {
      document.getElementById(location.hash.slice(1))?.scrollIntoView();
    }
  } catch {
    changelogError(container);
  } finally {
    container.removeAttribute("aria-busy");
  }
}

setUpMenu();
setUpLook();
setUpGallery();
setUpToc();
setUpChangelog();
if (typeof HTMLDialogElement === "function") {
  createLightbox();
}
