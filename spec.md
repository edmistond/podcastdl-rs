# Podcast Episode Downloader

## Overview
A command-line tool written in Rust that allows users to browse and download podcast episodes from RSS feeds. The tool provides a terminal user interface (TUI) for episode selection and displays download progress.

## Dependencies
- TUI Framework: ratatui
- Feed Parser: feed-rs (supports RSS/Atom/JSON Feed)

## Command Line Interface
The tool accepts the following arguments:
- Required: Feed URL or file path (string, position 1)
- Optional: `--max-size` Maximum download size in MB (default: 200)
- Optional: `--max-redirects` Maximum number of HTTP redirects to follow (default: 10)

## User Interface
### Layout
The TUI consists of two vertical panels:
1. Main Area (larger):
   - Scrollable list of episodes
   - Each entry shows:
     - Episode title
     - Publication date (format: "14 Jan 2024")
   - Arrow key navigation

2. Status Area (smaller):
   - Download progress information
   - Error messages
   - Status updates

### Controls
- ↑/↓: Navigate episode list
- D: Download selected episode
- Q: Quit application

## Core Functionality

### Feed Processing
- Support for RSS 2.0, Atom, and JSON Feed through feed-rs library
- Error handling for malformed feeds:
  - Display error in status area
  - Exit application

### Download Handling
- Save location: Current working directory
- Filename: Preserve original filename and extension from URL
- Progress indication:
  - Progress bar (when file size available)
  - Downloaded size / Total size (when available)
  - Activity indicator (always visible during download)
  - Displayed in status area

### Error Handling
- Network timeouts:
  - Maximum 3 retry attempts
  - Exponential backoff between attempts
  - Display retry status in status area
- HTTP redirects:
  - Follow up to configured maximum (default: 10)
  - Support for system curl for handling redirects
- Invalid media URLs:
  - Display error in status area
  - Return to episode selection
- File size limits:
  - Check against configured maximum (default: 200MB)
  - Abort download if exceeded
  - Display error and return to episode selection

## Future Enhancements (Out of Scope)
- Local SQLite database for feed caching
- Download cancellation
- Episode sorting/filtering
- Additional keyboard shortcuts