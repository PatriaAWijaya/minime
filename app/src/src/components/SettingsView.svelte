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
    settingsPrefs,
    setPref,
    profileStore,
    theme,
    toggleCustomProvider,
    toggleProvider,
    usingRealBackend,
  } from "../lib/store";
  import type { Theme } from "../lib/store";
  import { onMount } from "svelte";

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

  let category = $state<CatId>("general");
  let catQuery = $state("");
  const visibleCats = $derived(
    CATEGORIES.filter(
      (c) =>
        !catQuery.trim() ||
        $t(c.labelKey).toLowerCase().includes(catQuery.trim().toLowerCase()),
    ),
  );

  /* ---------- Mobile Connect pairing code ---------- */
  let pairCode = $state("");
  function regenPairCode() {
    pairCode = String(Math.floor(100000 + Math.random() * 900000));
  }
  regenPairCode();

  let checkedUpdates = $state(false);
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
  });
  let provEndpoint = $state("");

  const APP_VERSION = "0.1.0";

  function createProvider() {
    addCustomProvider(provName, provEndpoint);
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
      <h3 class="sec-title">{$t("set.sec.island")}</h3>
      <div class="card">
        {@render switchRow("islandEnabled", "set.island.enabled", "set.island.enabled.desc")}
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.island.position")}</span>
            <span class="srow-desc">{$t("set.island.position.desc")}</span>
          </div>
          <select
            class="sselect"
            value={$settingsPrefs.islandPosition}
            onchange={(e) => setPref("islandPosition", (e.currentTarget as HTMLSelectElement).value as "top" | "bottom")}
            aria-label={$t("set.island.position")}
          >
            <option value="top">{$t("set.pos.top")}</option>
            <option value="bottom">{$t("set.pos.bottom")}</option>
          </select>
        </div>
      </div>
    {:else if category === "computer"}
      <h2>{$t("set.cat.computer")}</h2>
      <h3 class="sec-title">{$t("set.sec.computer")}</h3>
      <div class="card">
        {@render switchRow("computerBrowser", "set.comp.browser", "set.comp.browser.desc")}
        {@render switchRow("computerHidden", "set.comp.hidden", "set.comp.hidden.desc")}
        {@render switchRow("computerMirror", "set.comp.mirror", "set.comp.mirror.desc")}
      </div>
    {:else if category === "profile"}
      <h2>{$t("set.cat.profile")}</h2>
      <h3 class="sec-title">{$t("set.sec.profile")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.profile.name")}</span>
            <span class="srow-desc">{$t("set.profile.name.desc")}</span>
          </div>
          <input
            class="sinput"
            type="text"
            value={$profileStore.name}
            oninput={(e) => profileStore.update((p) => ({ ...p, name: (e.currentTarget as HTMLInputElement).value }))}
            placeholder="Patria A. Wijaya"
            aria-label={$t("set.profile.name")}
          />
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.profile.email")}</span>
            <span class="srow-desc">{$t("set.profile.email.desc")}</span>
          </div>
          <input
            class="sinput"
            type="email"
            value={$profileStore.email}
            oninput={(e) => profileStore.update((p) => ({ ...p, email: (e.currentTarget as HTMLInputElement).value }))}
            placeholder="nama@email.com"
            aria-label={$t("set.profile.email")}
          />
        </div>
      </div>
    {:else if category === "billing"}
      <h2>{$t("set.cat.billing")}</h2>
      <h3 class="sec-title">{$t("set.sec.billing")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.billing.plan")}</span>
            <span class="srow-desc">{$t("set.billing.plan.desc")}</span>
          </div>
          <button class="btn-ghost" disabled>{$t("set.billing.soon")}</button>
        </div>
      </div>
    {:else if category === "mobile"}
      <h2>{$t("set.cat.mobile")}</h2>
      <h3 class="sec-title">{$t("set.sec.mobile")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.mobile.code")}</span>
            <span class="srow-desc">{$t("set.mobile.code.desc")}</span>
          </div>
          <span class="pair-code">{pairCode}</span>
        </div>
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("set.mobile.regen")}</span>
            <span class="srow-desc">{$t("set.mobile.regen.desc")}</span>
          </div>
          <button class="btn-ghost" onclick={regenPairCode}>{$t("set.mobile.regen")}</button>
        </div>
      </div>
    {:else if category === "updates"}
      <h2>{$t("set.cat.updates")}</h2>
      <h3 class="sec-title">{$t("set.sec.updatesec")}</h3>
      <div class="card">
        <div class="srow">
          <div class="srow-meta">
            <span class="srow-label">{$t("settings.version")}</span>
            <span class="srow-desc">Minime {APP_VERSION}</span>
          </div>
          <button class="btn-ghost" onclick={() => (checkedUpdates = true)}>
            {$t("settings.checkUpdates")}
          </button>
        </div>
        {@render switchRow("autoUpdate", "set.update.auto", "set.update.auto.desc")}
      </div>
      {#if checkedUpdates}
        <p class="lede ok">{$t("settings.upToDate")}</p>
      {/if}
      <p class="lede">{$t("settings.about")}</p>
    {:else if category === "providers"}
      <h2>{$t("set.cat.providers")}</h2>
  <div class="section">
    <div class="row-between">
      <h3>{$t("providers.title")}</h3>
      <button class="btn-ghost sm" onclick={() => refreshProviders()}>
        {$t("providers.recheck")}
      </button>
    </div>

    <span class="group-label">{$t("providers.default")}</span>
    {#each PROVIDERS as p}
      {@const st = $providerState[p.id] ?? { connected: false, active: true }}
      {@const rs = realStatus(p.id)}
      {@const isPaid = p.kind === "paid"}
      <div class="prov-row">
        <input
          type="radio"
          id="prov-{p.id}"
          name="default-provider"
          checked={$defaultProviderId === p.id}
          onchange={() => defaultProviderId.set(p.id)}
          aria-label={p.name}
        />
        <label for="prov-{p.id}" class="prov-meta">
          <span class="prov-name">
            {p.name}
            <span class="chip {p.kind === 'paid' ? 'paid' : ''}">
              {$t(p.kind === "free" ? "providers.free" : "providers.paid")}
            </span>
            {#if p.tagKey}
              <span class="chip dim">{$t(p.tagKey)}</span>
            {/if}
          </span>
          <span class="prov-status" title={rs?.detail ?? ""}>
            {#if rs}
              {#if rs.state === "connected"}
                {$t("providers.connected")}{rs.detail ? ` · ${rs.detail}` : ""}
              {:else if rs.state === "no_key"}
                {$t("providers.state.noKey")}
              {:else if rs.state === "unreachable"}
                {$t("providers.state.unreachable")}{rs.detail ? ` · ${rs.detail}` : ""}
              {:else}
                {$t("providers.notConnected")}
              {/if}
            {:else if p.local}
              {$t("providers.availableLocal")}
            {:else}
              {st.connected ? $t("providers.connected") : $t("providers.notConnected")}
            {/if}
          </span>
        </label>
        {#if isPaid && p.id !== "routerku"}
          {#if st.connected}
            <button class="btn-ghost sm" onclick={() => disconnectKey(p.id)}>
              {$t("providers.disconnect")}
            </button>
          {:else if keyFormFor !== p.id}
            <button class="btn-ghost sm" onclick={() => openKeyForm(p.id)}>
              {$t("providers.connect")}
            </button>
          {/if}
        {:else if !p.local && !st.connected && p.id !== "routerku"}
          <button class="btn-ghost sm" onclick={() => connectProvider(p.id)}>
            {$t("providers.connect")}
          </button>
        {/if}
        <label class="switch">
          <input
            type="checkbox"
            checked={st.active}
            onchange={(e) => toggleProvider(p.id, e.currentTarget.checked)}
            aria-label={p.name}
          />
          <span class="track" aria-hidden="true"></span>
        </label>
      </div>
      {#if isPaid && p.id !== "routerku" && keyFormFor === p.id}
        <div class="prov-detail">
          <div class="field">
            <label for="key-{p.id}">{$t("providers.keyLabel")} — {p.name}</label>
            <input
              id="key-{p.id}"
              type="password"
              bind:value={keyInput}
              placeholder={$t("providers.keyPlaceholder")}
              autocomplete="off"
            />
          </div>
          <p class="note">{$t("providers.keySaved")}</p>
          <div class="row-end">
            <button
              class="btn-primary"
              disabled={keyBusy || !keyInput.trim()}
              onclick={() => submitKeyForm(p.id)}
            >
              {$t("providers.routerku.save")}
            </button>
            <button class="btn-ghost sm" onclick={() => (keyFormFor = null)}>
              {$t("providers.cancel")}
            </button>
          </div>
        </div>
      {/if}
      {#if keySavedFor === p.id}
        <p class="ok-note" role="status">{$t("providers.keySaved")}</p>
      {/if}
      {#if p.id === "routerku"}
        <div class="prov-detail">
          <div class="field">
            <label for="routerku-key">{$t("providers.routerku.key")}</label>
            <input
              id="routerku-key"
              type="password"
              bind:value={rkKey}
              placeholder={$t("providers.routerku.keyPlaceholder")}
              autocomplete="off"
            />
          </div>
          <div class="field">
            <label for="routerku-baseurl">{$t("providers.routerku.baseUrl")}</label>
            <input
              id="routerku-baseurl"
              type="text"
              bind:value={rkUrl}
              placeholder={$t("providers.routerku.baseUrlPlaceholder")}
              inputmode="url"
            />
          </div>
          <p class="note">{$t("providers.routerku.note")}</p>
          <div class="row-end">
            <button class="btn-primary" disabled={rkBusy} onclick={saveRouterkuForm}>
              {$t("providers.routerku.save")}
            </button>
            {#if rkSaved}
              <span class="ok-note" role="status">{$t("providers.routerku.saved")}</span>
            {/if}
          </div>
        </div>
      {/if}
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
          <label for="prov-name">{$t("providers.customName")}</label>
          <input id="prov-name" type="text" bind:value={provName} />
        </div>
        <div class="field">
          <label for="prov-endpoint">{$t("providers.customEndpoint")}</label>
          <input id="prov-endpoint" type="text" bind:value={provEndpoint} placeholder="https://" />
        </div>
        <div class="actions">
          <button class="btn-ghost sm" onclick={() => (showProvDialog = false)}>
            {$t("providers.cancel")}
          </button>
          <button
            class="btn-primary"
            onclick={createProvider}
            disabled={!provName.trim() || !provEndpoint.trim()}
          >
            {$t("providers.add")}
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
  max-width: 1120px;
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
  border: 1px solid var(--border);
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
.lede.ok { color: var(--ok); }
@media (max-width: 900px) {
  .settings-layout { grid-template-columns: 230px minmax(0, 1fr); }
  .set-main { padding: 20px; }
}
</style>
