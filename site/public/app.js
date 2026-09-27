"use strict";
const tabs = [...document.querySelectorAll("[data-tab]")];
const panels = [...document.querySelectorAll(".code-panel")];
const copyButton = document.querySelector("#copy-code");
const copyStatus = document.querySelector("#copy-status");

function selectTab(tab, focus = false) {
  tabs.forEach((item) => {
    const active = item === tab;
    item.setAttribute("aria-selected", String(active));
    item.tabIndex = active ? 0 : -1;
  });
  panels.forEach((panel) => { panel.hidden = panel.id !== tab.getAttribute("aria-controls"); });
  copyStatus.textContent = "";
  if (focus) tab.focus();
}

tabs.forEach((tab, index) => {
  tab.addEventListener("click", () => selectTab(tab));
  tab.addEventListener("keydown", (event) => {
    let next;
    if (event.key === "ArrowRight") next = (index + 1) % tabs.length;
    if (event.key === "ArrowLeft") next = (index - 1 + tabs.length) % tabs.length;
    if (event.key === "Home") next = 0;
    if (event.key === "End") next = tabs.length - 1;
    if (next !== undefined) { event.preventDefault(); selectTab(tabs[next], true); }
  });
});

copyButton.addEventListener("click", async () => {
  const code = panels.find((panel) => !panel.hidden).querySelector("code");
  try {
    await navigator.clipboard.writeText(code.textContent);
    copyStatus.textContent = "Copied to clipboard";
  } catch {
    const selection = window.getSelection();
    const range = document.createRange();
    range.selectNodeContents(code);
    selection.removeAllRanges();
    selection.addRange(range);
    copyStatus.textContent = "Text selected. Copy with Ctrl/Cmd+C.";
  }
});

document.querySelector(".tabs").hidden = false;
copyButton.hidden = false;
document.documentElement.classList.add("js-ready");
