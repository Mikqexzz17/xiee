
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Webview } from "@tauri-apps/api/webview";
import { listen } from "@tauri-apps/api/event";

const mainWindow = getCurrentWindow();

// Global State
let tabs = [];
let activeTabId = null;
let tabCounter = 0;
let loadInterval = null;
let isDarkMode = false;
let bookmarks = [];
let isShieldActive = true;

const urlInput = document.getElementById("url-input");
const progressBar = document.getElementById("progress-bar");
const tabsContainer = document.getElementById("tabs-container");
const bookmarksBar = document.getElementById("bookmarks-bar");
const shieldBtn = document.getElementById("shield-btn");
const headerHeight = 97;

// --- SECURITY HELPERS ---
const blockedKeywords = ["porn", "xxx", "sex", "casino", "betting", "gambling", "malware", "phishing", "hack", "viagra", "escort"];

function isSafeUrl(url) {
    if (!isShieldActive) return true;
    const lowerUrl = url.toLowerCase();
    for (let word of blockedKeywords) {
        const regex = new RegExp(`\\b${word}\\b`, "i");
        if (regex.test(lowerUrl)) return false;
    }
    return true;
}

shieldBtn.addEventListener("click", () => {
    isShieldActive = !isShieldActive;
    if (isShieldActive) {
        shieldBtn.style.color = "green";
        shieldBtn.title = "Tarcza Ochronna (WĹÄ„CZONA)";
    } else {
        shieldBtn.style.color = "red";
        shieldBtn.title = "Tarcza Ochronna (WYĹÄ„CZONA)";
    }
});

// --- UI HELPERS ---
function startProgress() {
    progressBar.style.display = "block";
    progressBar.style.width = "10%";
    let width = 10;
    if (loadInterval) clearInterval(loadInterval);
    loadInterval = setInterval(() => {
        if (width >= 85) clearInterval(loadInterval);
        else { width += Math.random() * 5; progressBar.style.width = width + "%"; }
    }, 200);
}

function stopProgress() {
    if (loadInterval) clearInterval(loadInterval);
    progressBar.style.width = "100%";
    setTimeout(() => {
        progressBar.style.display = "none";
        progressBar.style.width = "0%";
    }, 300);
}

function parseUrl(input) {
    if (input.startsWith("http://")) {
        return input.replace("http://", "https://");
    }
    if (input.startsWith("https://")) return input;
    
    if (input.includes(".") && !input.includes(" ")) return "https://" + input;
    
    let engine = document.getElementById("search-engine").value;
    if (engine === "duckduckgo") return "https://duckduckgo.com/?q=" + encodeURIComponent(input);
    return "https://www.google.com/search?q=" + encodeURIComponent(input);
}

// --- BOOKMARKS ---
function loadBookmarks() {
    try { const stored = localStorage.getItem("liteBrowserBookmarks"); if (stored) bookmarks = JSON.parse(stored); } catch(e) {}
    renderBookmarks();
}
function saveBookmarks() {
    try { localStorage.setItem("liteBrowserBookmarks", JSON.stringify(bookmarks)); } catch(e) {}
}
function renderBookmarks() {
    bookmarksBar.innerHTML = "";
    bookmarks.forEach((bm, index) => {
        const btn = document.createElement("button");
        btn.className = "bookmark-btn";
        btn.innerText = bm.title || bm.url;
        btn.onclick = () => navigateTo(bm.url);
        btn.oncontextmenu = (e) => {
            e.preventDefault();
            if (confirm(`Czy usunÄ…Ä‡ zakĹ‚adkÄ™: ${bm.title}?`)) { bookmarks.splice(index, 1); saveBookmarks(); renderBookmarks(); }
        };
        bookmarksBar.appendChild(btn);
    });
}
document.getElementById("add-bookmark-btn").addEventListener("click", () => {
    const t = getActiveTab();
    if (t && t.url && !bookmarks.some(b => b.url === t.url)) {
        bookmarks.push({ title: t.titleEl.innerText, url: t.url });
        saveBookmarks(); renderBookmarks();
    }
});

// --- IPC SETUP ---
// NasĹ‚uchiwanie ze stron na zmiany URL i intencje za pomocÄ… domyĹ›lnego nasĹ‚uchiwacza zdarzeĹ„
listen("webview-sync", (event) => {
    const data = event.payload;
    if (!data) return;
    
    const t = tabs.find(tab => tab.id === data.tabId);
    if (!t) return;
    
    if (data.type === "intent") {
        if(!isSafeUrl(data.url)) {
             alert("Tarcza zablokowaĹ‚a wejĹ›cie na niebezpieczny link ze strony.");
             return;
        }
        // JeĹĽeli url jest bezpieczny, pozwalamy na rÄ™cznÄ… nawigacjÄ™ 
        t.webview.eval(`window.location.href = decodeURIComponent("${encodeURIComponent(data.url)}");`).catch(()=>{});
        startProgress();
        setTimeout(stopProgress, 800);
    } else if (data.type === "update") {
        t.url = data.url;
        if (t.id === activeTabId && urlInput !== document.activeElement) {
             urlInput.value = data.url;
        }
        if (data.title) t.titleEl.innerText = data.title;
    }
});

// --- TABS LOGIC ---
const injectScripts = (id) => `
    // Adblock injection
    (function() {
        const adKeywords = ["ads", "analytics", "tracking", "banner", "doubleclick"];
        const originalFetch = window.fetch;
        window.fetch = async function(...args) {
            if(typeof args[0] === "string" && adKeywords.some(k => args[0].includes(k))) return new Response();
            return originalFetch.apply(this, args);
        };
    })();
    
    // Injected Security interceptor to block navigation to bad links BEFORE they load
    document.addEventListener("click", function(e) {
        const a = e.target.closest("a");
        if (a && a.href) {
            e.preventDefault(); // Zatrzymujemy domyĹ›lnÄ… nawigacjÄ™
            window.__TAURI__.event.emit("webview-sync", {
                type: "intent",
                tabId: ${id},
                url: a.href
            });
        }
    }, true);
    
    // Prosta synchronizacja stanu URL z interwaĹ‚em uĹĽywajÄ…c event.emit
    setInterval(() => {
        if(document.title !== window.__lastTitleSync || window.location.href !== window.__lastUrlSync) {
           window.__lastTitleSync = document.title;
           window.__lastUrlSync = window.location.href;
           window.__TAURI__.event.emit("webview-sync", {
                type: "update",
                tabId: ${id},
                url: window.location.href,
                title: document.title
            });
        }
    }, 1000);
`;

async function createTab(initialUrl = "https://duckduckgo.com") {
    tabCounter++;
    const id = tabCounter;
    
    const tabEl = document.createElement("div");
    tabEl.className = "tab";
    tabEl.id = `tab-${id}`;
    
    const titleEl = document.createElement("span");
    titleEl.className = "tab-title";
    titleEl.innerText = "Nowa karta";
    tabEl.appendChild(titleEl);
    
    const closeBtn = document.createElement("button");
    closeBtn.className = "close-tab-btn";
    closeBtn.innerText = "âś–";
    closeBtn.title = "Zamknij kartÄ™ (Ctrl+W)";
    closeBtn.onclick = (e) => { e.stopPropagation(); closeTab(id); };
    tabEl.appendChild(closeBtn);
    
    tabEl.onclick = () => switchTab(id);
    tabsContainer.appendChild(tabEl);
    
    const scaleFactor = await mainWindow.scaleFactor();
    
    let webview = new Webview(mainWindow, `webview-${id}`, {
        url: initialUrl,
        x: 0,
        y: Math.floor(headerHeight * scaleFactor),
        width: Math.floor(window.innerWidth * scaleFactor),
        height: Math.floor((window.innerHeight - headerHeight) * scaleFactor),
        incognito: true,
        initializationScripts: [injectScripts(id)]
    });
    
    const tabData = { id, webview, url: initialUrl, titleEl, isSleeping: false };
    tabs.push(tabData);
    await switchTab(id);
}

async function wakeUpTab(tabData) {
    const scaleFactor = await mainWindow.scaleFactor();
    tabData.webview = new Webview(mainWindow, `webview-${tabData.id}`, {
        url: tabData.url,
        x: 0,
        y: Math.floor(headerHeight * scaleFactor),
        width: Math.floor(window.innerWidth * scaleFactor),
        height: Math.floor((window.innerHeight - headerHeight) * scaleFactor),
        incognito: true,
        initializationScripts: [injectScripts(tabData.id)]
    });
    tabData.isSleeping = false;
    document.getElementById(`tab-${tabData.id}`).classList.remove("sleeping");
    if (isDarkMode) setTimeout(applyDarkModeToActive, 500);
}

async function switchTab(id) {
    if (activeTabId === id) return;
    if (activeTabId !== null) {
        const oldTab = tabs.find(t => t.id === activeTabId);
        if (oldTab && oldTab.webview && !oldTab.isSleeping) {
            document.getElementById(`tab-${activeTabId}`).classList.remove("active");
            oldTab.webview.hide().catch(()=>{});
        }
    }
    
    activeTabId = id;
    const newTab = tabs.find(t => t.id === activeTabId);
    if (newTab) {
        document.getElementById(`tab-${activeTabId}`).classList.add("active");
        if (newTab.isSleeping) await wakeUpTab(newTab);
        else newTab.webview.show().catch(()=>{});
        urlInput.value = newTab.url;
    }
}

async function closeTab(id) {
    const tabIndex = tabs.findIndex(t => t.id === id);
    if (tabIndex === -1) return;
    const tabData = tabs[tabIndex];
    if (tabData.webview && !tabData.isSleeping) await tabData.webview.close();
    document.getElementById(`tab-${id}`).remove();
    tabs.splice(tabIndex, 1);
    if (activeTabId === id) {
        activeTabId = null;
        if (tabs.length > 0) await switchTab(tabs[tabs.length - 1].id);
        else { urlInput.value = ""; createTab(); }
    }
}

function getActiveTab() { return tabs.find(t => t.id === activeTabId); }

document.getElementById("new-tab-btn").addEventListener("click", () => createTab());

document.getElementById("back-btn").addEventListener("click", () => {
    const t = getActiveTab();
    if(t && t.webview && !t.isSleeping) {
        startProgress(); t.webview.eval(`window.history.back();`).catch(() => {}); setTimeout(stopProgress, 600);
    }
});

document.getElementById("forward-btn").addEventListener("click", () => {
    const t = getActiveTab();
    if(t && t.webview && !t.isSleeping) {
        startProgress(); t.webview.eval(`window.history.forward();`).catch(() => {}); setTimeout(stopProgress, 600);
    }
});

document.getElementById("refresh-btn").addEventListener("click", () => {
    const t = getActiveTab();
    if(t && t.webview && !t.isSleeping) {
        startProgress(); t.webview.eval(`window.location.reload();`).catch(() => {}); setTimeout(stopProgress, 600);
    }
});

// --- NAVIGATION ---
async function navigateTo(input) {
    let url = parseUrl(input);
    
    if (!isSafeUrl(url)) {
        alert("OSTRZEĹ»ENIE! Wykryto zablokowane sĹ‚owo kluczowe w adresie. DostÄ™p zablokowany ze wzglÄ™dĂłw bezpieczeĹ„stwa.");
        return;
    }
    
    urlInput.value = url;
    let t = getActiveTab();
    if (!t) await createTab(url);
    else {
        startProgress();
        t.url = url;
        if(t.isSleeping) await wakeUpTab(t);
        // Safely pass the URL using encodeURIComponent and decodeURIComponent in eval
        t.webview.eval(`window.location.href = decodeURIComponent("${encodeURIComponent(url)}");`).catch(()=>{});
        setTimeout(stopProgress, 800);
    }
}

document.getElementById("go-btn").addEventListener("click", () => { let i = urlInput.value; if(i) navigateTo(i); });
urlInput.addEventListener("keypress", (e) => { if(e.key === "Enter") { let i = urlInput.value; if(i) navigateTo(i); }});
urlInput.addEventListener("click", function() { this.select(); });

// --- MEMORY SAVER ---
document.getElementById("sleep-tabs-btn").addEventListener("click", async () => {
    let freedCount = 0;
    for (let tab of tabs) {
        if (tab.id !== activeTabId && !tab.isSleeping && tab.webview) {
            await tab.webview.close();
            tab.webview = null;
            tab.isSleeping = true;
            document.getElementById(`tab-${tab.id}`).classList.add("sleeping");
            freedCount++;
        }
    }
    if (freedCount > 0) alert(`UĹ›piono ${freedCount} kart w tle. PamiÄ™Ä‡ RAM zostaĹ‚a zwolniona!`);
    else alert("Brak kart w tle do uĹ›pienia.");
});

// --- READER MODE ---
document.getElementById("reader-mode-btn").addEventListener("click", () => {
    const t = getActiveTab();
    if(t && t.webview && !t.isSleeping) {
        startProgress();
        const rs = `(() => {
            const art = document.querySelector("article") || document.body;
            const content = Array.from(art.querySelectorAll("h1, h2, h3, h4, p")).map(e => e.outerHTML).join("");
            document.body.innerHTML = "<div style=\"max-width: 800px; margin: 40px auto; font-family: Georgia, serif; font-size: 18px; line-height: 1.6; padding: 20px;\">" + content + "</div>";
            document.head.innerHTML = "";
        })();`;
        t.webview.eval(rs).catch(()=>{});
        setTimeout(stopProgress, 800);
    }
});

// --- DARK MODE ---
function applyDarkModeToActive() {
    const t = getActiveTab();
    if(t && t.webview && !t.isSleeping) {
        const darkCss = isDarkMode ? 
            `document.documentElement.style.filter = "invert(1) hue-rotate(180deg)";
             document.documentElement.style.background = "#000";` : 
            `document.documentElement.style.filter = "none";
             document.documentElement.style.background = "";`;
        t.webview.eval(darkCss).catch(()=>{});
    }
}
document.getElementById("dark-mode-btn").addEventListener("click", () => {
    isDarkMode = !isDarkMode;
    if (isDarkMode) {
        document.body.style.background = "#222";
        document.getElementById("toolbar").style.background = "#333";
        document.getElementById("toolbar").style.borderColor = "#111";
        document.getElementById("tabs-bar").style.background = "#1a1a1a";
        document.getElementById("bookmarks-bar").style.background = "#2a2a2a";
        document.querySelectorAll(".btn").forEach(b => b.style.color = "#ddd");
        document.getElementById("url-input").style.background = "#444";
        document.getElementById("url-input").style.color = "#fff";
        document.getElementById("search-engine").style.background = "#444";
        document.getElementById("search-engine").style.color = "#fff";
    } else {
        document.body.style.background = "#e0e0e0";
        document.getElementById("toolbar").style.background = "#f8f9fa";
        document.getElementById("toolbar").style.borderColor = "#caced1";
        document.getElementById("tabs-bar").style.background = "#d4d4d4";
        document.getElementById("bookmarks-bar").style.background = "#f1f3f4";
        document.querySelectorAll(".btn").forEach(b => b.style.color = "#333");
        document.getElementById("url-input").style.background = "#fff";
        document.getElementById("url-input").style.color = "#000";
        document.getElementById("search-engine").style.background = "#fff";
        document.getElementById("search-engine").style.color = "#000";
    }
    applyDarkModeToActive();
});

// --- SHORTCUTS ---
window.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.key.toLowerCase() === "t") { e.preventDefault(); createTab(); }
    if (e.ctrlKey if (e.ctrlKey && e.key.toLowerCase() === "w") { e.preventDefault(); if (activeTabId !== null) closeTab(activeTabId); }if (e.ctrlKey && e.key.toLowerCase() === "w") { e.preventDefault(); if (activeTabId !== null) closeTab(activeTabId); } e.key.toLowerCase() === "w") { e.preventDefault(); if (activeTabId !== null) closeTab(activeTabId); }
    if (e.ctrlKey if (e.ctrlKey && e.key.toLowerCase() === "w") { e.preventDefault(); if (activeTabId !== null) closeTab(activeTabId); }if (e.ctrlKey && e.key.toLowerCase() === "w") { e.preventDefault(); if (activeTabId !== null) closeTab(activeTabId); } e.key === "`") { e.preventDefault(); toggleTerminal(); }
});

// --- RESIZE ---
window.addEventListener("resize", async () => {
    try {
        const sf = await mainWindow.scaleFactor();
        for (let t of tabs) {
            if (t.webview && !t.isSleeping) {
                t.webview.setSize({
                    type: "Physical",
                    width: Math.floor(window.innerWidth * sf),
                    height: Math.floor((window.innerHeight - headerHeight) * sf)
                }).catch(()=>{});
            }
        }
    } catch(e){}
});

// INIT
const oldInit = window.onload;
window.onload = () => {
    if(oldInit) oldInit();
    loadBookmarks();
    createTab("https://duckduckgo.com");
};


// --- XIARR Terminal ---
const terminalContainer = document.getElementById("terminal-container");
const terminalOutput = document.getElementById("terminal-output");
const terminalInput = document.getElementById("terminal-input");
let isTerminalOpen = false;

function toggleTerminal() {
    isTerminalOpen = !isTerminalOpen;
    terminalContainer.style.display = isTerminalOpen ? "flex" : "none";
    if (isTerminalOpen) {
        terminalInput.focus();
    }
}

document.getElementById("terminal-btn").addEventListener("click", toggleTerminal);
document.getElementById("terminal-close-btn").addEventListener("click", () => {
    isTerminalOpen = false;
    terminalContainer.style.display = "none";
});

function printTerminal(text, isCommand = false) {
    const line = document.createElement("div");
    if (isCommand) {
        line.style.color = "#aaa";
        line.innerText = `xiarr> ${text}`;
    } else {
        line.innerText = text;
        // Basic parsing for help text to make commands cyan
        line.innerHTML = line.innerHTML.replace(/(\/\w+)/g, "<span style=\"color:#00ffff;\">$1</span>");
    }
    terminalOutput.appendChild(line);
    terminalOutput.scrollTop = terminalOutput.scrollHeight;
}

const commands = {
    "/help": {
        desc: "Pokazuje listÄ™ dostÄ™pnych komend",
        action: () => {
            let helpText = "DostÄ™pne komendy XIARR Terminal:\n";
            for (const [cmd, data] of Object.entries(commands)) {
                helpText += `  ${cmd.padEnd(15)} - ${data.desc}\n`;
            }
            printTerminal(helpText);
        }
    },
    "/clear": {
        desc: "CzyĹ›ci ekran terminala",
        action: () => {
            terminalOutput.innerHTML = "";
            printTerminal("Witamy w XIARR Terminal! Wpisz /help, aby zobaczyÄ‡ dostÄ™pne komendy.");
        }
    },
    "/new": {
        desc: "Otwiera nowÄ…, pustÄ… kartÄ™ (/new [url])",
        action: (args) => {
            const url = args.length > 0 ? parseUrl(args.join(" ")) : "https://duckduckgo.com";
            createTab(url);
            printTerminal(`Otwieranie nowej karty: ${url}`);
        }
    },
    "/close": {
        desc: "Zamyka aktywnÄ… kartÄ™",
        action: () => {
            if (activeTabId !== null) {
                closeTab(activeTabId);
                printTerminal("ZamkniÄ™to aktywnÄ… kartÄ™.");
            } else {
                printTerminal("Brak aktywnej karty do zamkniÄ™cia.", false);
            }
        }
    },
    "/sleep": {
        desc: "Usypia wszystkie karty w tle uwalniajÄ…c RAM",
        action: () => {
            document.getElementById("sleep-tabs-btn").click();
            printTerminal("Uruchomiono optymalizacjÄ™ pamiÄ™ci.");
        }
    },
    "/dark": {
        desc: "PrzeĹ‚Ä…cza globalny tryb ciemny przeglÄ…darki",
        action: () => {
            document.getElementById("dark-mode-btn").click();
            printTerminal(`Tryb ciemny: ${isDarkMode ? "WĹÄ„CZONY" : "WYĹÄ„CZONY"}`);
        }
    },
    "/reader": {
        desc: "WĹ‚Ä…cza tryb czytnika na aktywnej stronie",
        action: () => {
            document.getElementById("reader-mode-btn").click();
            printTerminal("Tryb czytnika zostaĹ‚ wĹ‚Ä…czony dla obecnej strony.");
        }
    },
    "/nav": {
        desc: "Przechodzi na podany adres w obecnej karcie (/nav [url])",
        action: (args) => {
            if (args.length === 0) {
                printTerminal("BĹ‚Ä…d: podaj adres, np. /nav xier.com");
                return;
            }
            const url = args.join(" ");
            navigateTo(url);
            printTerminal(`Nawigowanie do: ${url}`);
        }
    },
    "/bm": {
        desc: "Dodaje aktywnÄ… stronÄ™ do zakĹ‚adek",
        action: () => {
            document.getElementById("add-bookmark-btn").click();
            printTerminal("PrĂłba dodania do zakĹ‚adek...");
        }
    },
    "/killall": {
        desc: "Zamyka wszystkie karty oprĂłcz aktywnej (ekstremalne zwalnianie zasobĂłw)",
        action: async () => {
            const tabsToClose = tabs.filter(t => t.id !== activeTabId);
            let count = 0;
            for(let t of tabsToClose) {
                await closeTab(t.id);
                count++;
            }
            printTerminal(`Zabito ${count} kart(y).`);
        }
    },
    "/sysinfo": {
        desc: "Pokazuje wirtualne statystyki przeglÄ…darki",
        action: () => {
            const active = tabs.filter(t => !t.isSleeping).length;
            const sleeping = tabs.filter(t => t.isSleeping).length;
            const total = tabs.length;
            printTerminal(`Statystyki Xier:\n  Karty ogĂłĹ‚em: ${total}\n  Karty aktywne: ${active}\n  Karty uĹ›pione: ${sleeping}\n  Tryb ciemny: ${isDarkMode}\n  Tarcza ochronna: ${isShieldActive}`);
        }
    }
};

terminalInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
        const input = terminalInput.value.trim();
        if (!input) return;
        
        printTerminal(input, true);
        terminalInput.value = "";
        
        if (input.startsWith("/")) {
            const parts = input.split(" ");
            const cmd = parts[0];
            const args = parts.slice(1);
            
            if (commands[cmd]) {
                commands[cmd].action(args);
            } else {
                printTerminal(`Nieznana komenda: ${cmd}. Wpisz /help, aby zobaczyÄ‡ listÄ™ komend.`);
            }
        } else {
            // JeĹ›li to nie komenda to np. szukamy tego w google/duckduckgo
            navigateTo(input);
            printTerminal(`Wyszukiwanie: ${input}`);
        }
    }
});

// ============================================================
// XIARR — Pobieranie plikow
// ============================================================

// Wykryj linki do pobierania (klikniecia na linki z atrybutem download
// lub rozszerzenia plikow binarnych)
const DOWNLOAD_EXTENSIONS = [
  'zip','tar','gz','bz2','xz','7z','rar',
  'exe','deb','rpm','AppImage','iso',
  'pdf','mp3','mp4','mkv','avi','mov',
  'png','jpg','jpeg','gif','webp',
  'apk','dmg'
];

function isDownloadUrl(url) {
  try {
    const path = new URL(url).pathname.toLowerCase();
    return DOWNLOAD_EXTENSIONS.some(ext => path.endsWith('.' + ext));
  } catch { return false; }
}

function getFilenameFromUrl(url) {
  try {
    const path = new URL(url).pathname;
    const parts = path.split('/');
    const name = parts[parts.length - 1];
    return name || 'plik';
  } catch { return 'plik'; }
}

function showDownloadDialog(url, filename) {
  const dialog = document.getElementById('xiarr-download-dialog');
  document.getElementById('dl-filename').textContent = filename;
  document.getElementById('dl-url').textContent = url.length > 60 ? url.slice(0, 60) + '...' : url;
  dialog.style.display = 'flex';

  document.getElementById('dl-accept').onclick = async () => {
    dialog.style.display = 'none';
    document.getElementById('dl-progress').style.display = 'block';
    document.getElementById('dl-progress-name').textContent = filename;

    try {
      const { invoke } = window.__TAURI__.core;
      const result = await invoke('accept_download', { url, filename });
      document.getElementById('dl-progress').style.display = 'none';
      showDownloadSuccess(filename);
    } catch (err) {
      document.getElementById('dl-progress').style.display = 'none';
      alert('Blad pobierania: ' + err);
    }
  };

  document.getElementById('dl-cancel').onclick = () => {
    dialog.style.display = 'none';
  };
}

function showDownloadSuccess(filename) {
  const notif = document.getElementById('dl-success');
  document.getElementById('dl-success-name').textContent = filename;
  notif.style.display = 'block';
  setTimeout(() => { notif.style.display = 'none'; }, 3500);
}

// Podpnij sie pod klikniecia w zakladkach webview
// WebView wychwytuje eventy przez postMessage z iframem
// Dla linkow w glownym oknie monitorujemy klikniecia
document.addEventListener('click', (e) => {
  const a = e.target.closest('a');
  if (!a) return;
  const href = a.href;
  if (!href) return;

  const hasDownloadAttr = a.hasAttribute('download');
  if (hasDownloadAttr || isDownloadUrl(href)) {
    e.preventDefault();
    e.stopPropagation();
    const filename = a.getAttribute('download') || getFilenameFromUrl(href);
    showDownloadDialog(href, filename);
  }
}, true);
