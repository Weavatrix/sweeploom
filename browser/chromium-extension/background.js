"use strict";

const HOST = "com.sweeploom.companion";
const VERSION = "0.1.0";

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
  port.onDisconnect.addListener(() => {
    port = null;
  });
  port.postMessage({ type: "hello", version: VERSION });
  port.onMessage.addListener((message) => {
    if (message && message.type === "apply" && Array.isArray(message.actions)) {
      void applyActions(message.actions);
    }
  });
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
  native.postMessage({
    type: "tabs",
    tabs: tabs.filter((tab) => typeof tab.id === "number").map(snapshot),
    active_tab_id: activeId,
  });
}

chrome.runtime.onInstalled.addListener(schedule);
chrome.runtime.onStartup.addListener(schedule);
chrome.tabs.onUpdated.addListener(schedule);
chrome.tabs.onRemoved.addListener(schedule);
chrome.tabs.onActivated.addListener(schedule);
chrome.alarms.create("sweeploom-tabs", { periodInMinutes: 5 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "sweeploom-tabs") {
    void sendTabs();
  }
});
void sendTabs();

async function applyActions(actions) {
  for (const item of actions) {
    if (!item || typeof item.tab_id !== "number") {
      continue;
    }
    if (item.action === "discard") {
      try {
        await chrome.tabs.discard(item.tab_id);
      } catch (_error) {
        /* tab gone */
      }
    } else if (item.action === "bookmark_and_close") {
      await bookmarkAndClose(item.tab_id);
    }
  }
}

async function bookmarkAndClose(tabId) {
  let tab;
  try {
    tab = await chrome.tabs.get(tabId);
  } catch (_error) {
    return;
  }
  if (!tab || tab.pinned || tab.audible || tab.incognito) {
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
  const created = await chrome.bookmarks.create({
    title: tab.title || url,
    url,
  });
  if (created && created.id) {
    await chrome.tabs.remove(tabId);
  }
}
