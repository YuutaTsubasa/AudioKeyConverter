<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let audioFiles = $state([]);
  let selectedFile = $state(null);
  let semitones = $state(0);
  let outputFormat = $state("mp3");
  let processing = $state(false);
  let message = $state("");
  let youtubeUrl = $state("");
  let downloadingYoutube = $state(false);
  let systemInfo = $state(null);

  // File drop handling
  onMount(async () => {
    try {
      // Get system information
      systemInfo = await invoke("get_system_info");
      
      const unlisten = await listen("files-dropped", (event) => {
        handleFilesDrop(event.payload);
      });

      return () => {
        unlisten();
      };
    } catch (error) {
      console.error("Error setting up file drop listener:", error);
    }
  });

  async function handleFilesDrop(files) {
    for (const filePath of files) {
      // Check if it's an audio file
      const ext = filePath.split('.').pop().toLowerCase();
      if (['mp3', 'wav', 'flac', 'm4a', 'aac', 'ogg'].includes(ext)) {
        try {
          const fileInfo = await invoke("get_audio_info", { filePath });
          audioFiles = [...audioFiles, fileInfo];
        } catch (error) {
          console.error("Error getting file info:", error);
          message = "Error loading file: " + error;
        }
      }
    }
  }

  async function processAudio() {
    if (!selectedFile) {
      message = "Please select a file to process";
      return;
    }

    processing = true;
    message = "";

    try {
      const result = await invoke("process_audio_file", {
        filePath: selectedFile.path,
        options: {
          semitones: semitones,
          outputFormat: outputFormat,
          outputPath: selectedFile.path.replace(/\.[^/.]+$/, `_shifted.${outputFormat}`)
        }
      });
      message = result;
    } catch (error) {
      console.error("Error processing audio:", error);
      message = "Error processing audio: " + error;
    } finally {
      processing = false;
    }
  }

  async function downloadYouTube() {
    if (!youtubeUrl.trim()) {
      message = "Please enter a YouTube URL";
      return;
    }

    downloadingYoutube = true;
    message = "";

    try {
      const result = await invoke("download_youtube_audio", {
        url: youtubeUrl,
        outputDir: "."
      });
      message = result;
    } catch (error) {
      console.error("Error downloading YouTube audio:", error);
      message = "Error downloading YouTube audio: " + error;
    } finally {
      downloadingYoutube = false;
    }
  }

  function selectAudioFile(file) {
    selectedFile = file;
  }

  function removeFile(index) {
    audioFiles = audioFiles.filter((_, i) => i !== index);
    if (selectedFile && audioFiles.indexOf(selectedFile) === -1) {
      selectedFile = null;
    }
  }

  function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  // Simulate file selection for demo purposes
  function simulateFileSelection() {
    const demoFile = {
      name: "demo_audio.mp3",
      path: "/path/to/demo_audio.mp3", 
      size: 5242880, // 5MB
      duration: 180.5 // 3 minutes 30 seconds
    };
    audioFiles = [...audioFiles, demoFile];
    message = "Demo file added! (This is just for testing the UI)";
  }
</script>

<main class="container">
  <h1>Audio Key Converter</h1>
  <p class="subtitle">Convert audio pitch and download from YouTube</p>

  <!-- File Input Section -->
  <section class="file-section">
    <h2>Audio Files</h2>
    
    <div class="file-input-area">
      <div class="drop-zone">
        <p>Drag and drop audio files here</p>
        <p>or</p>
        <button type="button" onclick={simulateFileSelection} class="select-btn">
          Add Demo File (for testing)
        </button>
      </div>
    </div>

    {#if audioFiles.length > 0}
      <div class="file-list">
        <h3>Loaded Files:</h3>
        {#each audioFiles as file, index}
          <div class="file-item" class:selected={selectedFile === file}>
            <div class="file-info" role="button" tabindex="0" onclick={() => selectAudioFile(file)} onkeydown={(e) => e.key === 'Enter' && selectAudioFile(file)}>
              <span class="file-name">{file.name}</span>
              <span class="file-size">{formatFileSize(file.size)}</span>
            </div>
            <button type="button" onclick={() => removeFile(index)} class="remove-btn">
              ×
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <!-- YouTube Download Section -->
  <section class="youtube-section">
    <h2>YouTube Download</h2>
    <div class="youtube-input">
      <input 
        type="url" 
        bind:value={youtubeUrl} 
        placeholder="Enter YouTube URL"
        class="url-input"
      />
      <button 
        type="button" 
        onclick={downloadYouTube} 
        disabled={downloadingYoutube}
        class="download-btn"
      >
        {downloadingYoutube ? "Downloading..." : "Download"}
      </button>
    </div>
  </section>

  <!-- Processing Section -->
  <section class="processing-section">
    <h2>Audio Processing</h2>
    
    {#if selectedFile}
      <div class="selected-file">
        <h3>Selected File: {selectedFile.name}</h3>
      </div>
    {/if}

    <div class="controls">
      <div class="control-group">
        <label for="semitones">Pitch Shift (Semitones):</label>
        <input 
          type="range" 
          id="semitones" 
          bind:value={semitones} 
          min="-12" 
          max="12" 
          step="1"
          class="slider"
        />
        <span class="value">{semitones > 0 ? '+' : ''}{semitones}</span>
      </div>

      <div class="control-group">
        <label for="format">Output Format:</label>
        <select id="format" bind:value={outputFormat} class="format-select">
          <option value="mp3">MP3</option>
          <option value="wav">WAV</option>
          <option value="flac">FLAC</option>
          <option value="aac">AAC</option>
        </select>
      </div>

      <button 
        type="button" 
        onclick={processAudio} 
        disabled={processing || !selectedFile}
        class="process-btn"
      >
        {processing ? "Processing..." : "Process Audio"}
      </button>
    </div>
  </section>

  {#if message}
    <div class="message" class:error={message.includes("Error")}>
      {message}
    </div>
  {/if}

  <!-- System Information -->
  {#if systemInfo}
    <section class="system-info">
      <h2>System Information</h2>
      <div class="info-grid">
        <div class="info-item">
          <span class="label">Platform:</span>
          <span class="value">{systemInfo.platform}</span>
        </div>
        <div class="info-item">
          <span class="label">Architecture:</span>
          <span class="value">{systemInfo.arch}</span>
        </div>
        <div class="info-item">
          <span class="label">FFmpeg:</span>
          <span class="value" class:available={systemInfo.ffmpeg_available} class:unavailable={!systemInfo.ffmpeg_available}>
            {systemInfo.ffmpeg_available ? 'Available' : 'Not Available'}
          </span>
        </div>
        <div class="info-item">
          <span class="label">yt-dlp:</span>
          <span class="value" class:available={systemInfo.ytdlp_available} class:unavailable={!systemInfo.ytdlp_available}>
            {systemInfo.ytdlp_available ? 'Available' : 'Not Available'}
          </span>
        </div>
      </div>
      {#if !systemInfo.ffmpeg_available || !systemInfo.ytdlp_available}
        <div class="system-note">
          <p><strong>Note:</strong> This is a development build. In the production version, FFmpeg and yt-dlp would be bundled with the application.</p>
        </div>
      {/if}
    </section>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
  }

  .container {
    max-width: 900px;
    margin: 0 auto;
    padding: 20px;
    color: white;
  }

  h1 {
    text-align: center;
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
    text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
  }

  .subtitle {
    text-align: center;
    font-size: 1.1rem;
    opacity: 0.9;
    margin-bottom: 2rem;
  }

  section {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 15px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  h2 {
    margin-top: 0;
    margin-bottom: 1rem;
    font-size: 1.5rem;
  }

  .drop-zone {
    border: 2px dashed rgba(255, 255, 255, 0.5);
    border-radius: 10px;
    padding: 2rem;
    text-align: center;
    transition: all 0.3s ease;
    cursor: pointer;
  }

  .drop-zone:hover {
    border-color: rgba(255, 255, 255, 0.8);
    background: rgba(255, 255, 255, 0.05);
  }

  .select-btn {
    background: rgba(255, 255, 255, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.3);
    color: white;
    padding: 0.7rem 1.5rem;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1rem;
    transition: all 0.3s ease;
  }

  .select-btn:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .file-list {
    margin-top: 1rem;
  }

  .file-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(255, 255, 255, 0.1);
    padding: 0.7rem;
    margin-bottom: 0.5rem;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .file-item:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .file-item.selected {
    background: rgba(255, 255, 255, 0.3);
    border: 2px solid rgba(255, 255, 255, 0.5);
  }

  .file-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-grow: 1;
  }

  .file-name {
    font-weight: 500;
  }

  .file-size {
    opacity: 0.8;
    font-size: 0.9rem;
  }

  .remove-btn {
    background: rgba(255, 0, 0, 0.6);
    border: none;
    color: white;
    width: 25px;
    height: 25px;
    border-radius: 50%;
    cursor: pointer;
    font-size: 1.2rem;
    margin-left: 1rem;
  }

  .youtube-input {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .url-input {
    flex-grow: 1;
    padding: 0.7rem;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.1);
    color: white;
    font-size: 1rem;
  }

  .url-input::placeholder {
    color: rgba(255, 255, 255, 0.7);
  }

  .download-btn, .process-btn {
    background: rgba(0, 150, 255, 0.7);
    border: 1px solid rgba(0, 150, 255, 0.8);
    color: white;
    padding: 0.7rem 1.5rem;
    border-radius: 8px;
    cursor: pointer;
    font-size: 1rem;
    transition: all 0.3s ease;
  }

  .download-btn:hover, .process-btn:hover {
    background: rgba(0, 150, 255, 0.9);
  }

  .download-btn:disabled, .process-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .selected-file {
    background: rgba(255, 255, 255, 0.1);
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }

  .controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .control-group {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .control-group label {
    min-width: 180px;
    font-weight: 500;
  }

  .slider {
    flex-grow: 1;
    height: 6px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.3);
    outline: none;
  }

  .value {
    min-width: 40px;
    text-align: center;
    font-weight: bold;
    font-size: 1.1rem;
  }

  .format-select {
    padding: 0.5rem;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.1);
    color: white;
    font-size: 1rem;
  }

  .message {
    background: rgba(0, 255, 0, 0.2);
    border: 1px solid rgba(0, 255, 0, 0.5);
    padding: 1rem;
    border-radius: 8px;
    margin-top: 1rem;
  }

  .message.error {
    background: rgba(255, 0, 0, 0.2);
    border-color: rgba(255, 0, 0, 0.5);
  }

  .system-info {
    margin-top: 2rem;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(255, 255, 255, 0.1);
    padding: 0.5rem 1rem;
    border-radius: 6px;
  }

  .info-item .label {
    font-weight: 500;
  }

  .info-item .value {
    font-weight: bold;
  }

  .info-item .value.available {
    color: #4ade80;
  }

  .info-item .value.unavailable {
    color: #f87171;
  }

  .system-note {
    background: rgba(255, 193, 7, 0.2);
    border: 1px solid rgba(255, 193, 7, 0.5);
    padding: 1rem;
    border-radius: 8px;
    margin-top: 1rem;
  }

  .system-note p {
    margin: 0;
    font-size: 0.9rem;
  }

  @media (max-width: 768px) {
    .container {
      padding: 10px;
    }

    .youtube-input {
      flex-direction: column;
    }

    .control-group {
      flex-direction: column;
      align-items: stretch;
    }

    .control-group label {
      min-width: auto;
    }
  }
</style>
