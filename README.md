# pm — Minimal Password Manager (Rust, CLI)

A local and secure password manager for **Linux / macOS / Windows**.

## 🔐 Features
- **Local storage** (each service = separate `.enc` file)
- **Encryption**: XChaCha20-Poly1305 (MK protected via Argon2id)
- **Master Key cache** (5 minutes TTL)
- **OTP/TOTP support** (compatible with Google Authenticator, Aegis, GitHub, etc.)
- **Clipboard integration** in GUI terminal (`pm clip`, `pm otp clip`)
- **Tree view listing** (`pm ls`)

📘 **[Русская версия README](https://github.com/Avdushin/pm/blob/main/docs/ru/README.md)** 

## 📦 One-command installation (no Rust required)

### 🐧 Linux
```bash
curl -sSfL https://raw.githubusercontent.com/Avdushin/pm/main/scripts/install-linux.sh | bash
```

### 🍎 macOS
```bash
curl -sSfL https://raw.githubusercontent.com/Avdushin/pm/main/scripts/install-macos.sh | bash
```

### 🪟 Windows (PowerShell)
```powershell
irm https://github.com/Avdushin/pm/releases/latest/download/install.ps1 | iex
```

After installation, verify:

```bash
pm --help
```

## 🧰 Usage

### Initialize password store
```bash
pm init
```

### Add a password entry  
(leave password empty to auto-generate)
```bash
pm add work/github
```

### View an entry
```bash
pm show work/github
pm show work/github --password-only
pm show work/github --json
```

### List entries
```bash
pm ls
pm ls work
```

### Copy password or username to clipboard
```bash
pm clip work/github
pm clip work/github --field username
```

### Add OTP (Base32 or `otpauth://` link)
```bash
pm otp add work/github
```

### Get or copy current TOTP code
```bash
pm otp show work/github
pm otp clip work/github
```

### Create backup (default: `.zip`)
```bash
pm backup create
pm backup create my_backup
pm backup create my_backup.tar.gz
```
