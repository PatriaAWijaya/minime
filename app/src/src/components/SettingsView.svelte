<script lang="ts">
  import { lang, t, type Lang } from "../lib/i18n";

  const langs: { id: Lang; label: string }[] = [
    { id: "id", label: "Indonesia" },
    { id: "en", label: "English" },
    { id: "zh", label: "中文" },
  ];

  let notifyOn = $state(true);
  let checkedUpdates = $state(false);

  const APP_VERSION = "0.1.0";
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
</style>
