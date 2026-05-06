# Corpus+

**Corpus+** is a modern document corpus management system designed for scanning, organizing, and processing document images into structured collections. It features a high-performance Rust backend and a dynamic, responsive React frontend.

## 🏗 Architecture

- **`corpus-server`**: A Rust-based backend (Axum) that manages metadata, file storage, and processing jobs.
- **`corpus-ui`**: A modern React frontend (Vite/TypeScript) for viewing and managing document corpora.
- **`doc-scanner`**: The core image processing engine used by the server for binarization, cleaning, and boundary detection.

## 🚀 Getting Started

The easiest way to run the entire stack is using **Docker Compose**.

### Prerequisites
- Docker & Docker Compose
- Node.js (for local UI development, optional)
- Rust (for local server development, optional)

### Running the Application

1. **Start with one command**:
   ```bash
   docker-compose up --build
   ```
2. **Access the UI**:
   Open [http://localhost:3000](http://localhost:3000) in your browser.
3. **Backend API**:
   The server runs on [http://localhost:8081](http://localhost:8081).

## 🛠 Management CLI

For convenience, you can create a `corpus` command to manage the application from anywhere. 

### 1. Create the script
Save the following as `~/Scripts/corpus` (or any folder in your PATH):

```bash
#!/bin/bash
PROJECT_DIR="$(pwd)" # Or hardcode your project path

case "$1" in
    run)     cd "$PROJECT_DIR" && docker-compose up -d ;;
    stop)    cd "$PROJECT_DIR" && docker-compose down ;;
    status)  cd "$PROJECT_DIR" && docker-compose ps ;;
    logs)    cd "$PROJECT_DIR" && docker-compose logs -f ;;
    backup)  docker run --rm -v corpus_corpus-data:/src -v /path/to/backup:/dst alpine rsync -av --delete /src/ /dst/ ;;
    *)       echo "Usage: corpus {run|stop|status|logs|backup}"; exit 1 ;;
esac
```

### 2. Make it executable
```bash
chmod +x ~/Scripts/corpus
```

### 3. Add to PATH (Optional)
Add this to your `~/.bashrc` or `~/.zshrc`:
```bash
export PATH="$PATH:$HOME/Scripts"
```
Now you can simply run `corpus run` or `corpus stop` from any terminal.

## 💾 Data & Backups

By default, all document data is stored in a Docker volume named `corpus_corpus-data`. 

To keep your data safe, use the provided backup script or run a manual sync:
```bash
docker run --rm -v corpus_corpus-data:/src -v /path/to/backup:/dst alpine rsync -av --delete /src/ /dst/
```
This will sync all uploaded images and metadata to your chosen backup directory.

## 🛠 Development

### Root Directory Structure
- `/corpus-server`: Rust backend source.
- `/corpus-ui`: React/Vite frontend source.
- `/doc-scanner`: Core Rust processing library.
- `docker-compose.yml`: Full stack orchestration.

## 📄 License
MIT
