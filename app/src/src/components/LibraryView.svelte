<script lang="ts">
  import { agents, getAgent, agentDisplayName } from "../lib/agents";
  import {
    agentNames,
    agentEnabled,
    libraryItems,
    deleteLibraryItem,
    pendingBrief,
    activeAgentId,
    activeView,
    customAgents,
  } from "../lib/store";
  import { t } from "../lib/i18n";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { LibraryItem, LibraryType } from "../lib/types";

  let typeFilter = $state<string>("all");
  let agentFilter = $state<string>("all");
  let taskFilter = $state<string>("all");
  let timeFilter = $state<string>("30");

  const types: LibraryType[] = ["doc", "table", "deck", "image", "video", "app"];
  const ranges = ["7", "30", "90", "365"];

  // Task options come from the items themselves (mock seeds + real results).
  const taskOptions = $derived(
    Array.from(new Set($libraryItems.map((i) => i.taskKey))),
  );

  const visible = $derived(
    $libraryItems.filter((i) => {
      if (typeFilter !== "all" && i.type !== typeFilter) return false;
      if (agentFilter !== "all" && i.agentId !== agentFilter) return false;
      if (taskFilter !== "all" && i.taskKey !== taskFilter) return false;
      if (i.daysAgo > Number(timeFilter)) return false;
      return true;
    }),
  );
  let viewItem: LibraryItem | null = $state(null);
  // Dialog pilih agen untuk "Input Tugas" + konfirmasi hapus dua langkah.
  let taskItem: LibraryItem | null = $state(null);
  let pickedAgent = $state("");
  let confirmDeleteId: number | null = $state(null);

  // Agen yang bisa diberi tugas: yang aktif saja (Mimi selalu aktif).
  const pickableAgents = $derived(
    [...agents, ...$customAgents].filter(
      (a) => a.id === "mimi" || ($agentEnabled[a.id] ?? false),
    ),
  );

  function openTaskPicker(item: LibraryItem) {
    taskItem = item;
    // Default: agen pembuat item bila masih aktif, sonst agen aktif pertama.
    pickedAgent = pickableAgents.some((a) => a.id === item.agentId)
      ? item.agentId
      : (pickableAgents[0]?.id ?? "");
  }

  function confirmTask() {
    if (!taskItem || !pickedAgent) return;
    const title = taskItem.title || (taskItem.titleKey ? $t(taskItem.titleKey) : "Output");
    const body = (taskItem.content ?? taskItem.summary ?? "").slice(0, 1500);
    // Brief masuk sebagai draft chat (bukan auto-kirim) agar user bisa
    // menambah detail sebelum mengirim ke agen yang dipilih.
    pendingBrief.set({
      agentId: pickedAgent,
      text: `${$t("library.fromLibrary")}: "${title}"\n\n${body}\n\n---\n${$t("library.extraBrief")} `,
    });
    activeAgentId.set(pickedAgent);
    activeView.set("agents");
    taskItem = null;
    viewItem = null;
  }

  function deleteItem(item: LibraryItem) {
    if (confirmDeleteId === item.id) {
      deleteLibraryItem(item.id);
      confirmDeleteId = null;
      if (viewItem?.id === item.id) viewItem = null;
    } else {
      confirmDeleteId = item.id;
    }
  }

  // Gambar item Library (24 Okt 2026): dari field images (item baru) +
  // fallback link markdown ![alt](path) di content (item lama yang hanya
  // menyimpan teks — gambarnya dulu hanya tampil sebagai path file://).
  function itemImages(item: LibraryItem): string[] {
    const out: string[] = [...(item.images ?? [])];
    if (item.content) {
      const re = /!\[\[^\]]*\]\(([^)]+)\)/g;
      let m: RegExpExecArray | null;
      while ((m = re.exec(item.content)) !== null) {
        let pth = m[1].trim().replace(/^file:\/\//, "");
        try {
          pth = decodeURIComponent(pth);
        } catch {}
        if (pth && !out.includes(pth)) out.push(pth);
      }
    }
    return out;
  }

  function downloadItem(item: LibraryItem) {
    const blob = new Blob([item.content ?? item.summary ?? ""], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (item.title ?? "output").replace(/[^a-zA-Z0-9-_ ]/g, "").trim() + ".txt";
    a.click();
    URL.revokeObjectURL(url);
  }

  function reuseItem(item: LibraryItem) {
    // Salin konten ke clipboard untuk dipakai di tugas berikutnya
    if (item.content) {
      navigator.clipboard.writeText(item.content).catch(() => {});
    }
  }
</script>

<div class="page">
  <div class="page-head">
    <h2>{$t("library.title")}</h2>
  </div>

  <div class="filters">
    <div class="field">
      <label for="f-type">{$t("library.type")}</label>
      <select id="f-type" bind:value={typeFilter}>
        <option value="all">{$t("library.all")}</option>
        {#each types as ty}
          <option value={ty}>{$t(`library.type.${ty}`)}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="f-agent">{$t("library.agent")}</label>
      <select id="f-agent" bind:value={agentFilter}>
        <option value="all">{$t("library.all")}</option>
        {#each agents as a}
          <option value={a.id}>{agentDisplayName(a, $agentNames, $t)}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="f-task">{$t("library.task")}</label>
      <select id="f-task" bind:value={taskFilter}>
        <option value="all">{$t("library.all")}</option>
        {#each taskOptions as tk}
          <option value={tk}>{$t(tk)}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="f-time">{$t("library.time")}</label>
      <select id="f-time" bind:value={timeFilter}>
        {#each ranges as r}
          <option value={r}>{$t(`library.time.${r}`)}</option>
        {/each}
      </select>
    </div>
  </div>

  {#if visible.length === 0}
    <p class="empty-note">{$t("library.empty")}</p>
  {:else}
    <div class="grid">
      {#each visible as item (item.id)}
        {@const imgs = itemImages(item)}
        <div class="card">
          <span class="chip">{$t(`library.type.${item.type}`)}</span>
          <h4>{item.title ?? $t(item.titleKey)}</h4>
          {#if imgs.length > 0}
            <img class="lib-img" src={convertFileSrc(imgs[0])} alt={item.title ?? ""} />
          {/if}
          {#if item.summary}
            <p class="summary">{item.summary}</p>
          {/if}
          <p>{$t("library.by", { agent: getAgent(item.agentId).name })} · {item.dateLabel ?? $t(item.dateKey)}</p>
          <div class="actions">
            {#if item.content}
              <button class="btn-sm" onclick={() => (viewItem = item)}>Lihat</button>
              <button class="btn-sm" onclick={() => downloadItem(item)}>Unduh</button>
              <button class="btn-sm" onclick={() => reuseItem(item)}>Pakai lagi</button>
            {/if}
            <button class="btn-sm" onclick={() => openTaskPicker(item)}>{$t("library.inputTask")}</button>
            <button
              class="btn-sm"
              class:danger={confirmDeleteId === item.id}
              onclick={() => deleteItem(item)}
            >{confirmDeleteId === item.id ? $t("library.confirmDelete") : $t("library.delete")}</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if taskItem}
    <div class="dialog-backdrop" onclick={() => (taskItem = null)}>
      <div class="dialog" onclick={(e) => e.stopPropagation()}>
        <h3>{$t("library.inputTask")}: {taskItem.title || (taskItem.titleKey ? $t(taskItem.titleKey) : "Output")}</h3>
        <p class="summary">{$t("library.chooseAgent")}</p>
        {#if pickableAgents.length === 0}
          <p class="summary">{$t("library.empty")}</p>
        {:else}
          <div class="radio-list">
            {#each pickableAgents as a (a.id)}
              <label class="radio-row">
                <input type="radio" name="task-agent" value={a.id} bind:group={pickedAgent} />
                <span>{agentDisplayName(a, $agentNames, $t)}</span>
              </label>
            {/each}
          </div>
        {/if}
        <div class="actions">
          <button class="btn-sm" disabled={!pickedAgent} onclick={confirmTask}>{$t("library.continueChat")}</button>
          <button class="btn-sm" onclick={() => (taskItem = null)}>Tutup</button>
        </div>
      </div>
    </div>
  {/if}

  {#if viewItem}
    <div class="dialog-backdrop" onclick={() => (viewItem = null)}>
      <div class="dialog" onclick={(e) => e.stopPropagation()}>
        <h3>{viewItem.title}</h3>
        {@const vimgs = itemImages(viewItem)}
        {#each vimgs as vp}
          <img class="lib-img-full" src={convertFileSrc(vp)} alt={viewItem.title ?? ""} />
        {/each}
        <pre class="content">{viewItem.content}</pre>
        <div class="actions">
          <button class="btn-sm" onclick={() => downloadItem(viewItem!)}>Unduh</button>
          <button class="btn-sm" onclick={() => { reuseItem(viewItem!); viewItem = null; }}>Salin & Tutup</button>
          <button class="btn-sm" onclick={() => openTaskPicker(viewItem!)}>{$t("library.inputTask")}</button>
          <button
            class="btn-sm"
            class:danger={confirmDeleteId === viewItem.id}
            onclick={() => deleteItem(viewItem!)}
          >{confirmDeleteId === viewItem.id ? $t("library.confirmDelete") : $t("library.delete")}</button>
          <button class="btn-sm" onclick={() => (viewItem = null)}>Tutup</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }
  .btn-sm {
    padding: 4px 10px;
    font-size: 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    /* Tombol tidak pernah terbelah kata ("Pakai lagi" jadi 2 baris) dan
       tidak keluar kartu: pindah baris utuh bila ruang habis. */
    white-space: nowrap;
    cursor: pointer;
  }
  .btn-sm:hover {
    background: var(--hover);
  }
  .btn-sm.danger {
    color: #ff6b6b;
    border-color: #ff6b6b;
  }
  .btn-sm:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .radio-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 12px 0;
  }
  .radio-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    cursor: pointer;
  }
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    max-width: 700px;
    max-height: 80vh;
    width: 90%;
    overflow: auto;
  }
  .dialog .content {
    white-space: pre-wrap;
    font-size: 13px;
    line-height: 1.6;
    max-height: 50vh;
    overflow: auto;
  }
  .summary {
    font-size: 13px;
    color: var(--muted);
    margin: 6px 0;
    line-height: 1.45;
  }
  /* Thumbnail kartu: potong rapi, tidak mendorong kartu terlalu tinggi */
  .lib-img {
    width: 100%;
    max-height: 200px;
    object-fit: cover;
    object-position: top;
    border-radius: 8px;
    margin: 6px 0;
    display: block;
  }
  /* Dialog: tampil penuh, tidak dipotong */
  .lib-img-full {
    width: 100%;
    max-height: 60vh;
    object-fit: contain;
    border-radius: 8px;
    margin: 6px 0;
    display: block;
  }
</style>
