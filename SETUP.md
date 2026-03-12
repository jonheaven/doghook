# Kabosu Quick Setup Guide

## 1. Clone the Repository
```powershell
git clone <repo-url>
cd kabosu
```

## 2. Configure User-Specific Settings
- Copy the example config:
  ```powershell
  Copy-Item kabosu.toml.example kabosu.toml
  ```
- Edit `kabosu.toml` as needed. By default, user paths use the `%USERPROFILE%` environment variable for Windows (or `$HOME` for Unix).
- Set your Dogecoin Core RPC credentials as environment variables:
  ```powershell
  $env:DOGE_RPC_USERNAME="youruser"
  $env:DOGE_RPC_PASSWORD="yourpass"
  ```

## 3. Build the Project
```powershell
kabosu-build
```

## 4. Launch the Indexer
```powershell
kabosu-launch
```

## 5. (Optional) Refresh the .blk Index
```powershell
kabosu-refresh-blk-index
```

---

- All user-specific config (like `kabosu.toml`) is ignored by git.
- Example config and scripts use environment variables for portability.
- For advanced configuration, see `kabosu.toml.example` and CLI.md.
