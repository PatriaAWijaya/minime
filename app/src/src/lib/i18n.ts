// Minimal hand-rolled i18n: Indonesian (default), English, Mandarin.
// Preference persists in localStorage. UI text only — no code comments here.

import { writable, derived } from "svelte/store";

export type Lang = "id" | "en" | "zh";

const KEY = "minime.lang";

function load(): Lang {
  try {
    const v = localStorage.getItem(KEY);
    return v === "en" || v === "zh" ? v : "id";
  } catch {
    return "id";
  }
}

export const lang = writable<Lang>(load());

lang.subscribe((v) => {
  try {
    localStorage.setItem(KEY, v);
    document.documentElement.lang = v === "zh" ? "zh-CN" : v;
  } catch {
    /* storage unavailable — keep in-memory value */
  }
});

type Dict = Record<string, string>;

const id: Dict = {
  "app.tagline": "Tim AI untuk bisnismu",
  "nav.newTask": "+ Tugas Baru",
  "nav.chat": "Chat",
  "nav.settings": "Pengaturan",
  "nav.usage": "Penggunaan",
  "nav.tools": "Peralatan",
  "agents.title": "Tim AI",
  "status.idle": "Siap",
  "status.working": "Bekerja",
  "status.attention": "Butuh kamu",
  "agent.mimi.role": "Koordinator tim",
  "agent.kreatif.role": "Bikin konten & desain",
  "agent.riset.role": "Riset & data bisnis",
  "agent.finance.role": "Keuangan & akuntansi",
  "agent.strategis.role": "Strategi & marketing",
  "agent.code.role": "Bikin aplikasi",
  "chat.empty": "Sapa {name} — ceritakan apa yang kamu butuhkan.",
  "chat.placeholder": "Tulis kebutuhanmu di sini...",
  "chat.send": "Kirim",
  "chat.thinking": "{name} sedang berpikir...",
  "chat.mockReply":
    "Baik, saya catat. Ini masih versi demo — kemampuan penuh saya aktif di Tahap 2.",
  "monitor.title": "Tugas Berjalan",
  "monitor.empty": "Belum ada tugas. Mulai dari tombol + Tugas Baru.",
  "monitor.mock1": "Riset kompetitor",
  "monitor.mock2": "Draf konten minggu ini",
  "settings.title": "Pengaturan",
  "settings.language": "Bahasa aplikasi",
  "settings.aiPerAgent": "Pilihan AI per anggota tim",
  "settings.aiKind": "Jenis AI",
  "settings.advanced": "Pengaturan lanjutan",
  "settings.advancedNote":
    "Opsi teknis kami sembunyikan agar tetap simpel. Pengaturan bawaan sudah optimal untuk kebanyakan pengguna.",
  "settings.note": "Pilihan tersimpan otomatis di perangkat ini.",
  "ai.gemini": "Gemini 3.8 (disarankan)",
  "ai.Muse": "Muse",
  "ai.local": "AI lokal gratis",
  "usage.title": "Penggunaan",
  "usage.messages": "Pesan terkirim",
  "usage.tasks": "Tugas selesai",
  "usage.note": "Statistik pemakaian lengkap tersedia di Tahap 2.",
  "tools.title": "Peralatan",
  "tools.connect": "Hubungkan",
  "tools.notConnected": "Belum terhubung",
  "tools.note":
    "Koneksi peralatan aktif di Tahap 3. Setiap publikasi tetap butuh persetujuanmu dulu.",
  "tools.meta": "Meta (Instagram & Facebook)",
  "tools.metaDesc": "Buat draf postingan — tanpa auto-posting",
  "tools.postiz": "Postiz / Viralin",
  "tools.postizDesc": "Jadwalkan konten — selalu mulai dari draf",
  "tools.pireel": "Pireel",
  "tools.pireelDesc": "Edit video dengan AI",
  "tools.drive": "Google Drive",
  "tools.driveDesc": "Simpan hasil kerja ke folder MIMI",
};

const en: Dict = {
  "app.tagline": "An AI team for your business",
  "nav.newTask": "+ New Task",
  "nav.chat": "Chat",
  "nav.settings": "Settings",
  "nav.usage": "Usage",
  "nav.tools": "Tools",
  "agents.title": "AI Team",
  "status.idle": "Ready",
  "status.working": "Working",
  "status.attention": "Needs you",
  "agent.mimi.role": "Team coordinator",
  "agent.kreatif.role": "Content & design",
  "agent.riset.role": "Research & business data",
  "agent.finance.role": "Finance & accounting",
  "agent.strategis.role": "Strategy & marketing",
  "agent.code.role": "Build apps",
  "chat.empty": "Say hi to {name} — tell them what you need.",
  "chat.placeholder": "Type what you need here...",
  "chat.send": "Send",
  "chat.thinking": "{name} is thinking...",
  "chat.mockReply":
    "Got it. This is still a demo — my full abilities arrive in Phase 2.",
  "monitor.title": "Running Tasks",
  "monitor.empty": "No tasks yet. Start with + New Task.",
  "monitor.mock1": "Competitor research",
  "monitor.mock2": "This week's content draft",
  "settings.title": "Settings",
  "settings.language": "App language",
  "settings.aiPerAgent": "AI choice per team member",
  "settings.aiKind": "AI type",
  "settings.advanced": "Advanced settings",
  "settings.advancedNote":
    "Technical options are hidden to keep things simple. The defaults are already optimal for most users.",
  "settings.note": "Choices are saved automatically on this device.",
  "ai.gemini": "Gemini 3.8 (recommended)",
  "ai.Muse": "Muse",
  "ai.local": "Free local AI",
  "usage.title": "Usage",
  "usage.messages": "Messages sent",
  "usage.tasks": "Tasks completed",
  "usage.note": "Full usage statistics arrive in Phase 2.",
  "tools.title": "Tools",
  "tools.connect": "Connect",
  "tools.notConnected": "Not connected",
  "tools.note":
    "Tool connections go live in Phase 3. Every publish still needs your approval first.",
  "tools.meta": "Meta (Instagram & Facebook)",
  "tools.metaDesc": "Draft posts — no auto-posting",
  "tools.postiz": "Postiz / Viralin",
  "tools.postizDesc": "Schedule content — always starts as a draft",
  "tools.pireel": "Pireel",
  "tools.pireelDesc": "Edit video with AI",
  "tools.drive": "Google Drive",
  "tools.driveDesc": "Save finished work to the MIMI folder",
};

const zh: Dict = {
  "app.tagline": "为您的业务打造的AI团队",
  "nav.newTask": "+ 新任务",
  "nav.chat": "聊天",
  "nav.settings": "设置",
  "nav.usage": "使用情况",
  "nav.tools": "工具",
  "agents.title": "AI团队",
  "status.idle": "就绪",
  "status.working": "工作中",
  "status.attention": "需要您",
  "agent.mimi.role": "团队协调员",
  "agent.kreatif.role": "内容与设计",
  "agent.riset.role": "调研与商业数据",
  "agent.finance.role": "财务与会计",
  "agent.strategis.role": "战略与营销",
  "agent.code.role": "构建应用",
  "chat.empty": "和{name}打个招呼——告诉他你需要什么。",
  "chat.placeholder": "在这里写下您的需求...",
  "chat.send": "发送",
  "chat.thinking": "{name}正在思考...",
  "chat.mockReply": "好的，我记下了。这是演示版——我的完整功能将在第二阶段上线。",
  "monitor.title": "运行中的任务",
  "monitor.empty": "还没有任务。从“+ 新任务”开始。",
  "monitor.mock1": "竞品调研",
  "monitor.mock2": "本周内容草稿",
  "settings.title": "设置",
  "settings.language": "应用语言",
  "settings.aiPerAgent": "每位团队成员的AI选择",
  "settings.aiKind": "AI类型",
  "settings.advanced": "高级设置",
  "settings.advancedNote": "技术选项已隐藏以保持简洁。默认设置已针对大多数用户优化。",
  "settings.note": "选择会自动保存在此设备上。",
  "ai.gemini": "Gemini 3.8（推荐）",
  "ai.Muse": "Muse",
  "ai.local": "免费本地AI",
  "usage.title": "使用情况",
  "usage.messages": "已发送消息",
  "usage.tasks": "已完成任务",
  "usage.note": "完整的使用统计将在第二阶段提供。",
  "tools.title": "工具",
  "tools.connect": "连接",
  "tools.notConnected": "未连接",
  "tools.note": "工具连接将在第三阶段上线。每次发布仍需您先批准。",
  "tools.meta": "Meta（Instagram和Facebook）",
  "tools.metaDesc": "起草帖子——不自动发布",
  "tools.postiz": "Postiz / Viralin",
  "tools.postizDesc": "安排内容——始终先起草",
  "tools.pireel": "Pireel",
  "tools.pireelDesc": "用AI剪辑视频",
  "tools.drive": "Google Drive",
  "tools.driveDesc": "将成果保存到MIMI文件夹",
};

const dicts: Record<Lang, Dict> = { id, en, zh };

/** Translate a key, with optional {var} interpolation. Falls back to Indonesian. */
export const t = derived(lang, ($lang) => {
  return (key: string, vars?: Record<string, string>): string => {
    let s = dicts[$lang][key] ?? dicts.id[key] ?? key;
    if (vars) {
      for (const [k, v] of Object.entries(vars)) s = s.replace(`{${k}}`, v);
    }
    return s;
  };
});
