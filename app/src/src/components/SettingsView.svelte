<script lang="ts">
  import { lang, t, type Lang } from "../lib/i18n";
  import {
    PROVIDERS,
    addCustomProvider,
    autoScan,
    connectProvider,
    customProviders,
    defaultProviderId,
    providerState,
    theme,
    toggleCustomProvider,
    toggleProvider,
  } from "../lib/store";
  import type { Theme } from "../lib/store";

  const themes: { id: Theme; labelKey: string }[] = [
    { id: "system", labelKey: "settings.theme.system" },
    { id: "dark", labelKey: "settings.theme.dark" },
    { id: "light", labelKey: "settings.theme.light" },
  ];

  const langs: { id: Lang; label: string }[] = [
    { id: "id", label: "Indonesia" },
    { id: "en", label: "English" },
    { id: "zh", label: "中文" },
  ];

  let notifyOn = $state(true);
  let checkedUpdates = $state(false);
  let scanning = $state(false);
  let showProvDialog = $state(false);
  let provName = $state("");
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

<div class="page">
  <div class="page-head">
    <h2>{$t("settings.title")}</h2>
  </div>

  <div class="section">
    <div class="field">
      <span class="group-label" id="lang-label">{$t("settings.language")}</span>
      <div class="seg" role="group" aria-labelledby="lang-label">
        {#each langs as l}
          <button
            class:active={$lang === l.id}
            onclick={() => lang.set(l.id)}
          >{l.label}</button>
        {/each}
      </div>
    </div>
  </div>

  <div class="section">
    <span class="group-label" id="theme-label">{$t("settings.theme")}</span>
    <div class="seg" role="radiogroup" aria-labelledby="theme-label">
      {#each themes as th}
        <button
          role="radio"
          aria-checked={$theme === th.id}
          class:active={$theme === th.id}
          onclick={() => theme.set(th.id)}
        >{$t(th.labelKey)}</button>
      {/each}
    </div>
  </div>

  <div class="section">
    <h3>{$t("providers.title")}</h3>

    <span class="group-label">{$t("providers.default")}</span>
    {#each PROVIDERS as p}
      {@const st = $providerState[p.id] ?? { connected: false, active: true }}
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
          <span class="prov-status">
            {p.local
              ? $t("providers.availableLocal")
              : st.connected
                ? $t("providers.connected")
                : $t("providers.notConnected")}
          </span>
        </label>
        {#if !p.local && !st.connected}
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

  <div class="section">
    <div class="row">
      <div class="meta">
        <span class="label">{$t("settings.notifications")}</span>
        <span class="value">{$t("settings.notifications.desc")}</span>
      </div>
      <label class="switch">
        <input
          type="checkbox"
          bind:checked={notifyOn}
          aria-label={$t("settings.notifications")}
        />
        <span class="track" aria-hidden="true"></span>
      </label>
    </div>
  </div>

  <div class="section">
    <h3>{$t("settings.general")}</h3>
    <div class="row">
      <span class="label">{$t("settings.version")}</span>
      <span class="value">{APP_VERSION}</span>
    </div>
    <p class="lede">{$t("settings.about")}</p>
  </div>

  <div class="section">
    <h3>{$t("settings.updates")}</h3>
    <div class="row">
      <span class="label">{$t("settings.checkUpdates")}</span>
      <button class="btn-ghost" onclick={() => (checkedUpdates = true)}>
        {$t("settings.checkUpdates")}
      </button>
    </div>
    {#if checkedUpdates}
      <p class="lede">{$t("settings.upToDate")}</p>
    {/if}
  </div>

  <p class="lede">{$t("settings.note")}</p>

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
</style>
