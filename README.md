# AudioKeyConverter

AudioKeyConverter 是一個基於 **Tauri V2** 和 **Svelte** 的應用程式，專注於音檔升降 Key 轉換與輸出處理。其主要目的是提供快速、易用的音檔處理功能，並內建支援多種格式與功能。

## 功能特色

1. **音檔升降 Key 調整**
   - 支援通過 **Drag and Drop** 或 **檔案 URL** 匯入音檔（如 `.mp3`, `.wav` 等格式）。
   - 提供升降 Key 的選項，並可將調整後的音檔輸出。

2. **YouTube 音檔/影片下載與處理**
   - 支援輸入 **YouTube URL**，自動下載影片或音檔。
   - 可對下載的音檔進行升降 Key 調整並輸出。

3. **內建工具支援**
   - 內建處理工具（如 **ffmpeg**、**yt-dlp**），免除額外安裝依賴，簡化使用流程。

## 安裝與使用

### 前置需求
- 系統需求：
  - Windows、macOS 或 Linux
  - Node.js (僅在開發模式需要)
- 工具需求：
  - Tauri 2.x
  - Svelte

### 安裝
1. 克隆專案：
   ```bash
   git clone https://github.com/YuutaTsubasa/AudioKeyConverter.git
   cd AudioKeyConverter
   ```

2. 安裝依賴：
   ```bash
   npm install
   ```

3. 執行程式：
   ```bash
   npm run tauri dev
   ```

4. 打包應用程式 (可選)：
   ```bash
   npm run tauri build
   ```

## 目錄結構

```plaintext
AudioKeyConverter/
├── src/                # Source code (Svelte components)
├── public/             # Static assets
├── tauri.conf.json     # Tauri configuration
├── package.json        # Node.js dependencies
└── README.md           # 說明文件
```

## 貢獻

歡迎任何形式的貢獻！請遵循以下步驟參與開發：
1. Fork 本專案。
2. 創建分支：
   ```bash
   git checkout -b feature/<功能名稱>
   ```
3. 提交更改：
   ```bash
   git commit -m "新增 <功能名稱>"
   ```
4. 發送 Pull Request。

## 授權

此專案基於 [MIT License](LICENSE) 進行授權。

---

如果您有任何問題或建議，請透過 Issues 區塊與我們聯繫：[新增 Issue](https://github.com/YuutaTsubasa/AudioKeyConverter/issues/new)
