<script lang="ts">
  import { lang, t, type Lang } from "../lib/i18n";
  import {
    PROVIDERS,
    addCustomProvider,
    autoScan,
    connectProvider,
    connectProviderKey,
    customProviders,
    defaultProviderId,
    disconnectProviderKey,
    providerState,
    realProviders,
    refreshProviders,
    routerkuBaseUrl,
    saveRouterkuSecure,
    saveElevenlabsKey,
    removeElevenlabsKey,
    settingsPrefs,
    setPref,
    profileStore,
    islandStatus,
    fetchSystemPermissions,
    openPrivacySettings,
    type SystemPermissions,
    fetchDeviceInfo,
    type DeviceInfo,
    agentStatus,
    updateInfo,
    checkForUpdate,
    restartApp,
    theme,
    toggleCustomProvider,
    toggleProvider,
    usingRealBackend,
  } from "../lib/store";
  import ProviderRow from "./ProviderRow.svelte";
  import ModelPicker from "./ModelPicker.svelte";
  import type { Theme } from "../lib/store";
  import { onMount } from "svelte";
  import appIcon from "../assets/app-icon.png";

  const themes: { id: Theme; labelKey: string }[] = [
    { id: "system", labelKey: "settings.theme.system" },
    { id: "dark", labelKey: "settings.theme.dark" },
    { id: "light", labelKey: "settings.theme.light" },
  ];

  const langs: { id: Lang | "system"; label: string }[] = [
    { id: "system", label: "System default" },
    { id: "id", label: "Indonesia" },
    { id: "en", label: "English" },
    { id: "zh", label: "中文" },
  ];

  // Logo colors (Settings > General, per reference): pastel palette; the
  // selected color paints the rail logo tile.
  const LOGO_COLORS = [
    "#D98E8E", "#D99A6C", "#D9B84F", "#7AC74F", "#6CA8D9",
    "#B18CFF", "#D98AB5", "#E5C4A8", "#9AA0A6", "#F2F2F2",
  ];

  /* ---------- Settings categories (two-column layout) ---------- */
  type CatId =
    | "general" | "notifications" | "voice" | "island" | "computer"
    | "profile" | "billing" | "mobile" | "updates" | "providers";

  const CATEGORIES: { id: CatId; labelKey: string; icon: string }[] = [
    { id: "general", labelKey: "set.cat.general", icon: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>' },
    { id: "notifications", labelKey: "set.cat.notifications", icon: '<path d="M18 8a6 6 0 1 0-12 0c0 7-3 8-3 8h18s-3-1-3-8"/><path d="M13.7 20a2 2 0 0 1-3.4 0"/>' },
    { id: "voice", labelKey: "set.cat.voice", icon: '<path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><path d="M12 19v4M8 23h8"/>' },
    { id: "island", labelKey: "set.cat.island", icon: '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M9 9h6"/>' },
    { id: "computer", labelKey: "set.cat.computer", icon: '<path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z"/><path d="M13 13l6 6"/>' },
    { id: "profile", labelKey: "set.cat.profile", icon: '<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>' },
    { id: "billing", labelKey: "set.cat.billing", icon: '<rect x="1" y="4" width="22" height="16" rx="2"/><path d="M1 10h22"/>' },
    { id: "mobile", labelKey: "set.cat.mobile", icon: '<rect x="5" y="2" width="14" height="20" rx="2"/><path d="M12 18h.01"/>' },
    { id: "updates", labelKey: "set.cat.updates", icon: '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 10l5 5 5-5M12 15V3"/>' },
    { id: "providers", labelKey: "set.cat.providers", icon: '<rect x="2" y="3" width="20" height="7" rx="2"/><rect x="2" y="14" width="20" height="7" rx="2"/><path d="M6 6.5h.01M6 17.5h.01"/>' },
  ];

  let providerCatalog = $state<Array<{
    id: string; name: string; local: boolean; requires_key: boolean;
    requires_base_url: boolean; key_env: string | null; base_url_env: string | null;
    default_base_url: string | null; suggested: string[];
    keys_url: string | null; key_set: boolean; base_url_set: boolean;
  }>>([]);
  let defaultModelSpec = $state("");

  async function loadCatalog() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      providerCatalog = await invoke("provider_catalog");
    } catch (e) { console.error(e); }
  }
  $effect(() => { if (category === "providers") loadCatalog(); });

  let category = $state<CatId>("general");
  let catQuery = $state("");
  const visibleCats = $derived(
    CATEGORIES.filter(
      (c) =>
        !catQuery.trim() ||
        $t(c.labelKey).toLowerCase().includes(catQuery.trim().toLowerCase()),
    ),
  );

  /* ---------- Voice: dictation models & ElevenLabs key ---------- */
  let elevenKey = $state("");
  let elevenBusy = $state(false);
  let downloading = $state<Record<string, boolean>>({});

  // Model files are fetched by the dictation engine; here we track the
  // download state per model (the engine hookup lands with real STT).
  function downloadModel(id: string) {
    if (downloading[id] || $settingsPrefs.dictationDownloads.includes(id)) return;
    downloading = { ...downloading, [id]: true };
    setTimeout(() => {
      downloading = { ...downloading, [id]: false };
      setPref("dictationDownloads", [...$settingsPrefs.dictationDownloads, id]);
    }, 1400);
  }

  async function saveElevenKey() {
    elevenBusy = true;
    try {
      if (await saveElevenlabsKey(elevenKey)) elevenKey = "";
    } finally {
      elevenBusy = false;
    }
  }

  /* ---------- Computer Use: real macOS permission status ---------- */
  let sysPerms = $state<SystemPermissions | null>(null);
  async function refreshPerms() {
    sysPerms = await fetchSystemPermissions();
  }
  $effect(() => {
    if (category === "computer") void refreshPerms();
  });

  /* ---------- Profile: photo, sign-out, device session ---------- */
  let photoInput = $state<HTMLInputElement | null>(null);
  let emailDraft = $state("");
  let signOutArm = $state(false);
  let deviceInfo = $state<DeviceInfo | null>(null);
  let lastActive = $state(Date.now());

  const initials = $derived.by(() => {
    const n = $profileStore.name.trim();
    if (n) {
      const parts = n.split(/\s+/);
      return (
        parts[0][0] + (parts.length > 1 ? parts[parts.length - 1][0] : "")
      ).toUpperCase();
    }
    const e = $profileStore.email.trim();
    return e ? e[0].toUpperCase() : "M";
  });

  // Pick a photo, downscale it to a 128px square JPEG data URL, store it.
  function onPhotoChosen(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const img = new Image();
      img.onload = () => {
        const size = 128;
        const canvas = document.createElement("canvas");
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;
        const scale = Math.max(size / img.width, size / img.height);
        const w = img.width * scale;
        const h = img.height * scale;
        ctx.drawImage(img, (size - w) / 2, (size - h) / 2, w, h);
        profileStore.update((p) => ({
          ...p,
          photo: canvas.toDataURL("image/jpeg", 0.85),
        }));
      };
      img.src = String(reader.result);
    };
    reader.readAsDataURL(file);
  }

  function removePhoto() {
    profileStore.update((p) => ({ ...p, photo: "" }));
  }

  function saveEmail() {
    const v = emailDraft.trim();
    if (!v) return;
    profileStore.update((p) => ({ ...p, email: v }));
    emailDraft = "";
  }

  // Two-step sign out: the first click arms, the second clears the local
  // identity (name, email, photo) from this device.
  function onSignOut() {
    if (!signOutArm) {
      signOutArm = true;
      setTimeout(() => (signOutArm = false), 3000);
      return;
    }
    signOutArm = false;
    profileStore.update((p) => ({ ...p, name: "", email: "", photo: "" }));
  }

  async function refreshSession() {
    lastActive = Date.now();
    deviceInfo = await fetchDeviceInfo();
  }
  $effect(() => {
    if (category === "profile") void refreshSession();
  });

  function fmtDate(ts: number): string {
    if (!ts) return "—";
    const loc = $lang === "zh" ? "zh-CN" : $lang === "en" ? "en-US" : "id-ID";
    return new Intl.DateTimeFormat(loc, {
      dateStyle: "medium",
      timeStyle: "medium",
    }).format(new Date(ts));
  }

  /* ---------- Billing: add-server chooser ---------- */

  /* ---------- Updates: check on open, install when idle ---------- */
  let showWhatsNew = $state(false);
  let updNote = $state("");
  $effect(() => {
    if (category === "updates") void checkForUpdate();
  });
  function installWhenIdle() {
    if (!$updateInfo?.update_available) {
      updNote = $t("upd.note.noupdate");
      return;
    }
    const anyWorking = Object.values($agentStatus).some((s) => s === "working");
    if (anyWorking) {
      updNote = $t("upd.note.busy");
      return;
    }
    void restartApp();
  }

  /* ---------- Mobile Connect: QR, one-time codes, permanent link ---------- */
  let mobilePlatform = $state<"ios" | "android">("ios");
  let copiedWhich = $state<string | null>(null);

  function randomToken(len: number): string {
    const chars = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789";
    const arr = new Uint32Array(len);
    crypto.getRandomValues(arr);
    return Array.from(arr, (n) => chars[n % chars.length]).join("");
  }

  async function makeQr(text: string): Promise<string> {
    if (!text) return "";
    try {
      const mod = await import("qrcode");
      const qr = (mod as { default?: typeof mod }).default ?? mod;
      return await qr.toDataURL(text, { width: 240, margin: 1 });
    } catch {
      return "";
    }
  }

  async function copyText(text: string, which: string) {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copiedWhich = which;
      setTimeout(() => (copiedWhich = null), 2000);
    } catch {
      /* clipboard unavailable */
    }
  }

  // The permanent link doubles as the install invite link (card 1).
  const permanentLink = $derived(
    $settingsPrefs.mobileLinkToken
      ? `minime://connect?token=${$settingsPrefs.mobileLinkToken}`
      : "",
  );
  let inviteQr = $state("");
  $effect(() => {
    if (category !== "mobile") return;
    if (!$settingsPrefs.mobileLinkToken) {
      setPref("mobileLinkToken", randomToken(24));
      return;
    }
    void makeQr(permanentLink).then((u) => (inviteQr = u));
  });

  function generatePermanent() {
    setPref("mobileLinkToken", randomToken(24));
  }
  function revokePermanent() {
    setPref("mobileLinkToken", "");
    inviteQr = "";
  }

  // One-time sign-in code: valid for two minutes, single use — generating
  // a new one invalidates the previous code.
  let oneTime = $state<{ code: string; expiresAt: number } | null>(null);
  let oneTimeQr = $state("");
  let oneTimeLeft = $state(0);
  const oneTimeLink = $derived(
    oneTime ? `minime://connect?code=${oneTime.code}` : "",
  );
  let oneTimeTimer: ReturnType<typeof setInterval> | undefined;
  function generateOneTime() {
    const code = String(Math.floor(100000 + Math.random() * 900000));
    oneTime = { code, expiresAt: Date.now() + 120_000 };
    oneTimeLeft = 120;
    void makeQr(`minime://connect?code=${code}`).then((u) => (oneTimeQr = u));
    if (oneTimeTimer) clearInterval(oneTimeTimer);
    oneTimeTimer = setInterval(() => {
      if (!oneTime) return;
      oneTimeLeft = Math.max(
        0,
        Math.round((oneTime.expiresAt - Date.now()) / 1000),
      );
      if (oneTimeLeft <= 0 && oneTimeTimer) clearInterval(oneTimeTimer);
    }, 500);
  }

  let scanning = $state(false);
  let showProvDialog = $state(false);
  let provName = $state("");

  // RouterKu credentials — key goes to OS Keychain (Tahap 2), never localStorage
  let rkKey = $state("");
  let rkUrl = $state("");
  let rkInit = $state(false);
  let rkSaved = $state(false);
  let rkBusy = $state(false);
  $effect(() => {
    if (!rkInit) {
      rkUrl = $routerkuBaseUrl;
      rkInit = true;
    }
  });
  async function saveRouterkuForm() {
    rkBusy = true;
    try {
      const ok = await saveRouterkuSecure(rkKey, rkUrl);
      rkSaved = ok;
      if (ok) rkKey = "";
    } finally {
      rkBusy = false;
    }
  }

  // Per-provider API key form (paid providers)
  let keyFormFor = $state<string | null>(null);
  let keyInput = $state("");
  let keyBusy = $state(false);
  let keySavedFor = $state<string | null>(null);
  function openKeyForm(id: string) {
    keyFormFor = id;
    keyInput = "";
    keySavedFor = null;
  }
  // Patria 22 Okt: local-first — pilih provider yang belum siap,
  // otomatis trigger download/setup + bukakan window koneksi.
  function selectProvider(p: { id: string; kind: string; local?: boolean }) {
    defaultProviderId.set(p.id);
    const st = $providerState[p.id] ?? { connected: false };
    const rs = realStatus(p.id);
    const connected = rs ? rs.state === "connected" : st.connected;
    if (connected) return;
    if (p.kind === "paid" && !p.local) {
      // Premium: bukakan dialog API key
      openKeyForm(p.id);
    } else if (p.local) {
      // Local: trigger koneksi otomatis
      connectProvider(p.id);
    }
  }
  async function submitKeyForm(id: string) {
    keyBusy = true;
    try {
      const ok = await connectProviderKey(id, keyInput);
      if (ok) {
        keySavedFor = id;
        keyFormFor = null;
        keyInput = "";
      }
    } finally {
      keyBusy = false;
    }
  }
  async function disconnectKey(id: string) {
    await disconnectProviderKey(id);
    keySavedFor = null;
  }

  /** Status nyata dari backend Rust bila tersedia, fallback ke mock lokal. */
  function realStatus(id: string) {
    return $realProviders?.find((p) => p.id === id) ?? null;
  }

  onMount(() => {
    refreshProviders();
    // Patria 21 Okt: jika user pilih provider premium dari panel agen,
    // langsung bukakan form bantu konek.
    try {
      const pending = localStorage.getItem("minime.pending-connect");
      if (pending) {
        localStorage.removeItem("minime.pending-connect");
        openKeyForm(pending);
      }
    } catch {}
  });
  let provEndpoint = $state("");
  let provId = $state("");
  let provApiKey = $state("");
  let discoveredModels = $state<string[]>([]);
  let selectedModels = $state<string[]>([]);
  let findingModels = $state(false);
  let modelsError = $state("");

  async function findModels() {
    const url = provEndpoint.trim();
    if (!url) return;
    findingModels = true;
    modelsError = "";
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const models = await invoke<string[]>("discover_models", {
        baseUrl: url,
        apiKey: provApiKey.trim() || null,
      });
      discoveredModels = models;
      selectedModels = [...models];
      if (models.length === 0) modelsError = "Tidak ada model ditemukan di endpoint ini.";
    } catch (e) {
      modelsError = String(e);
      discoveredModels = [];
      selectedModels = [];
    } finally {
      findingModels = false;
    }
  }

  function toggleModelSelect(mid: string, on: boolean) {
    selectedModels = on
      ? [...selectedModels, mid]
      : selectedModels.filter((m) => m !== mid);
  }

  const APP_VERSION = "0.1.0";

  function createProvider() {
    addCustomProvider(provId, provName, provEndpoint, provApiKey, selectedModels);
    provName = "";
    provEndpoint = "";
    showProvDialog = false;
  }

  function rescan() {
    scanning = true;
    setTimeout(() => (scanning = false), 1200);
  }
</script>

{#snippet switchRow(prefKey: string, labelKey: string, descKey: string)}
  <div class="srow">
    <div class="srow-meta">
      <span class="srow-label">{$t(labelKey)}</span>
      <span class="srow-desc">{$t(descKey)}</span>
    </div>
    <label class="switch">
      <input
        type="checkbox"
        checked={$settingsPrefs[prefKey as keyof typeof $settingsPrefs] as boolean}
        onchange={(e) => setPref(prefKey as never, (e.currentTarget as HTMLInputElement).checked as never)}
        aria-label={$t(labelKey)}
      />
      <span class="track" aria-hidden="true"></span>
    </label>
  </div>
{/snippet}

<div class="settings-layout">
  <aside class="set-side">
    <h2>{$t("settings.title")}</h2>
    <label class="set-search">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <circle cx="11" cy="11" r="7" /><path d="M21 21l-4.3-4.3" />
      </svg>
      <input type="text" bind:value={catQuery} placeholder={$t("set.search")} aria-label={$t("set.search")} />
    </label>
    <nav class="set-nav" aria-label={$t("settings.title")}>
      {#each visibleCats as c (c.id)}
        <button class="set-nav-item" class:active={category === c.id} onclick={() => (category = c.id)}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true">{@html c.icon}</svg>
          <span>{$t(c.labelKey)}</span>
        </button>
      {/each}
    </nav>
  </aside>

  <div class="set-main">
    {#if category === "general"}
      <h2>{$t("set.cat.general")}</h2>

      <h3 class="sec-title">{$t("set.sec.appearance")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.language")}</span>
            <span class="srow-desc">{$t("set.language.desc")}</span>
          </div>
          <select
            class="sselect"
            value={$settingsPrefs.langMode}
            onchange={(e) => {
              const v = (e.currentTarget as HTMLSelectElement).value;
              setPref("langMode", v as "system" | Lang);
              if (v === "system") {
                const nav = (navigator.language || "id").toLowerCase();
                lang.set(nav.startsWith("zh") ? "zh" : nav.startsWith("en") ? "en" : "id");
              } else {
                lang.set(v as Lang);
              }
            }}
            aria-label={$t("set.language")}
          >
            {#each langs as l (l.id)}
              <option value={l.id}>{l.label}</option>
            {/each}
          </select>
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.theme")}</span>
            <span class="srow-desc">{$t("set.theme.desc")}</span>
          </div>
          <select
            class="sselect"
            value={$theme}
            onchange={(e) => theme.set((e.currentTarget as HTMLSelectElement).value as Theme)}
            aria-label={$t("set.theme")}
          >
            {#each themes as th (th.id)}
              <option value={th.id}>{$t(th.labelKey)}</option>
            {/each}
          </select>
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.logocolor")}</span>
            <span class="srow-desc">{$t("set.logocolor.desc")}</span>
          </div>
          <div class="logo-colors">
            {#each LOGO_COLORS as color (color)}
              <button
                class="color-dot"
                class:active={$settingsPrefs.logoColor === color}
                style:background={color}
                onclick={() => setPref("logoColor", color)}
                aria-label={color}
              ></button>
            {/each}
          </div>
        </div>
      </div>

      <h3 class="sec-title">{$t("set.sec.conversations")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.send")}</span>
            <span class="srow-desc">{$t("set.send.desc")}</span>
          </div>
          <select
            class="sselect"
            value={$settingsPrefs.sendShortcut}
            onchange={(e) => setPref("sendShortcut", (e.currentTarget as HTMLSelectElement).value as "enter" | "ctrlEnter")}
            aria-label={$t("set.send")}
          >
            <option value="enter">{$t("set.send.enter")}</option>
            <option value="ctrlEnter">{$t("set.send.ctrlenter")}</option>
          </select>
        </div>
        {@render switchRow("steerAgents", "set.steer", "set.steer.desc")}
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.memories")}</span>
            <span class="srow-desc">{$t("set.memories.desc")}</span>
          </div>
          <select
            class="sselect"
            value={$settingsPrefs.memoriesPerAgent}
            onchange={(e) => setPref("memoriesPerAgent", Number((e.currentTarget as HTMLSelectElement).value))}
            aria-label={$t("set.memories")}
          >
            {#each [16, 32, 64, 128] as n (n)}
              <option value={n}>{n}</option>
            {/each}
          </select>
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.links")}</span>
            <span class="srow-desc">{$t("set.links.desc")}</span>
          </div>
          <select
            class="sselect"
            value={$settingsPrefs.externalLinks}
            onchange={(e) => setPref("externalLinks", (e.currentTarget as HTMLSelectElement).value as "browser" | "inapp")}
            aria-label={$t("set.links")}
          >
            <option value="browser">{$t("set.links.browser")}</option>
            <option value="inapp">{$t("set.links.inapp")}</option>
          </select>
        </div>
      </div>

      <h3 class="sec-title">{$t("set.sec.behavior")}</h3>
      <div class="card">
        {@render switchRow("launchAtLogin", "set.launch", "set.launch.desc")}
        {@render switchRow("fastConnect", "set.fast", "set.fast.desc")}
        {@render switchRow("keepBackground", "set.background", "set.background.desc")}
        {@render switchRow("restoreWorkspace", "set.restore", "set.restore.desc")}
      </div>

      <h3 class="sec-title">{$t("set.sec.permissions")}</h3>
      <div class="card">
        {@render switchRow("shareAnalytics", "set.analytics", "set.analytics.desc")}
      </div>
    {:else if category === "notifications"}
      <h2>{$t("set.cat.notifications")}</h2>
      <h3 class="sec-title">{$t("set.sec.alerts")}</h3>
      <div class="card">
        {@render switchRow("notifApproval", "set.notif.approval", "set.notif.approval.desc")}
        {@render switchRow("notifDone", "set.notif.done", "set.notif.done.desc")}
        {@render switchRow("notifSound", "set.notif.sound", "set.notif.sound.desc")}
      </div>
      <p class="lede">{$t("settings.note")}</p>
    {:else if category === "voice"}
      <h2>{$t("set.cat.voice")}</h2>

      <h3 class="sec-title">{$t("voice.sec.model")}</h3>
      <div class="card">
        <div class="model-row">
          <button class="model-pick" onclick={() => setPref("dictationModel", "parakeet")}>
            <span class="model-name">Parakeet TDT 0.6B v3</span>
            <span class="model-desc">{$t("voice.parakeet.desc")}</span>
            <span class="model-sub">{$t("voice.parakeet.size")}</span>
          </button>
          <div class="model-actions">
            {#if $settingsPrefs.dictationModel === "parakeet"}
              <span class="inuse">{$t("voice.inuse")}</span>
            {/if}
            <button
              class="btn-ghost"
              disabled={downloading.parakeet || $settingsPrefs.dictationDownloads.includes("parakeet")}
              onclick={() => downloadModel("parakeet")}
            >
              {#if $settingsPrefs.dictationDownloads.includes("parakeet")}
                {$t("voice.downloaded")}
              {:else if downloading.parakeet}
                {$t("voice.downloading")}
              {:else}
                ⬇ {$t("voice.download")}
              {/if}
            </button>
          </div>
        </div>

        <div class="model-row">
          <button class="model-pick" onclick={() => setPref("dictationModel", "whisper")}>
            <span class="model-name">Whisper large-v3 turbo</span>
            <span class="model-desc">{$t("voice.whisper.desc")}</span>
            <span class="model-sub">{$t("voice.whisper.size")}</span>
          </button>
          <div class="model-actions">
            {#if $settingsPrefs.dictationModel === "whisper"}
              <span class="inuse">{$t("voice.inuse")}</span>
            {/if}
            <button
              class="btn-ghost"
              disabled={downloading.whisper || $settingsPrefs.dictationDownloads.includes("whisper")}
              onclick={() => downloadModel("whisper")}
            >
              {#if $settingsPrefs.dictationDownloads.includes("whisper")}
                {$t("voice.downloaded")}
              {:else if downloading.whisper}
                {$t("voice.downloading")}
              {:else}
                ⬇ {$t("voice.download")}
              {/if}
            </button>
          </div>
        </div>

        <div class="model-row col">
          <button class="model-pick" onclick={() => setPref("dictationModel", "elevenlabs")}>
            <span class="model-name">ElevenLabs Scribe v2 Realtime</span>
            <span class="model-desc">{$t("voice.eleven.desc")}</span>
            <span class="model-sub">{$t("voice.eleven.needs")}</span>
          </button>
          {#if $settingsPrefs.dictationModel === "elevenlabs"}
            <span class="inuse">{$t("voice.inuse")}</span>
          {/if}
          <div class="key-form">
            {#if $settingsPrefs.elevenlabsKeySaved}
              <span class="key-saved">{$t("voice.key.saved")}</span>
              <button class="btn-ghost" onclick={() => removeElevenlabsKey()}>
                {$t("voice.key.remove")}
              </button>
            {:else}
              <input
                type="password"
                bind:value={elevenKey}
                placeholder={$t("voice.key.placeholder")}
                aria-label={$t("voice.key.placeholder")}
              />
              <button
                class="btn-ghost"
                disabled={!elevenKey.trim() || elevenBusy}
                onclick={saveElevenKey}
              >
                {$t("voice.key.save")}
              </button>
            {/if}
          </div>
        </div>
      </div>

      <h3 class="sec-title">{$t("voice.sec.lang")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("voice.lang")}</span>
            <span class="srow-desc">{$t("voice.lang.desc")}</span>
          </div>
          <select
            class="sselect"
            value={$settingsPrefs.dictationLang}
            onchange={(e) => setPref("dictationLang", (e.currentTarget as HTMLSelectElement).value)}
            aria-label={$t("voice.lang")}
          >
            <option value="auto">🌐 {$t("voice.lang.auto")}</option>
            <option value="id">Indonesia</option>
            <option value="en">English</option>
            <option value="zh">中文</option>
            <option value="ja">日本語</option>
            <option value="ko">한국어</option>
            <option value="ar">العربية</option>
          </select>
        </div>
      </div>

      <h3 class="sec-title">{$t("set.sec.voice")}</h3>
      <div class="card">
        {@render switchRow("voiceEnabled", "set.voice.enabled", "set.voice.enabled.desc")}
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.voice.lang")}</span>
            <span class="srow-desc">{$t("set.voice.lang.desc")}</span>
          </div>
          <select
            class="sselect"
            value={$settingsPrefs.voiceLang}
            onchange={(e) => setPref("voiceLang", (e.currentTarget as HTMLSelectElement).value)}
            aria-label={$t("set.voice.lang")}
          >
            <option value="id">Indonesia</option>
            <option value="en">English</option>
            <option value="zh">中文</option>
          </select>
        </div>
      </div>
    {:else if category === "island"}
      <h2>{$t("set.cat.island")}</h2>

      <h3 class="sec-title">{$t("island.sec.behavior")}</h3>
      <div class="card">
        {@render switchRow("islandNotch", "island.notch", "island.notch.desc")}
        {@render switchRow("islandIdle", "island.idle", "island.idle.desc")}
        {@render switchRow("islandExtra", "island.extra", "island.extra.desc")}
        {@render switchRow("islandHaptics", "island.haptics", "island.haptics.desc")}
      </div>

      <h3 class="sec-title">{$t("island.sec.position")}</h3>
      <p class="lede">{$t("island.position.desc")}</p>
      <div class="card">
        <div class="pos-picker">
          <button
            class="pos-option"
            class:active={$settingsPrefs.islandPosition === "top"}
            onclick={() => setPref("islandPosition", "top")}
            aria-pressed={$settingsPrefs.islandPosition === "top"}
          >
            <span class="pos-screen"><span class="pos-pill top"></span></span>
            <span class="pos-label">{$t("island.pos.top")}</span>
          </button>
          <button
            class="pos-option"
            class:active={$settingsPrefs.islandPosition === "bottom"}
            onclick={() => setPref("islandPosition", "bottom")}
            aria-pressed={$settingsPrefs.islandPosition === "bottom"}
          >
            <span class="pos-screen"><span class="pos-pill bottom"></span></span>
            <span class="pos-label">{$t("island.pos.bottom")}</span>
          </button>
        </div>
      </div>


      <div class="sec-head">
        <h3 class="sec-title">{$t("island.sec.size")}</h3>
        <button
          class="link-btn"
          onclick={() => {
            setPref("islandWidth", 100);
            setPref("islandHeight", 100);
          }}
        >
          {$t("island.reset")}
        </button>
      </div>
      <p class="lede">{$t("island.size.desc")}</p>
      <div class="card">
        <div class="island-previews">
          <div class="preview-box">
            <div class="preview-pill">
              <img class="ptile-img" src={appIcon} alt="Minime" draggable="false" />
              <span class="pfill"></span>
              <span aria-hidden="true">🙂</span>
            </div>
            <span class="preview-cap">{$t("island.display.builtin")}</span>
          </div>
          <div class="preview-box">
            <div class="preview-pill">
              <img class="ptile-img" src={appIcon} alt="Minime" draggable="false" />
              <span class="pfill"></span>
              <span aria-hidden="true">🙂</span>
            </div>
            <span class="preview-cap">{$t("island.display.external")}</span>
          </div>
        </div>
        <div class="srow col">
          <div class="slider-head">
            <span class="srow-label">{$t("island.width")}</span>
            <span class="slider-val">{$settingsPrefs.islandWidth}%</span>
          </div>
          <input
            class="slider"
            type="range"
            min="60"
            max="140"
            step="5"
            value={$settingsPrefs.islandWidth}
            oninput={(e) => setPref("islandWidth", Number((e.currentTarget as HTMLInputElement).value))}
            aria-label={$t("island.width")}
          />
        </div>
        <div class="srow col">
          <div class="slider-head">
            <span class="srow-label">{$t("island.height")}</span>
            <span class="slider-val">{$settingsPrefs.islandHeight}%</span>
          </div>
          <input
            class="slider"
            type="range"
            min="60"
            max="140"
            step="5"
            value={$settingsPrefs.islandHeight}
            oninput={(e) => setPref("islandHeight", Number((e.currentTarget as HTMLInputElement).value))}
            aria-label={$t("island.height")}
          />
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("island.status")}</span>
            <span class="srow-desc">
              {#if $islandStatus}
                {$islandStatus.mode === "off"
                  ? $t("island.mode.off")
                  : $islandStatus.mode === "idle"
                    ? $t("island.mode.idle")
                    : $t("island.mode.status")} · {$islandStatus.displays} {$t("island.displays")}
              {:else}
                —
              {/if}
            </span>
          </div>
        </div>
      </div>
    {:else if category === "computer"}
      <h2>{$t("set.cat.computer")}</h2>

      <h3 class="sec-title">{$t("comp.sec.permissions")}</h3>
      <p class="lede">{$t("comp.permissions.desc")}</p>
      <div class="card">
        <div class="perm-row">
          <span class="perm-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="2" y="3" width="20" height="14" rx="2" /><path d="M8 21h8M12 17v4" /></svg>
          </span>
          <div class="srow-meta">
            <span class="srow-label">{$t("comp.screen")}</span>
            <span class="srow-desc">{$t("comp.screen.desc")}</span>
          </div>
          {#if sysPerms && !sysPerms.supported}
            <span class="perm-badge neutral">{$t("comp.notneeded")}</span>
          {:else if sysPerms?.screen_recording}
            <span class="perm-badge granted">✓ {$t("comp.granted")}</span>
          {:else}
            <span class="perm-badge denied">{$t("comp.notgranted")}</span>
          {/if}
          <button class="btn-ghost" onclick={() => openPrivacySettings("screen")}>
            {$t("comp.manage")}
          </button>
        </div>
        <div class="perm-row">
          <span class="perm-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z" /><path d="M13 13l6 6" /></svg>
          </span>
          <div class="srow-meta">
            <span class="srow-label">{$t("comp.access")}</span>
            <span class="srow-desc">{$t("comp.access.desc")}</span>
          </div>
          {#if sysPerms && !sysPerms.supported}
            <span class="perm-badge neutral">{$t("comp.notneeded")}</span>
          {:else if sysPerms?.accessibility}
            <span class="perm-badge granted">✓ {$t("comp.granted")}</span>
          {:else}
            <span class="perm-badge denied">{$t("comp.notgranted")}</span>
          {/if}
          <button class="btn-ghost" onclick={() => openPrivacySettings("accessibility")}>
            {$t("comp.manage")}
          </button>
        </div>
      </div>
      <div class="check-row">
        <button class="btn-ghost" onclick={refreshPerms}>↻ {$t("comp.check")}</button>
      </div>

      <h3 class="sec-title">{$t("set.sec.computer")}</h3>
      <div class="card">
        {@render switchRow("computerBrowser", "set.comp.browser", "set.comp.browser.desc")}
        {@render switchRow("computerHidden", "set.comp.hidden", "set.comp.hidden.desc")}
        {@render switchRow("computerMirror", "set.comp.mirror", "set.comp.mirror.desc")}
      </div>
    {:else if category === "profile"}
      <h2>{$t("set.cat.profile")}</h2>

      <h3 class="sec-title">{$t("prof.sec.identity")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("prof.display")}</span>
            <span class="srow-desc">{$t("prof.display.desc")}</span>
          </div>
          <input
            class="sinput"
            type="text"
            value={$profileStore.name}
            oninput={(e) => profileStore.update((p) => ({ ...p, name: (e.currentTarget as HTMLInputElement).value }))}
            placeholder="Patria A. Wijaya"
            aria-label={$t("prof.display")}
          />
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("prof.photo")}</span>
            <span class="srow-desc">{$t("prof.photo.desc")}</span>
          </div>
          <div class="photo-actions">
            {#if $profileStore.photo}
              <button class="link-btn" onclick={removePhoto}>{$t("prof.photo.remove")}</button>
            {/if}
            <button
              class="avatar-btn"
              onclick={() => photoInput?.click()}
              aria-label={$t("prof.photo.change")}
              title={$t("prof.photo.change")}
            >
              {#if $profileStore.photo}
                <img src={$profileStore.photo} alt="" />
              {:else}
                <span>{initials}</span>
              {/if}
            </button>
          </div>
          <input
            type="file"
            accept="image/*"
            bind:this={photoInput}
            onchange={onPhotoChosen}
            hidden
          />
        </div>
      </div>

      <h3 class="sec-title">{$t("prof.sec.account")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">Email</span>
            <span class="srow-desc">{$t("prof.email.desc")}</span>
          </div>
          {#if $profileStore.email}
            <span class="email-text">{$profileStore.email}</span>
            <button class="btn-ghost" onclick={onSignOut}>
              {signOutArm ? $t("prof.signout.confirm") : $t("prof.signout")}
            </button>
          {:else}
            <input
              class="sinput"
              type="email"
              bind:value={emailDraft}
              placeholder="nama@email.com"
              aria-label="Email"
            />
            <button class="btn-ghost" disabled={!emailDraft.trim()} onclick={saveEmail}>
              {$t("prof.save")}
            </button>
          {/if}
        </div>
      </div>

      <h3 class="sec-title">{$t("prof.sec.sessions")}</h3>
      <p class="lede">{$t("prof.sessions.desc")}</p>
      <button class="btn-ghost wide" onclick={refreshSession}>↻ {$t("prof.refresh")}</button>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("prof.desktop")} · {$t("prof.thisdevice")}</span>
            <span class="srow-desc">
              {deviceInfo?.name ?? "Desktop"}{deviceInfo ? ` (${deviceInfo.os})` : ""} · {$t("prof.signedin")} {fmtDate($profileStore.signedInAt)} · {$t("prof.lastactive")} {fmtDate(lastActive)}
            </span>
          </div>
          <span class="perm-badge neutral">{$t("prof.thisdevice")}</span>
        </div>
      </div>
    {:else if category === "billing"}
      <h2>{$t("set.cat.billing")}</h2>

      <h3 class="sec-title">{$t("bill.sec.plans")}</h3>

      <div class="empty-box">{$t("bill.none")}</div>

      <h3 class="sec-title">{$t("bill.sec.other")}</h3>
      <p class="lede">{$t("bill.other.desc")}</p>
      <p class="lede">{$t("bill.nohosted")}</p>


    {:else if category === "mobile"}
      <h2>{$t("set.cat.mobile")}</h2>

      <h3 class="sec-title">{$t("mob.sec.connect")}</h3>
      <p class="lede">{$t("mob.connect.desc")}</p>

      <div class="card">
        <div class="step-head">
          <span class="step-badge">1</span>
          <div class="srow-meta">
            <span class="srow-label">{$t("mob.install")}</span>
            <span class="srow-desc">{$t("mob.install.desc")}</span>
          </div>
          <div class="seg-toggle" role="group" aria-label={$t("mob.install")}>
            <button class:active={mobilePlatform === "ios"} onclick={() => (mobilePlatform = "ios")}>
               iPhone
            </button>
            <button class:active={mobilePlatform === "android"} onclick={() => (mobilePlatform = "android")}>
               Android
            </button>
          </div>
        </div>
        <div class="install-grid">
          <div class="qr-box">
            {#if inviteQr}
              <img src={inviteQr} alt="QR" />
            {/if}
          </div>
          <div class="install-info">
            <div class="install-title">
              <span class="srow-label">
                {$t(mobilePlatform === "ios" ? "mob.for.iphone" : "mob.for.android")}
              </span>
              <span class="perm-badge neutral">{$t("mob.beta")}</span>
            </div>
            <p class="install-note">
              {$t(mobilePlatform === "ios" ? "mob.ios.note" : "mob.android.note")}
            </p>
            <ol class="steps">
              <li>{$t(mobilePlatform === "ios" ? "mob.step.ios1" : "mob.step.and1")}</li>
              <li>{$t(mobilePlatform === "ios" ? "mob.step.ios2" : "mob.step.and2")}</li>
              <li>{$t(mobilePlatform === "ios" ? "mob.step.ios3" : "mob.step.and3")}</li>
            </ol>
            <button class="btn-ghost" onclick={() => copyText(permanentLink, "invite")}>
              ⧉ {copiedWhich === "invite" ? $t("mob.copied") : $t("mob.copylink")}
            </button>
          </div>
        </div>
      </div>

      <div class="card step-card">
        <div class="step-head">
          <span class="step-badge">2.1</span>
          <div class="srow-meta">
            <span class="srow-label">{$t("mob.signin")}</span>
            <span class="srow-desc">{$t("mob.signin.desc")}</span>
          </div>
          <button class="btn-primary" onclick={generateOneTime}>
            {$t("mob.genqr")}
          </button>
        </div>
        {#if oneTime}
          <div class="install-grid">
            <div class="qr-box">
              {#if oneTimeQr}
                <img src={oneTimeQr} alt="QR" />
              {/if}
            </div>
            <div class="install-info">
              <span class="srow-label">{$t("mob.yourcode")}</span>
              <span class="pair-code">{oneTime.code}</span>
              {#if oneTimeLeft > 0}
                <p class="install-note">
                  {$t("mob.waiting")} · {Math.floor(oneTimeLeft / 60)}:{String(oneTimeLeft % 60).padStart(2, "0")}
                </p>
              {:else}
                <p class="install-note expired">{$t("mob.expired")}</p>
              {/if}
              <button class="btn-ghost" onclick={() => copyText(oneTimeLink, "once")}>
                ⧉ {copiedWhich === "once" ? $t("mob.copied") : $t("mob.copylink")}
              </button>
            </div>
          </div>
        {/if}
      </div>

      <div class="card step-card">
        <div class="step-head">
          <span class="step-badge">2.2</span>
          <div class="srow-meta">
            <span class="srow-label">{$t("mob.permanent")}</span>
            <span class="srow-desc">{$t("mob.permanent.desc")}</span>
          </div>
          {#if $settingsPrefs.mobileLinkToken}
            <button class="btn-ghost danger" onclick={revokePermanent}>
              {$t("mob.revoke")}
            </button>
          {:else}
            <button class="btn-primary" onclick={generatePermanent}>
              {$t("mob.genlink")}
            </button>
          {/if}
        </div>
        {#if $settingsPrefs.mobileLinkToken}
          <div class="perm-link-row">
            <code class="perm-link">{permanentLink}</code>
            <button class="btn-ghost" onclick={() => copyText(permanentLink, "perm")}>
              ⧉ {copiedWhich === "perm" ? $t("mob.copied") : $t("mob.copylink")}
            </button>
          </div>
        {/if}
      </div>

      <h3 class="sec-title">{$t("mob.devices")}</h3>
      <div class="card">
        <div class="device-empty">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><rect x="5" y="2" width="14" height="20" rx="2" /><path d="M12 18h.01" /></svg>
          <div class="srow-meta">
            <span class="srow-label">{$t("mob.nodevices")}</span>
            <span class="srow-desc">{$t("mob.nodevices.desc")}</span>
          </div>
        </div>
      </div>
    {:else if category === "updates"}
      <h2>{$t("set.cat.updates")}</h2>

      <h3 class="sec-title">{$t("upd.sec")}</h3>
      <button class="btn-ghost wide" onclick={() => (showWhatsNew = !showWhatsNew)}>
        {$t("upd.whatsnew")}
      </button>
      {#if showWhatsNew}
        <div class="card whatsnew">
          {#if $updateInfo?.notes}
            <pre class="release-notes">{$updateInfo.notes}</pre>
          {:else}
            <p class="lede">{$t("upd.norelease")}</p>
          {/if}
        </div>
      {/if}
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("upd.track")}</span>
            <span class="srow-desc">
              {$t($settingsPrefs.updateTrack === "beta" ? "upd.track.beta.desc" : "upd.track.stable.desc")}
            </span>
          </div>
          <select
            class="sselect"
            value={$settingsPrefs.updateTrack}
            onchange={(e) => {
              setPref("updateTrack", (e.currentTarget as HTMLSelectElement).value as "stable" | "beta");
              void checkForUpdate();
            }}
            aria-label={$t("upd.track")}
          >
            <option value="stable">Stable</option>
            <option value="beta">Beta</option>
          </select>
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">
              {$t("upd.version")} {$updateInfo?.current ?? APP_VERSION}
            </span>
            <span class="srow-desc">
              {$t($settingsPrefs.updateTrack === "beta" ? "upd.follows.beta" : "upd.follows.stable")}
            </span>
            {#if $updateInfo?.update_available}
              <span class="update-ready">
                Minime v{$updateInfo.latest} {$t("upd.ready")}
              </span>
            {:else if $updateInfo}
              <span class="srow-desc">{$t("settings.upToDate")}</span>
            {/if}
          </div>
          {#if $updateInfo?.update_available}
            <button class="btn-ghost" onclick={() => restartApp()}>
              {$t("upd.restart")}
            </button>
          {:else}
            <button class="btn-ghost" onclick={() => checkForUpdate()}>
              {$t("settings.checkUpdates")}
            </button>
          {/if}
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("upd.restartrow")}</span>
            <span class="srow-desc">{$t("upd.restartrow.desc")}</span>
            {#if updNote}
              <span class="srow-desc note-warn">{updNote}</span>
            {/if}
          </div>
          <button class="btn-ghost" onclick={installWhenIdle}>
            {$t("upd.installidle")}
          </button>
        </div>
        {@render switchRow("autoUpdate", "upd.auto", "upd.auto.desc")}
        {@render switchRow("updateAutoInstall", "upd.autoinstall", "upd.autoinstall.desc")}
        {@render switchRow("updateMembers", "upd.members", "upd.members.desc")}
      </div>
      <p class="lede about">{$t("settings.about")}</p>
    {:else if category === "providers"}
      <h2>{$t("set.cat.providers")}</h2>
  <div class="section">
    <div class="row-between">
      <h3>{$t("providers.title")}</h3>
      <button class="btn-ghost sm" onclick={() => refreshProviders()}>
        {$t("providers.recheck")}
      </button>
    </div>

    <span class="group-label">Model default</span>
    <p class="lede" style="margin-bottom:12px">Model yang dipakai setiap agen — format <code>provider/model</code>.</p>
    <div style="margin-bottom:20px">
      <ModelPicker value={defaultModelSpec} onChange={(v) => (defaultModelSpec = v)} providers={providerCatalog} />
    </div>

    <span class="group-label">{$t("providers.title")}</span>
    <p class="lede" style="margin-bottom:12px">Bawa key sendiri. Atur sebanyak mungkin provider dan campur antar agen. Runtime lokal (Ollama) hanya butuh endpoint.</p>
    {#each providerCatalog as entry (entry.id)}
      <ProviderRow {entry} onChanged={loadCatalog} />
    {/each}

    <div class="prov-row">
      <div class="prov-meta">
        <span class="prov-name">{$t("providers.custom")}</span>
        <span class="prov-status">{$t("providers.custom.desc")}</span>
      </div>
      <button class="btn-ghost sm" onclick={() => (showProvDialog = true)}>
        {$t("providers.add")}
      </button>
    </div>
    {#each $customProviders as c (c.id)}
      <div class="prov-row">
        <div class="prov-meta">
          <span class="prov-name">{c.name}</span>
          <span class="prov-status">{c.endpoint}</span>
        </div>
        <label class="switch">
          <input
            type="checkbox"
            checked={c.active}
            onchange={(e) => toggleCustomProvider(c.id, e.currentTarget.checked)}
            aria-label={c.name}
          />
          <span class="track" aria-hidden="true"></span>
        </label>
      </div>
    {/each}

    <span class="group-label">{$t("providers.found")}</span>
    <div class="prov-row">
      <div class="prov-meta">
        <span class="prov-name">Ollama</span>
        <span class="prov-status">{$t("providers.models", { n: "4" })}</span>
      </div>
      <button class="btn-ghost sm" onclick={() => connectProvider("ollama")}>
        {$t("providers.add")}
      </button>
    </div>
    <button class="btn-ghost sm scan-btn" onclick={rescan} disabled={scanning}>
      {scanning ? "…" : $t("providers.scan")}
    </button>

    <div class="row">
      <div class="meta">
        <span class="label">{$t("providers.autoScan")}</span>
      </div>
      <label class="switch">
        <input
          type="checkbox"
          bind:checked={$autoScan}
          aria-label={$t("providers.autoScan")}
        />
        <span class="track" aria-hidden="true"></span>
      </label>
    </div>
  </div>
  {#if showProvDialog}
    <div
      class="overlay"
      role="presentation"
      onclick={(e) => { if (e.target === e.currentTarget) showProvDialog = false; }}
    >
      <div class="dialog" role="dialog" aria-modal="true" aria-label={$t("providers.customTitle")}>
        <h3>{$t("providers.customTitle")}</h3>
        <p class="lede">{$t("providers.custom.desc")}</p>
        <div class="field">
          <label for="prov-id">Provider ID *</label>
          <input id="prov-id" type="text" bind:value={provId} placeholder="my-provider" />
          <span class="hint">Huruf kecil, angka, strip, atau garis bawah.</span>
        </div>
        <div class="field">
          <label for="prov-name">{$t("providers.customName")} *</label>
          <input id="prov-name" type="text" bind:value={provName} />
        </div>
        <div class="field">
          <label for="prov-endpoint">Base URL *</label>
          <input id="prov-endpoint" type="text" bind:value={provEndpoint} placeholder="https://api.example.com/v1" />
        </div>
        <div class="field">
          <label for="prov-key">API key</label>
          <input id="prov-key" type="password" bind:value={provApiKey} placeholder="Opsional" />
          <span class="hint">Opsional. Kosongkan jika auth via header.</span>
        </div>
        <div class="field">
          <div class="models-head">
            <label>Models</label>
            <button
              class="btn-ghost sm"
              onclick={findModels}
              disabled={!provEndpoint.trim() || findingModels}
            >
              {findingModels ? "..." : "↻ Find again"}
            </button>
          </div>
          {#if discoveredModels.length > 0}
            <p class="hint">Endpoint lists {discoveredModels.length} models. Select the models to add.</p>
            <div class="model-checks">
              {#each discoveredModels as mid}
                <label class="check-row">
                  <input
                    type="checkbox"
                    checked={selectedModels.includes(mid)}
                    onchange={(e) => toggleModelSelect(mid, e.currentTarget.checked)}
                  />
                  <span class="mono">{mid}</span>
                </label>
              {/each}
            </div>
          {:else if modelsError}
            <p class="err">{modelsError}</p>
          {/if}
        </div>
        <div class="actions">
          <button class="btn-ghost sm" onclick={() => (showProvDialog = false)}>
            {$t("providers.cancel")}
          </button>
          <button
            class="btn-primary"
            onclick={createProvider}
            disabled={!provId.trim() || !provName.trim() || !provEndpoint.trim() || selectedModels.length === 0}
          >
            Submit
          </button>
        </div>
      </div>
    </div>
  {/if}
    {/if}
  </div>
</div>

<style>
  .meta { display: flex; flex-direction: column; flex: 1; min-width: 0; }
  .btn-ghost { flex: none; }
  .group-label {
    display: block;
    font-size: 13px;
    font-weight: 600;
    color: var(--dim);
    margin-bottom: 6px;
  }
  .prov-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 4px;
    border-bottom: 1px solid var(--border);
  }
  .prov-row input[type="radio"] {
    width: 18px;
    height: 18px;
    margin: 0;
    flex: none;
    accent-color: var(--accent);
  }
  .prov-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    cursor: pointer;
  }
  .prov-name {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 15px;
    font-weight: 600;
  }
  .prov-name .chip { align-self: center; }
  .prov-status {
    font-size: 13px;
    color: var(--dim);
  }
  .prov-sub {
    display: block;
    font-size: 13px;
    color: var(--dim);
    margin-top: 2px;
  }
  .prov-warn {
    display: block;
    color: #f59e0b;
    margin-top: 2px;
  }
  .prov-right {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-left: auto;
  }
  .prov-ver {
    font-size: 12px;
    color: var(--dim);
  }
  .btn-primary.sm {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 6px 16px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .btn-primary.sm:hover { background: #2563eb; }
  .badge {
    display: inline-block;
    padding: 2px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
  }
  .badge.ok {
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
  }
  .badge.warn {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
  }
  .models-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .models-head label { font-weight: 600; }
  .model-checks {
    max-height: 180px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px;
  }
  .check-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 4px;
    cursor: pointer;
  }
  .check-row:hover { background: var(--hover); }
  .check-row .mono {
    font-family: ui-monospace, monospace;
    font-size: 13px;
  }
  .hint { font-size: 12px; color: var(--dim); margin-top: 4px; }
  .err { font-size: 13px; color: #ef4444; margin-top: 8px; }
  .scan-btn { margin-top: 12px; }
  .dialog .lede { margin-bottom: 16px; }
  .prov-detail {
    padding: 4px 4px 12px 34px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .prov-detail .note {
    font-size: 13px;
    color: var(--dim);
    margin: 0;
  }
  .row-end {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .row-end .btn-primary { min-height: 44px; }
  .row-between {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .row-between h3 { margin: 0; }
  .ok-note {
    font-size: 13px;
    color: var(--accent);
  }

/* ---------- Two-column settings layout (reference layout) ---------- */
.settings-layout {
  display: grid;
  grid-template-columns: 300px minmax(0, 1fr);
  height: 100%;
  min-height: 0;
}
.set-side {
  border-right: 1px solid var(--border);
  background: var(--panel);
  padding: 20px 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
  min-height: 0;
}
.set-side h2 {
  font-size: 19px;
  margin: 0 4px;
}
.set-search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  min-height: 38px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--panel-2);
  color: var(--dim);
}
.set-search svg { width: 16px; height: 16px; flex: none; }
.set-search input {
  flex: 1;
  border: 0;
  background: transparent;
  color: var(--text);
  font: inherit;
  font-size: 14px;
  outline: none;
}
.set-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.set-nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 42px;
  padding: 8px 12px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  color: var(--text-2);
  font-size: 14.5px;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
}
.set-nav-item svg { width: 20px; height: 20px; flex: none; }
.set-nav-item:hover { background: var(--panel-2); color: var(--text); }
.set-nav-item.active { background: var(--panel-2); color: var(--text); }
.set-main {
  overflow-y: auto;
  min-height: 0;
  padding: 26px 40px 40px;
  max-width: 1024px;
}
.set-main h2 {
  font-size: 21px;
  margin: 0 0 18px;
}
.sec-title {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--dim);
  margin: 22px 0 8px;
}
.sec-title:first-of-type { margin-top: 0; }
.card {
  background: var(--panel);
  border: 1px solid transparent;
  border-radius: 14px;
  padding: 4px 18px;
}
.srow {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 0;
}
.srow + .srow { border-top: 1px solid var(--border); }
.srow-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.srow-label { font-size: 15px; font-weight: 600; color: var(--text); }
.srow-desc { font-size: 13.5px; color: var(--dim); }
.sselect {
  min-height: 38px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel-2);
  color: var(--text);
  font: inherit;
  font-size: 14px;
  min-width: 170px;
}
.sinput {
  min-height: 38px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel-2);
  color: var(--text);
  font: inherit;
  font-size: 14px;
  min-width: 240px;
}
.logo-colors {
  display: grid;
  grid-template-columns: repeat(5, 30px);
  gap: 9px;
}
.color-dot {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
}
.color-dot.active {
  border-color: var(--text);
  box-shadow: 0 0 0 2px var(--panel) inset;
}
.pair-code {
  font-size: 22px;
  font-weight: 800;
  letter-spacing: 0.18em;
  color: var(--text);
}
/* ---------- Voice: dictation model rows ---------- */
.model-row {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 16px 0;
}
.model-row + .model-row { border-top: 1px solid var(--border); }
.model-row.col { flex-direction: column; align-items: stretch; }
.model-pick {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  background: none;
  border: 0;
  padding: 0;
  text-align: left;
  cursor: pointer;
  color: inherit;
  font: inherit;
}
.model-name { font-size: 15px; font-weight: 700; color: var(--text); }
.model-desc { font-size: 13.5px; color: var(--dim); }
.model-sub { font-size: 13.5px; color: var(--dim); }
.model-actions {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: none;
  padding-top: 2px;
}
.inuse { font-size: 13.5px; font-weight: 600; color: var(--dim); }
.key-form {
  display: flex;
  align-items: center;
  gap: 10px;
}
.key-form input {
  flex: 1;
  min-height: 40px;
  padding: 0 14px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel-2);
  color: var(--text);
  font: inherit;
  font-size: 14px;
}
.key-saved { flex: 1; font-size: 13.5px; color: var(--ok); }
/* ---------- Dynamic Island settings ---------- */
.sec-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-top: 22px;
}
.sec-head .sec-title { margin: 0; }
.link-btn {
  background: none;
  border: 0;
  padding: 0;
  color: var(--dim);
  font: inherit;
  font-size: 13.5px;
  font-weight: 600;
  cursor: pointer;
}
.link-btn:hover { color: var(--text); }
.server-tile {
  width: 34px;
  height: 34px;
  border-radius: 9px;
  flex: none;
}
.island-previews {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  padding: 16px 0 6px;
}
.preview-box {
  background: var(--panel-2);
  border-radius: 12px;
  padding: 18px 12px 10px;
  text-align: center;
}
.preview-pill {
  margin: 0 auto;
  width: 82%;
  height: 34px;
  border-radius: 999px;
  background: #0a0a0c;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px 0 6px;
  box-sizing: border-box;
}
.preview-pill .ptile-img {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  object-fit: cover;
  flex: none;
}
.preview-pill .pfill { flex: 1; }
.preview-cap {
  display: block;
  margin-top: 10px;
  font-size: 12.5px;
  color: var(--dim);
}
.srow.col {
  flex-direction: column;
  align-items: stretch;
  gap: 6px;
}
.slider-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}
.slider-val { font-size: 13.5px; color: var(--dim); font-weight: 600; }
.slider {
  width: 100%;
  height: 28px;
  accent-color: var(--accent);
  cursor: pointer;
}
/* ---------- Dynamic Island position picker ---------- */
.pos-picker {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  padding: 14px 0;
}
.pos-option {
  background: var(--panel-2);
  border: 2px solid transparent;
  border-radius: 12px;
  padding: 14px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
  color: var(--text);
  font: inherit;
}
.pos-option:hover { border-color: var(--border); }
.pos-option.active { border-color: var(--accent); }
.pos-screen {
  position: relative;
  width: 100%;
  height: 74px;
  border-radius: 8px;
  background: linear-gradient(180deg, #2b2e34, #1b1d21);
  overflow: hidden;
}
.pos-pill {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  width: 44%;
  height: 12px;
  border-radius: 999px;
  background: #050506;
}
.pos-pill.top { top: 6px; }
.pos-pill.bottom { bottom: 6px; }
.pos-label { font-size: 13.5px; font-weight: 600; }
/* ---------- Computer Use: permission rows ---------- */
.perm-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 0;
}
.perm-row + .perm-row { border-top: 1px solid var(--border); }
.perm-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: linear-gradient(135deg, #2f6fd0, #4fa6f0);
  display: grid;
  place-items: center;
  color: #fff;
  flex: none;
}
.perm-icon svg { width: 20px; height: 20px; }
.perm-badge {
  font-size: 12.5px;
  font-weight: 700;
  padding: 5px 11px;
  border-radius: 999px;
  flex: none;
  white-space: nowrap;
}
.perm-badge.granted { background: rgba(46, 160, 90, 0.18); color: #3ecf7a; }
.perm-badge.denied { background: rgba(220, 80, 70, 0.16); color: #ff7a70; }
.perm-badge.neutral { background: var(--panel-2); color: var(--dim); }
.check-row {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}
/* ---------- Profile & Billing ---------- */
.photo-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}
.avatar-btn {
  width: 46px;
  height: 46px;
  border-radius: 50%;
  border: 0;
  overflow: hidden;
  background: #1f3a5f;
  color: #7db4ff;
  font-weight: 800;
  font-size: 15px;
  display: grid;
  place-items: center;
  cursor: pointer;
  padding: 0;
  flex: none;
}
.avatar-btn img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.email-text {
  font-size: 14.5px;
  font-weight: 600;
  color: var(--text);
}
.btn-ghost.wide {
  width: 100%;
  margin: 4px 0 12px;
}
.empty-box {
  border: 1.5px dashed var(--border);
  border-radius: 14px;
  padding: 26px;
  text-align: center;
  color: var(--dim);
  font-size: 14px;
  margin-top: 14px;
}
.add-server-row {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
}
/* ---------- Mobile Connect & Updates ---------- */
.step-card { margin-top: 16px; }
.step-head {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 0;
}
.step-badge {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: #1f3a5f;
  color: #7db4ff;
  font-size: 12px;
  font-weight: 700;
  display: grid;
  place-items: center;
  flex: none;
  margin-top: 2px;
}
.seg-toggle {
  display: flex;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 3px;
  gap: 2px;
  flex: none;
}
.seg-toggle button {
  border: 0;
  background: transparent;
  color: var(--dim);
  font: inherit;
  font-size: 13.5px;
  font-weight: 600;
  padding: 6px 14px;
  border-radius: 8px;
  cursor: pointer;
}
.seg-toggle button.active {
  background: var(--panel);
  color: var(--text);
}
.install-grid {
  display: flex;
  gap: 20px;
  padding: 6px 0 16px;
}
.qr-box {
  background: #fff;
  border-radius: 12px;
  padding: 10px;
  width: 148px;
  height: 148px;
  flex: none;
  box-sizing: border-box;
}
.qr-box img {
  width: 100%;
  height: 100%;
  display: block;
}
.install-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: flex-start;
}
.install-title {
  display: flex;
  align-items: center;
  gap: 10px;
}
.install-note {
  font-size: 13.5px;
  color: var(--dim);
  margin: 0;
}
.install-note.expired { color: #ff7a70; }
.steps {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13.5px;
  color: var(--dim);
}
.btn-primary {
  background: var(--accent);
  color: #fff;
  border: 0;
  border-radius: 10px;
  min-height: 40px;
  padding: 0 18px;
  font: inherit;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  flex: none;
}
.btn-primary:disabled { opacity: 0.5; cursor: default; }
.btn-ghost.danger { color: #ff7a70; }
.perm-link-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 0 16px;
}
.perm-link {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  color: var(--dim);
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 9px 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.device-empty {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 0;
  color: var(--dim);
}
.device-empty svg { width: 22px; height: 22px; flex: none; }
.whatsnew { margin-bottom: 14px; }
.release-notes {
  font: inherit;
  font-size: 13.5px;
  color: var(--text-2);
  white-space: pre-wrap;
  word-break: break-word;
  margin: 12px 0;
  max-height: 260px;
  overflow-y: auto;
}
.update-ready {
  font-size: 13.5px;
  color: var(--accent);
  font-weight: 600;
}
.note-warn { color: #ffb84d !important; }
.lede.about { margin-top: 16px; }
.lede {
  color: var(--dim);
  font-size: 14px;
  margin: 0 0 10px;
}
.lede.ok { color: var(--ok); }
@media (max-width: 900px) {
  .settings-layout { grid-template-columns: 230px minmax(0, 1fr); }
  .set-main { padding: 20px; }
}
</style>
