# Media Studio Pipeline Skill (Fase 2/3, 24 Okt 2026)

Media Studio adalah tempat agen Kreatif memproduksi konten nyata — bukan
hanya konsep teks. Skill ini mendeskripsikan interface tool yang BENAR-BENAR
tersedia di runtime agen Minime. Jangan mengarang tool lain.

## Tools yang tersedia

Agen memanggil tool dengan SATU blok per tool, JSON valid di dalamnya:

<tool_call>
{"name": "generate_image", "args": {"prompt": "deskripsi visual detail dalam Bahasa Inggris", "size": "portrait"}}
</tool_call>

- `generate_image(prompt: string, size: "square"|"portrait"|"portrait45"|"landscape")`:
  menghasilkan file gambar PNG dan mengembalikan path file-nya.
- Ukuran: poster rasio 9:16 → `"portrait"`; feed kotak → `"square"`;
  story/poster 4:5 → `"portrait45"`; banner lebar → `"landscape"`.
- Setelah tool dipanggil, hasilnya dikirim kembali sebagai pesan
  `HASIL TOOL`. Lanjutkan sampai semua output nyata selesai, lalu berikan
  jawaban akhir TANPA blok tool_call lagi.
- Gambar yang berhasil dibuat OTOMATIS dilampirkan ke pesan hasilmu di chat
  (backend yang mengurus path-nya). Sebutkan di jawaban akhir bahwa gambar
  sudah dibuat dan apa isinya.

## Aturan

- Bila tugas membutuhkan gambar (poster, thumbnail, visual konten),
  PANGGIL `generate_image` — jangan hanya mendeskripsikan gambar yang
  "seharusnya" dibuat.
- Prompt gambar ditulis dalam Bahasa Inggris yang detail: subjek, gaya
  visual, palet warna, komposisi, suasana, dan teks yang harus tampil di
  gambar (tulis teks persis seperti yang diminta, dalam bahasa yang diminta).
- Jangan mengarang path file atau mengklaim gambar sudah dibuat bila tool
  belum mengembalikannya. Bila tool gagal, sampaikan kegagalannya dengan
  jujur dan tetap berikan konsep teks sebagai fallback.
- Bahasa jawaban ke user: Indonesia (default), ikuti preferensi user.
- JANGAN auto-posting. Semua konten berhenti di draft/chat sampai user
  memutuskan langkah berikutnya.
- Satu tugas bisa memanggil tool beberapa kali (mis. beberapa variasi
  poster), selama masih dalam batas iterasi loop.
