/*
 * Copyright (C) 2024-2025 Zexshia
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import BannerDarkZenith from "/webui.bannerdarkmode.avif";
import BannerLightZenith from "/webui.bannerlightmode.avif";
import AvatarZenith from "/webui.avatar.avif";
import SchemeBanner from "/webui.schemebanner.avif";
import ResoBanner from "/webui.reso.avif";
import { exec, toast } from "kernelsu";
const moduleInterface = window.$azenith;
const fileInterface = window.$FILE;
const RESO_VALS = "/sdcard/AZenith/config/value/resosettings";
const AXERONBINPATH = "/data/data/com.android.shell/AxManager/plugins/AetherZenith/system/bin"
const MODULEPATH = "/data/data/com.android.shell/AxManager/plugins/AetherZenith"


const executeCommand = async (cmd, cwd = null) => {
  try {
    const { errno, stdout, stderr } = await exec(cmd, cwd ? { cwd } : {});
    return { errno, stdout, stderr };
  } catch (e) {
    return { errno: -1, stdout: "", stderr: e.message || String(e) };
  }
};

window.executeCommand = executeCommand;

const showRandomMessage = () => {
  const c = document.getElementById("msg");
  if (!c) return;

  // Make sure currentTranslations exists
  if (!window.getTranslation) return;

  // Pick a random key from 0–29
  const randomIndex = Math.floor(Math.random() * 30);
  const message = window.getTranslation(`randomMessages.${randomIndex}`);

  if (message) {
    c.textContent = message;
  } else {
    c.textContent = ""; // fallback if translation not loaded
  }
};

let lastProfile = { time: 0, value: "" };
const checkProfile = async () => {
  const now = Date.now();
  if (now - lastProfile.time < 5000) return;

  try {
    const { errno: c, stdout: s } = await executeCommand(
      "cat /sdcard/AZenith/config/API/current_profile"
    );

    if (c !== 0) return;
    const r = s.trim();
    const d = document.getElementById("CurProfile");
    if (!d) return;

    let l =
      { 0: "Initializing...", 1: "Performance", 2: "Balanced", 3: "ECO Mode" }[
        r
      ] || "Unknown";

    // Check for Lite mode
    const { errno: c2, stdout: s2 } = await executeCommand(
      "cat /sdcard/AZenith/config/value/cpulimit"
    );
    if (c2 === 0 && s2.trim() === "1") l += " (Lite)";

    if (lastProfile.value === l) return;
    lastProfile = { time: now, value: l };

    d.textContent = l;

    // Detect theme mode
    const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;

    // Dark mode colors (original) vs light mode (darker/saturated)
    const colors = isDark
      ? {
          Performance: "#ef4444",
          Balanced: "#7dd3fc",
          "ECO Mode": "#5eead4",
          "Initializing...": "#60a5fa",
          Default: "#ffffff",
        }
      : {
          Performance: "#b91c1c",
          Balanced: "#0284c7",
          "ECO Mode": "#0d9488",
          "Initializing...": "#2563eb",
          Default: "#1f2937",
        };

    const key = l.replace(" (Lite)", "");
    d.style.color = colors[key] || colors.Default;
  } catch (m) {
    console.error("checkProfile error:", m);
  }
};

let cachedSOCData = null;
const fetchSOCDatabase = async () => {
  if (!cachedSOCData) {
    try {
      cachedSOCData = await (await fetch(`${MODULEPATH}/webroot/webui.soclist.json`)).json();
    } catch {
      cachedSOCData = {};
    }
  }
  return cachedSOCData;
};

const getSoCModel = async () => {
  const props = [
    "ro.soc.model",
    "ro.hardware.chipname",
    "ro.board.platform",
    "ro.product.board",
    "ro.chipname",
    "ro.mediatek.platform",
  ];

  for (const prop of props) {
    const { errno, stdout } = await executeCommand(`getprop ${prop}`);
    if (errno === 0 && stdout.trim()) {
      return stdout.trim();
    }
  }

  return "Unknown SoC";
};

const checkCPUInfo = async () => {
  const cached = localStorage.getItem("soc_info");
  try {
    const rawModel = await getSoCModel();
    const model = rawModel.replace(/\s+/g, "").toUpperCase();

    const db = await fetchSOCDatabase();
    let displayName = db[model];

    if (!displayName) {
      for (let i = model.length; i >= 6; i--) {
        const partial = model.substring(0, i);
        if (db[partial]) {
          displayName = db[partial];
          break;
        }
      }
    }

    if (!displayName) displayName = model;

    document.getElementById("cpuInfo").textContent = displayName;

    if (cached !== displayName) {
      localStorage.setItem("soc_info", displayName);
    }
  } catch {
    document.getElementById("cpuInfo").textContent = cached || "Error";
  }
};

const checkKernelVersion = async () => {
  let cachedKernel = localStorage.getItem("kernel_version");

  try {
    const { errno, stdout } = await executeCommand("uname -r");
    const el = document.getElementById("kernelInfo");

    if (errno === 0 && stdout.trim()) {
      const version = stdout.trim();
      el.textContent = version;

      if (cachedKernel !== version) {
        localStorage.setItem("kernel_version", version);
      }
    } else {
      el.textContent = cachedKernel || "Unknown Kernel";
    }
  } catch {
    el.textContent = cachedKernel || "Error";
  }
};

const getAndroidVersion = async () => {
  let cachedVersion = localStorage.getItem("android_version");

  try {
    const { errno, stdout } = await executeCommand("getprop ro.build.version.release");
    const el = document.getElementById("android");

    if (errno === 0 && stdout.trim()) {
      const version = stdout.trim();
      el.textContent = version;

      if (cachedVersion !== version) {
        localStorage.setItem("android_version", version);
      }
    } else {
      el.textContent = cachedVersion || "Unknown Android";
    }
  } catch {
    el.textContent = cachedVersion || "Error";
  }
};

let lastServiceCheck = { time: 0, status: "", pid: "" };

const checkServiceStatus = async () => {
  const now = Date.now();
  if (now - lastServiceCheck.time < 1000) return; // 1s throttle
  lastServiceCheck.time = now;

  const r = document.getElementById("serviceStatus");
  const d = document.getElementById("servicePID");
  if (!r || !d) return;

  try {
    // Get PID immediately
    const { errno: pidErr, stdout: pidOut } = await executeCommand(
      "/system/bin/toybox pidof sys.aetherzenith-service"
    );

    let status = "";
    let pidText = getTranslation("serviceStatus.servicePID", "null");

    if (pidErr === 0 && pidOut.trim() !== "0") {
      const pid = pidOut.trim();
      pidText = getTranslation("serviceStatus.servicePID", pid);
      d.textContent = pidText; // show PID immediately

      // Fetch profile & AI in parallel without blocking PID display
      Promise.all([
        executeCommand("cat /sdcard/AZenith/config/API/current_profile"),
        executeCommand("cat /sdcard/AZenith/config/value/AIenabled")
      ]).then(([profileRawResult, aiRawResult]) => {
        const profile = profileRawResult.stdout?.trim() || "";
        const ai = aiRawResult.stdout?.trim() || "";

        // Determine status
        if (profile === "0") status = getTranslation("serviceStatus.initializing");
        else if (["1", "2", "3"].includes(profile)) {
          status =
            ai === "1"
              ? getTranslation("serviceStatus.runningAuto")
              : ai === "0"
              ? getTranslation("serviceStatus.runningIdle")
              : getTranslation("serviceStatus.unknownProfile");
        } else status = getTranslation("serviceStatus.unknownProfile");

        if (lastServiceCheck.status !== status) r.textContent = status;
        lastServiceCheck.status = status;
      });
    } else {
      status = getTranslation("serviceStatus.suspended");
      d.textContent = pidText; // show null PID
      if (lastServiceCheck.status !== status) r.textContent = status;
      lastServiceCheck.status = status;
    }

    lastServiceCheck.pid = pidText;
  } catch (e) {
    console.warn("checkServiceStatus error:", e);
  }
};

const checkLiteModeStatus = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/cpulimit"
  );
  0 === c && (document.getElementById("LiteMode").checked = "1" === s.trim());
};

const setLiteModeStatus = async (c) => {
  await executeCommand(
    c
      ? "echo 1 > /sdcard/AZenith/config/value/cpulimit"
      : "echo 0 > /sdcard/AZenith/config/value/cpulimit"
  );
};


const checkSFL = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/SFL"
  );
  0 === c && (document.getElementById("SFL").checked = "1" === s.trim());
};

const setSFL = async (c) => {
  await executeCommand(
    c
      ? "echo 1 > /sdcard/AZenith/config/value/SFL"
      : "echo 0 > /sdcard/AZenith/config/value/SFL"
  );
};

const checkiosched = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/iosched"
  );
  0 === c && (document.getElementById("iosched").checked = "1" === s.trim());
};

const setiosched = async (c) => {
  await executeCommand(
    c
      ? "echo 1 /sdcard/AZenith/config/value/iosched"
      : "echo 0 /sdcard/AZenith/config/value/iosched"
  );
};

const applyFSTRIM = async () => {
  await executeCommand(
    `${AXERONBINPATH}/sys.aetherzenith-conf FSTrim`
  );
  const fstrimToast = getTranslation("toast.fstrim");
  toast(fstrimToast);
};

const hideGameListModal = () => {
  let c = document.getElementById("gamelistModal");
  c.classList.remove("show");
  document.body.classList.remove("modal-open");
  c._resizeHandler &&
    (window.removeEventListener("resize", c._resizeHandler),
    delete c._resizeHandler);
};

let originalGamelist = "";

const showGameListModal = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/AIenabled"
  );
  if (0 === c && "0" === s.trim()) {
    const showCantAccessToast = getTranslation("toast.showcantaccess");
    toast(showCantAccessToast);
    return;
  }

  const r = document.getElementById("gamelistModal");
  const d = document.getElementById("gamelistInput");
  const searchInput = document.getElementById("gamelistSearch");
  const l = r.querySelector(".gamelist-content");

  const { errno: m, stdout: h } = await executeCommand(
    "cat /sdcard/AZenith/config/gamelist/gamelist.txt"
  );

  if (m === 0) {
    const formatted = h.trim().replace(/\|/g, "\n");
    originalGamelist = formatted;
    d.value = formatted;
  }

  if (searchInput) {
    searchInput.value = "";
    searchInput.removeEventListener("input", filterGameList);
    searchInput.addEventListener("input", filterGameList);
  }

  r.classList.add("show");
  document.body.classList.add("modal-open");
  setTimeout(() => d.focus(), 100);

  const g = window.innerHeight;
  const f = () => {
    window.innerHeight < g - 150
      ? (l.style.transform = "translateY(-10%) scale(1)")
      : (l.style.transform = "translateY(0) scale(1)");
  };

  window.addEventListener("resize", f, { passive: true });
  r._resizeHandler = f;
  f();
};

const filterGameList = () => {
  const searchTerm = document
    .getElementById("gamelistSearch")
    .value.toLowerCase();
  const gamelistInput = document.getElementById("gamelistInput");

  if (!searchTerm) {
    gamelistInput.value = originalGamelist;
    return;
  }

  const filteredList = originalGamelist
    .split("\n")
    .filter((line) => line.toLowerCase().includes(searchTerm))
    .join("\n");

  gamelistInput.value = filteredList;
};

const saveGameList = async () => {
  const gamelistInput = document.getElementById("gamelistInput");
  const searchInput = document.getElementById("gamelistSearch");
  const searchTerm = (searchInput?.value || "").toLowerCase();

  const editedLines = gamelistInput.value
    .split("\n")
    .map((x) => x.trim())
    .filter(Boolean);

  const originalLines = originalGamelist
    .split("\n")
    .map((x) => x.trim())
    .filter(Boolean);

  if (!searchTerm) {
    const outputString = editedLines.join("|").replace(/"/g, '\\"');
    await executeCommand(
      `echo "${outputString}" > /sdcard/AZenith/config/gamelist/gamelist.txt`
    );
    const savedPackagesToast = getTranslation("toast.savedPackages", editedLines.length);
    toast(savedPackagesToast);
    hideGameListModal();
    return;
  }

  let editedIndex = 0;
  const mergedLines = originalLines.map((line) => {
    if (line.toLowerCase().includes(searchTerm)) {
      const replacement = editedLines[editedIndex++]?.trim();
      return replacement || line;
    }
    return line;
  });

  const outputString = mergedLines.join("|").replace(/"/g, '\\"');
  await executeCommand(
    `echo "${outputString}" > /sdcard/AZenith/config/gamelist/gamelist.txt`
  );
  const savedPackagesToast = getTranslation("toast.savedPackages", mergedLines.length);
  toast(savedPackagesToast);
  hideGameListModal();
};
const checklogger = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/debugmode"
  );
  0 === c && (document.getElementById("logger").checked = "true" === s.trim());
};

const setlogger = async (c) => {
  await executeCommand(
    c
      ? "echo true > /sdcard/AZenith/config/value/debugmode"
      : "echo false > /sdcard/AZenith/config/value/debugmode"
  );
};

const startService = async () => {
  try {
    let { stdout: c } = await executeCommand(
      "cat /sdcard/AZenith/config/API/current_profile"
    );
    let s = c.trim();

    if (s === "0") {
      const cantRestartToast = getTranslation("toast.cantRestart");
      toast(cantRestartToast);
      return;
    }

    let { stdout: pid } = await executeCommand(
      "/system/bin/toybox pidof sys.aetherzenith-service"
    );
    if (!pid || pid.trim() === "") {
      const serviceDeadToast = getTranslation("toast.serviceDead");
      toast(serviceDeadToast);
      return;
    }

    const restartingDaemonToast = getTranslation("toast.restartingDaemon");
    toast(restartingDaemonToast);
    
    await executeCommand(
      "pkill -9 -f sys.aetherzenith.thermalcore"
    );
    await executeCommand(
      "pkill -9 -f sys.aetherzenith-service; su -c '${AXERONBINPATH}/sys.aetherzenith-service > /dev/null 2>&1 & disown'"
    );

    await checkServiceStatus();
  } catch (r) {
    const restartFailedToast = getTranslation("toast.restartFailed");
    toast(restartFailedToast);
    console.error("startService error:", r);
  }
};

const checkGPreload = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/APreload"
  );
  0 === c && (document.getElementById("GPreload").checked = "1" === s.trim());
};

const setGPreloadStatus = async (c) => {
  await executeCommand(
    c
      ? "echo 1 > /sdcard/AZenith/config/value/APreload"
      : "echo 0 > /sdcard/AZenith/config/value/APreload"
  );
};
const checkRamBoost = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/clearbg"
  );
  0 === c && (document.getElementById("clearbg").checked = "1" === s.trim());
};

const setRamBoostStatus = async (c) => {
  await executeCommand(
    c
      ? "echo 1 > /sdcard/AZenith/config/value/clearbg"
      : "echo 0 > /sdcard/AZenith/config/value/clearbg"
  );
};

const checkAI = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/AIenabled"
  );
  0 === c && (document.getElementById("disableai").checked = "0" === s.trim());
};

const setAI = async (c) => {
  await executeCommand(
    c
      ? "echo 0 > /sdcard/AZenith/config/value/AIenabled"
      : "echo 1 > /sdcard/AZenith/config/value/AIenabled"
  );
  await executeCommand(
    c
      ? "mv /sdcard/AZenith/config/gamelist/gamelist.txt /sdcard/AZenith/config/gamelist/gamelist.bin"
      : "mv /sdcard/AZenith/config/gamelist/gamelist.bin /sdcard/AZenith/config/gamelist/gamelist.txt"
  );
};

const applyperformanceprofile = async () => {
  let { stdout: c } = await executeCommand(
    "cat /sdcard/AZenith/config/API/current_profile"
  );
  if ("1" === c.trim()) {
    const alreadyPerformanceToast = getTranslation("toast.alreadyPerformance");
    toast(alreadyPerformanceToast);
    return;
  }
  executeCommand("su -c aetherzenith 1 >/dev/null 2>&1 &");
};

const applybalancedprofile = async () => {
  let { stdout: c } = await executeCommand(
    "cat /sdcard/AZenith/config/API/current_profile"
  );
  if ("2" === c.trim()) {
    const alreadyBalancedToast = getTranslation("toast.alreadyBalanced");
    toast(alreadyBalancedToast);
    return;
  }
  executeCommand("su -c aetherzenith 2 >/dev/null 2>&1 &");
};

const applyecomode = async () => {
  let { stdout: c } = await executeCommand(
    "cat /sdcard/AZenith/config/API/current_profile"
  );
  if ("3" === c.trim()) {
    const alreadyECOToast = getTranslation("toast.alreadyECO");
    toast(alreadyECOToast);
    return;
  }
  executeCommand("su -c aetherzenith 3 >/dev/null 2>&1 &");
};

const checkjit = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/justintime"
  );
  0 === c && (document.getElementById("jit").checked = "1" === s.trim());
};

const setjit = async (c) => {
  await executeCommand(
    c
      ? "echo 1 > /sdcard/AZenith/config/value/justintime"
      : "echo 0 > /sdcard/AZenith/config/value/justintime"
  );
};

const checktoast = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/showtoast"
  );
  0 === c && (document.getElementById("toast").checked = "1" === s.trim());
};

const settoast = async (c) => {
  await executeCommand(
    c
      ? "echo 1 > /sdcard/AZenith/config/value/showtoast"
      : "echo 0 > /sdcard/AZenith/config/value/showtoast"
  );
};

const showAdditionalSettings = async () => {
  const c = document.getElementById("additional-modal"),
    s = c.querySelector(".additional-container");
  document.body.classList.add("modal-open");
  c.classList.add("show");

  const r = window.innerHeight;
  const d = () => {
    s.style.transform =
      window.innerHeight < r - 150
        ? "translateY(-10%) scale(1)"
        : "translateY(0) scale(1)";
  };
  window.addEventListener("resize", d, { passive: true });
  c._resizeHandler = d;
  d();
};

const hideAdditionalSettings = () => {
  const c = document.getElementById("additional-modal");
  c.classList.remove("show");
  document.body.classList.remove("modal-open");
  if (c._resizeHandler) {
    window.removeEventListener("resize", c._resizeHandler);
    delete c._resizeHandler;
  }
};

const showPreferenceSettings = async () => {
  const c = document.getElementById("preference-modal"),
    s = c.querySelector(".preference-container");
  document.body.classList.add("modal-open");
  c.classList.add("show");

  const r = window.innerHeight;
  const d = () => {
    s.style.transform =
      window.innerHeight < r - 150
        ? "translateY(-10%) scale(1)"
        : "translateY(0) scale(1)";
  };
  window.addEventListener("resize", d, { passive: true });
  c._resizeHandler = d;
  d();
};

const hidePreferenceSettings = () => {
  const c = document.getElementById("preference-modal");
  c.classList.remove("show");
  document.body.classList.remove("modal-open");
  if (c._resizeHandler) {
    window.removeEventListener("resize", c._resizeHandler);
    delete c._resizeHandler;
  }
};

const hideGamelistSettings = () => {
  const c = document.getElementById("gamelistModal");
  c.classList.remove("show");
  document.body.classList.remove("modal-open");
  if (c._resizeHandler) {
    window.removeEventListener("resize", c._resizeHandler);
    delete c._resizeHandler;
  }
};
const checkDND = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/dnd"
  );
  0 === c && (document.getElementById("DoNoDis").checked = "1" === s.trim());
};

const setDND = async (c) => {
  await executeCommand(
    c
      ? "echo 1 > /sdcard/AZenith/config/value/dnd"
      : "echo 0 > /sdcard/AZenith/config/value/dnd"
  );
};
const savelog = async () => {
  try {
    await executeCommand(
      `${AXERONBINPATH}/sys.aetherzenith-conf saveLog`
    );
    const logSavedMsg = getTranslation("toast.logSaved");
    toast(logSavedMsg);
  } catch (e) {
    const logSaveFailedMsg = getTranslation("toast.logSaveFailed");
    toast(logSaveFailedMsg);
    console.error("saveLog error:", e);
  }
};

const debounce = (fn, delay = 200) => {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
};

const detectResolution = async () => {
  const { errno, stdout } = await executeCommand(
    `wm size | grep -oE "[0-9]+x[0-9]+" | head -n 1`
  );
  if (errno !== 0 || !stdout.trim()) {
    console.error("Failed to detect resolution");
    const unableDetectResMsg = getTranslation("toast.unableDetectResolution");
    toast(unableDetectResMsg);
    return;
  }

  const defaultRes = stdout.trim();
  const [width, height] = defaultRes.split("x").map(Number);
  if (!width || !height) return;

  const mediumRes = `${Math.round(width * 0.9)}x${Math.round(height * 0.9)}`;
  const lowRes = `${Math.round(width * 0.8)}x${Math.round(height * 0.8)}`;

  const resoSizes = document.querySelectorAll(".reso-size");
  if (resoSizes.length === 3) {
    resoSizes[0].textContent = lowRes;
    resoSizes[1].textContent = mediumRes;
    resoSizes[2].textContent = defaultRes;
  }

  window._reso = {
    default: defaultRes,
    medium: mediumRes,
    low: lowRes,
    selected: defaultRes,
  };

  const { stdout: saved } = await executeCommand(`cat ${RESO_VALS}`);
  const savedRes = saved.trim();

  if (savedRes) {
    const buttons = document.querySelectorAll(".reso-option");
    buttons.forEach((btn) => {
      const text = btn.querySelector(".reso-size")?.textContent;
      if (text === savedRes) btn.classList.add("active");
      else btn.classList.remove("active");
    });
    if (window._reso) window._reso.selected = savedRes;
  } else {
    document.querySelectorAll(".reso-option")[2]?.classList.add("active");
  }
};

const selectResolution = async (btn) => {
  document
    .querySelectorAll(".reso-option")
    .forEach((b) => b.classList.remove("active"));
  btn.classList.add("active");

  const selectedText = btn.querySelector(".reso-size")?.textContent;
  if (!selectedText) return;

  if (window._reso) window._reso.selected = selectedText;
};

const applyResolution = async () => {
  if (!window._reso || !window._reso.selected) {
    const noResolutionSelected = getTranslation("toast.noResolutionSelected");
    toast(noResolutionSelected);
    return;
  }

  const selected = window._reso.selected;
  const def = window._reso.default;

  if (selected === def) {
    await executeCommand(`echo ${selected} ${RESO_VALS}`);
    await executeCommand("wm size reset");
  } else {
    await executeCommand(`echo ${selected} ${RESO_VALS}`);
    await executeCommand(`wm size ${selected}`);
  }
};

const showCustomResolution = async () => {
  const c = document.getElementById("resomodal");
  if (!c) return; // exit if modal not found

  const s = c.querySelector(".reso-container");
  if (!s) return;

  document.body.classList.add("modal-open");
  c.classList.add("show");

  await detectResolution();

  const r = window.innerHeight;
  const d = () => {
    window.innerHeight < r - 150
      ? (s.style.transform = "translateY(-10%) scale(1)")
      : (s.style.transform = "translateY(0) scale(1)");
  };

  window.addEventListener("resize", d, { passive: true });
  c._resizeHandler = d;
  d();
};

const hideResoSettings = () => {
  const c = document.getElementById("resomodal");
  c.classList.remove("show");
  document.body.classList.remove("modal-open");
  if (c._resizeHandler) {
    window.removeEventListener("resize", c._resizeHandler);
    delete c._resizeHandler;
  }
};

const showSettings = async () => {
  const c = document.getElementById("settingsModal"),
    s = c.querySelector(".settings-container");
  document.body.classList.add("modal-open");
  c.classList.add("show");

  const r = window.innerHeight;
  const d = () => {
    s.style.transform =
      window.innerHeight < r - 150
        ? "translateY(-10%) scale(1)"
        : "translateY(0) scale(1)";
  };
  window.addEventListener("resize", d, { passive: true });
  c._resizeHandler = d;
  d();
};

const hideSettings = () => {
  const c = document.getElementById("settingsModal");
  c.classList.remove("show");
  document.body.classList.remove("modal-open");
  if (c._resizeHandler) {
    window.removeEventListener("resize", c._resizeHandler);
    delete c._resizeHandler;
  }
};

  const c = document.getElementById("disableai");
  const s = document.getElementById("profile-button");

  if (c && s) {
    c.addEventListener("change", function () {
      setAI(this.checked);
      s.style.display = this.checked ? "block" : "none";
      s.classList.toggle("show", this.checked);
    });

    executeCommand("cat /sdcard/AZenith/config/value/AIenabled").then(
      ({ stdout: r }) => {
        const d = r.trim() === "0";
        c.checked = d;
        s.style.display = d ? "block" : "none";
        s.classList.toggle("show", d);
      }
    );
  }

const showProfilerSettings = async () => {
  const c = document.getElementById("profilermodal"),
    s = c.querySelector(".profiler-container");
  document.body.classList.add("modal-open");
  c.classList.add("show");

  const r = window.innerHeight;
  const d = () => {
    s.style.transform =
      window.innerHeight < r - 150
        ? "translateY(-10%) scale(1)"
        : "translateY(0) scale(1)";
  };
  window.addEventListener("resize", d, { passive: true });
  c._resizeHandler = d;
  d();
};

const hideProfilerSettings = () => {
  const c = document.getElementById("profilermodal");
  c.classList.remove("show");
  document.body.classList.remove("modal-open");
  if (c._resizeHandler) {
    window.removeEventListener("resize", c._resizeHandler);
    delete c._resizeHandler;
  }
};

const checkthermalcore = async () => {
  let { errno: c, stdout: s } = await executeCommand(
    "cat /sdcard/AZenith/config/value/thermalcore"
  );
  0 === c && (document.getElementById("thermalcore").checked = "1" === s.trim());
};

const setthermalcore = async (c) => {
  await executeCommand(
    c
      ? "echo 1 > /sdcard/AZenith/config/value/thermalcore && ${AXERONBINPATH}/sys.aetherzenith-conf setthermalcore 1 &"
      : "echo 0 > /sdcard/AZenith/config/value/thermalcore && ${AXERONBINPATH}/sys.aetherzenith-conf setthermalcore 0 &"
  );
};

const setupUIListeners = () => {
  const banner = document.getElementById("Banner");
  const avatar = document.getElementById("Avatar");
  const scheme = document.getElementById("Scheme");
  const reso = document.getElementById("Reso");

  if (avatar) avatar.src = AvatarZenith;
  if (scheme) scheme.src = SchemeBanner;
  if (reso) reso.src = ResoBanner;

  const updateBannerByTheme = () => {
    const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (banner) banner.src = isDark ? BannerDarkZenith : BannerLightZenith;
  };

  updateBannerByTheme();  

  // Listen for system theme changes
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", updateBannerByTheme);

  // Button Clicks
  document
    .getElementById("startButton")
    ?.addEventListener("click", startService);
    document
    .getElementById("applyperformance")
    ?.addEventListener("click", applyperformanceprofile);
  document
    .getElementById("applybalanced")
    ?.addEventListener("click", applybalancedprofile);
  document
    .getElementById("applypowersave")
    ?.addEventListener("click", applyecomode);
  document.getElementById("savelogButton")?.addEventListener("click", savelog);
  document.getElementById("FSTrim")?.addEventListener("click", applyFSTRIM);

  // Toggle Switches
  document
    .getElementById("jit")
    ?.addEventListener("change", (e) => setjit(e.target.checked));
  document
    .getElementById("disableai")
    ?.addEventListener("change", (e) => setAI(e.target.checked));
  document
    .getElementById("toast")
    ?.addEventListener("change", (e) => settoast(e.target.checked));
  document
    .getElementById("GPreload")
    ?.addEventListener("change", (e) => setthermalcore(e.target.checked));
  document
    .getElementById("clearbg")
    ?.addEventListener("change", (e) => setRamBoostStatus(e.target.checked));
  document
    .getElementById("SFL")
    ?.addEventListener("change", (e) => setSFL(e.target.checked));
  document
    .getElementById("LiteMode")
    ?.addEventListener("change", (e) => setLiteModeStatus(e.target.checked));
  document
    .getElementById("thermalcore")
    ?.addEventListener("change", (e) => setthermalcore(e.target.checked));
  document
    .getElementById("logger")
    ?.addEventListener("change", (e) => setlogger(e.target.checked));
  document
    .getElementById("iosched")
    ?.addEventListener("change", (e) => setiosched(e.target.checked));
  document
    .getElementById("DoNoDis")
    ?.addEventListener("change", (e) => setDND(e.target.checked));

  // Open settings
  document
    .getElementById("settingsButton")
    ?.addEventListener("click", showSettings);
  document
    .getElementById("close-settings")
    ?.addEventListener("click", hideSettings);
  document.getElementById("disableai").addEventListener("change", function () {
  setAI(this.checked),
    document
      .getElementById("profile-button")
      .classList.toggle("show", this.checked);
  });
    
  // Profile Settings
  document
    .getElementById("profile-button")
    ?.addEventListener("click", showProfilerSettings);
  document
    .getElementById("close-profiler")
    ?.addEventListener("click", hideProfilerSettings);
    
  // Additional Settings
  document
    .getElementById("show-additional-settings")
    ?.addEventListener("click", showAdditionalSettings);
  document
    .getElementById("close-additional")
    ?.addEventListener("click", hideAdditionalSettings);

  // Preference Settings
  document
    .getElementById("show-preference-settings")
    ?.addEventListener("click", showPreferenceSettings);
  document
    .getElementById("close-preference")
    ?.addEventListener("click", hidePreferenceSettings);

  // Custom Resolution Settings
  document
    .getElementById("customreso")
    ?.addEventListener("click", showCustomResolution);
  document.getElementById("applyreso")?.addEventListener("click", async () => {
    await applyResolution();
    hideResoSettings();
  });
  document
    .getElementById("resetreso-btn")
    ?.addEventListener("click", hideResoSettings);
  document
    .getElementById("close-reso")
    ?.addEventListener("click", hideResoSettings);

  // Selectable resolutions
  document.querySelectorAll(".reso-option")?.forEach((btn) => {
    btn.addEventListener("click", () => selectResolution(btn));
  });

  // Gamelist modal buttons
  document
    .getElementById("editGamelistButton")
    ?.addEventListener("click", showGameListModal);
  document
    .getElementById("cancelButton")
    ?.addEventListener("click", hideGameListModal);
  document
    .getElementById("saveGamelistButton")
    ?.addEventListener("click", saveGameList);
  document
    .getElementById("close-gamelist")
    ?.addEventListener("click", hideGamelistSettings);  
};

let loopsActive = false;
let loopTimeout = null;
let heavyInitDone = false;
let cleaningInterval = null;
let heavyInitTimeouts = [];

const cancelAllTimeouts = () => {
  heavyInitTimeouts.forEach(clearTimeout);
  heavyInitTimeouts = [];
};

const schedule = (fn, delay = 0) => {
  const id = setTimeout(() => {
    try {
      fn();
    } finally {
      heavyInitTimeouts = heavyInitTimeouts.filter((t) => t !== id);
    }
  }, delay);
  heavyInitTimeouts.push(id);
};

const cleanMemory = () => {
  if (typeof globalThis.gc === "function") globalThis.gc();
};

const monitoredTasks = [
  { fn: checkServiceStatus, interval: 5000 },
  { fn: checkProfile, interval: 5000 },
  { fn: showRandomMessage, interval: 10000 },
];

const runMonitoredTasks = async () => {
  if (!loopsActive) return;
  const now = Date.now();
  if (!runMonitoredTasks.lastRun) runMonitoredTasks.lastRun = {};

  for (const task of monitoredTasks) {
    const last = runMonitoredTasks.lastRun[task.fn.name] || 0;
    if (now - last >= task.interval) {
      try {
        await task.fn();
      } catch (e) {
        console.warn(`Task ${task.fn.name} failed:`, e);
      }
      runMonitoredTasks.lastRun[task.fn.name] = Date.now();
    }
  }

  loopTimeout = setTimeout(runMonitoredTasks, 1000);
};

const startMonitoringLoops = () => {
  if (loopsActive) return;
  loopsActive = true;
  runMonitoredTasks();
};

const stopMonitoringLoops = () => {
  loopsActive = false;
  if (loopTimeout) clearTimeout(loopTimeout);
};

const observeVisibility = () => {
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      stopMonitoringLoops();
      cancelAllTimeouts();
      if (cleaningInterval) clearInterval(cleaningInterval);
    } else {
      startMonitoringLoops();
    }
  });
};

const heavyInit = async () => {
  if (heavyInitDone) return;
  heavyInitDone = true;

  cancelAllTimeouts();
  if (cleaningInterval) clearInterval(cleaningInterval);

  const loader = document.getElementById("loading-screen");
  if (loader) loader.classList.remove("hidden");
  document.body.classList.add("no-scroll");

  const stage1 = [checkProfile, checkServiceStatus, showRandomMessage];
  await Promise.all(stage1.map((fn) => fn()));

  const quickChecks = [    
    checkCPUInfo,
    checkKernelVersion,
    getAndroidVersion,
    checkLiteModeStatus,
  ];
  await Promise.all(quickChecks.map((fn) => fn()));

  const heavyAsync = [
    checkiosched,
    checkGPreload,
  ];
  await Promise.all(heavyAsync.map((fn) => fn()));

  const heavySequential = [
    checkAI,
    checkthermalcore,
    checkDND,
    checkjit,
    checktoast,
    checkSFL,
    checklogger,
    checkRamBoost,
    detectResolution,
  ];
  for (const fn of heavySequential) {
    await fn();
  }

  startMonitoringLoops();
  observeVisibility();

  if (loader) loader.classList.add("hidden");
  document.body.classList.remove("no-scroll");

  cleaningInterval = setInterval(cleanMemory, 15000);
};

// Event Listeners
setupUIListeners();
heavyInit();
checkCPUInfo();
checkKernelVersion();
getAndroidVersion();    

