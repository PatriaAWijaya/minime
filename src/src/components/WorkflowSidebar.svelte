<script lang="ts">
  import { onMount } from "svelte";
  import { api, isTauri } from "../lib/daemon";

  interface Workflow {
    id: string;
    name: string;
    description: string;
  }

  let workflows = $state<Workflow[]>([]);
  let selected = $state<string | null>(null);

  // Daftar workflow statis sesuai pipelines/ di repo
  const STATIC_WORKFLOWS: Workflow[] = [
    { id: "minime-pipeline", name: "Minime Pipeline", description: "Pipeline utama: riset → strategi → kreatif → QC → approval" },
    { id: "konten-video-otomatis", name: "Konten Video Otomatis", description: "Generate → edit Pireel → QC → approval" },
    { id: "reels-pipeline", name: "Reels Pipeline", description: "Pipeline khusus reels Instagram" },
    { id: "smo-subpipeline", name: "SMO Subpipeline", description: "Social media optimization" },
    { id: "codebuilder-subpipeline", name: "Code Builder", description: "Subpipeline pembangunan kode" },
  ];

  onMount(() => {
    workflows = STATIC_WORKFLOWS;
    if (workflows.length > 0) selected = workflows[0].id;
  });

  export function getSelected(): string | null {
    return selected;
  }
</script>

<aside class="workflow-sidebar">
  <div class="ws-header">
    <span class="ws-title">Workflow</span>
  </div>
  <div class="ws-search">
    <input type="text" placeholder="Cari workflow…" aria-label="Cari workflow" />
  </div>
  <nav class="ws-list">
    {#each workflows as wf (wf.id)}
      <button
        class="ws-item"
        class:active={selected === wf.id}
        onclick={() => (selected = wf.id)}
        title={wf.description}
      >
        <span class="ws-icon">◈</span>
        <span class="ws-text">
          <span class="ws-name">{wf.name}</span>
          <span class="ws-desc">{wf.description}</span>
        </span>
      </button>
    {/each}
  </nav>
  {#if !isTauri}
    <div class="ws-demo">Mode demo</div>
  {/if}
</aside>

<style>
  .workflow-sidebar {
    width: 240px;
    min-width: 240px;
    background: #fafafa;
    border-right: 1px solid #e5e5e5;
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .ws-header {
    padding: 16px 16px 8px;
  }
  .ws-title {
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #666;
  }
  .ws-search {
    padding: 0 12px 12px;
  }
  .ws-search input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #ddd;
    border-radius: 8px;
    font-size: 14px;
    box-sizing: border-box;
  }
  .ws-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px;
  }
  .ws-item {
    display: flex;
    gap: 10px;
    width: 100%;
    text-align: left;
    padding: 10px 12px;
    border: none;
    background: transparent;
    border-radius: 8px;
    cursor: pointer;
    margin-bottom: 2px;
  }
  .ws-item:hover {
    background: #f0f0f0;
  }
  .ws-item.active {
    background: #e8e8e8;
  }
  .ws-icon {
    font-size: 18px;
    color: #888;
    flex-shrink: 0;
  }
  .ws-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .ws-name {
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ws-desc {
    font-size: 12px;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ws-demo {
    padding: 8px 16px;
    font-size: 12px;
    color: #999;
    border-top: 1px solid #e5e5e5;
  }
</style>
