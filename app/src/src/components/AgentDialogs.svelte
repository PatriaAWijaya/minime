<script lang="ts">
  import { t } from "../lib/i18n";

  interface Props {
    agentDlg: string | null;
    closeDialog: () => void;
    displayName: string;
    // Browser dialog
    browserOn: boolean;
    setBrowser: (v: boolean) => void;
    // Task dialog
    taskTitle: string;
    taskProject: string;
    taskAdded: boolean;
    // ... additional props as needed
  }

  let {
    agentDlg,
    closeDialog,
    displayName,
    browserOn,
    setBrowser,
    taskTitle = $bindable(),
    taskProject = $bindable(),
    taskAdded,
  }: Props = $props();
</script>

{#if agentDlg}
  <div class="scrim" onclick={closeDialog} aria-hidden="true"></div>
  <div class="agent-dlg" role="dialog" aria-modal="true" aria-label={$t("agents.menu.title")}>
    <button
      class="panel-x"
      onclick={closeDialog}
      aria-label={$t("common.close")}
      title={$t("common.close")}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
        <line x1="6" y1="6" x2="18" y2="18" />
        <line x1="18" y1="6" x2="6" y2="18" />
      </svg>
    </button>
    {#if agentDlg === "browser"}
      <h3>{$t("agents.browser.title")}</h3>
      <p class="dlg-agent">{displayName}</p>
      <div class="row">
        <span class="label">{$t("agents.browser.allow")}</span>
        <label class="switch">
          <input
            type="checkbox"
            checked={browserOn}
            onchange={(e) => setBrowser(e.currentTarget.checked)}
            aria-label={$t("agents.browser.allow")}
          />
          <span class="track" aria-hidden="true"></span>
        </label>
      </div>
      <p class="note">{$t("agents.browser.note")}</p>
    {/if}
    <!-- Additional dialog types (files, task, memories, skills, etc.) -->
    <!-- TODO: Migrate remaining dialog content from AgentsView -->
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 100;
  }
  .agent-dlg {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
    min-width: 400px;
    max-width: 90vw;
    max-height: 90vh;
    overflow-y: auto;
    z-index: 101;
  }
  .panel-x {
    position: absolute;
    top: 12px;
    right: 12px;
  }
</style>
