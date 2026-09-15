"use strict";

const HOST = "com.sweeploom.companion";
const VERSION = "0.1.0";
const INSTANCE = `${chrome.runtime.id || "firefox"}:${Date.now()}`;
let epoch = Date.now();

let port = null;
let sendTimer = 0;

function connect() {
  if (port) {
    return port;
  }
  try {
    port = chrome.runtime.connectNative(HOST);
  } catch (_error) {
    port = null;
    return null;
  }
  epoch += 1;
  port.onDisconnect.addListener(() => {
    port = null;
  });
  port.onMessage.addListener((message) => {
    if (message && message.type === "apply" && Array.isArray(message.actions)) {
      void applyActions(message.actions);
    }
  });
  try {
    port.postMessage({
      type: "hello",
      version: VERSION,
      instance_id: INSTANCE,
      epoch,
    });
  } catch (_error) {
    port = null;
    return null;
  }
  return port;
}

function safeUrl(raw) {
  if (!raw) {
    return "";
  }
  try {
    const url = new URL(raw);
    url.username = "";
    url.password = "";
    url.search = "";
    url.hash = "";
    return url.href;
  } catch (_error) {
    return "";
  }
}

function snapshot(tab) {
  return {
    tab_id: tab.id,
    window_id: tab.windowId,
    title: tab.title || "",
    url: safeUrl(tab.url || tab.pendingUrl || ""),
    last_accessed_ms:
      typeof tab.lastAccessed === "number" ? Math.trunc(tab.lastAccessed) : null,
    pinned: Boolean(tab.pinned),
    audible: Boolean(tab.audible),
    discarded: Boolean(tab.discarded),
    incognito: Boolean(tab.incognito),
  };
}

function schedule() {
  if (sendTimer) {
    clearTimeout(sendTimer);
  }
  sendTimer = setTimeout(() => {
    sendTimer = 0;
    void sendTabs();
  }, 1500);
}

async function sendTabs() {
  const native = connect();
  if (!native) {
    return;
  }
  const tabs = await chrome.tabs.query({});
  const active = await chrome.tabs.query({
    active: true,
    lastFocusedWindow: true,
  });
  const activeId =
    active[0] && typeof active[0].id === "number" ? active[0].id : null;
  try {
    native.postMessage({
      type: "tabs",
      tabs: tabs.filter((tab) => typeof tab.id === "number").map(snapshot),
      active_tab_id: activeId,
      instance_id: INSTANCE,
      epoch,
    });
  } catch (_error) {
    port = null;
  }
}

chrome.runtime.onInstalled.addListener(schedule);
chrome.runtime.onStartup.addListener(schedule);
chrome.tabs.onUpdated.addListener(schedule);
chrome.tabs.onRemoved.addListener(schedule);
chrome.tabs.onActivated.addListener(schedule);
chrome.alarms.create("sweeploom-tabs", { periodInMinutes: 1 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "sweeploom-tabs") {
    void sendTabs();
  }
});
void sendTabs();

function isProtected(tab) {
  return Boolean(
    !tab || tab.pinned || tab.audible || tab.incognito || tab.active
  );
}

async function applyActions(actions) {
  for (const item of actions) {
    try {
      await applyOne(item);
    } catch (_error) {
      /* keep the rest of the batch */
    }
  }
}

async function applyOne(item) {
  if (!item || typeof item.tab_id !== "number") {
    return;
  }
  if (item.instance_id && item.instance_id !== INSTANCE) {
    return;
  }
  if (item.epoch && item.epoch !== epoch) {
    return;
  }
  if (item.action === "discard") {
    await discardTab(item);
  } else if (item.action === "focus") {
    await focusTab(item.tab_id);
  } else if (item.action === "bookmark_and_close") {
    await bookmarkAndClose(item);
  }
}

async function freshTab(tabId) {
  return chrome.tabs.get(tabId);
}

async function discardTab(item) {
  const tab = await freshTab(item.tab_id);
  if (isProtected(tab) || tab.discarded) {
    return;
  }
  if (item.expected_url && safeUrl(tab.url || "") !== item.expected_url) {
    return;
  }
  await chrome.tabs.discard(item.tab_id);
}

async function focusTab(tabId) {
  let tab;
  try {
    tab = await chrome.tabs.get(tabId);
    await chrome.tabs.update(tabId, { active: true });
  } catch (_error) {
    return;
  }
  if (tab && typeof tab.windowId === "number") {
    try {
      await chrome.windows.update(tab.windowId, { focused: true });
    } catch (_error) {
      /* window gone */
    }
  }
}

async function bookmarkAndClose(item) {
  const tabId = item.tab_id;
  let tab = await freshTab(tabId);
  if (isProtected(tab)) {
    return;
  }
  const url = tab.url || "";
  if (
    !url ||
    url.startsWith("chrome:") ||
    url.startsWith("about:") ||
    url.startsWith("moz-extension:")
  ) {
    return;
  }
  const approved = url;
  const created = await chrome.bookmarks.create({
    title: tab.title || url,
    url,
  });
  if (!created || !created.id) {
    return;
  }
  tab = await freshTab(tabId);
  if (isProtected(tab) || (tab.url || "") !== approved) {
    return;
  }
  await chrome.tabs.remove(tabId);
}
